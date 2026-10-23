use crate::{
    app::{
        Action, AppState, BookmarkRequest, ClipboardRequest, CodePointMove, FooterStatus,
        FooterStatusLevel, View,
    },
    bookmarks::Bookmark,
    browser::{BrowseState, BrowseTarget},
    copy_dialog::CopyDialogState,
    inspector::{InspectorField, InspectorFieldId},
    normalization::NormalizationState,
    property::PropertyState,
    search::SearchState,
    unicode::CodePoint,
};

pub fn update(state: &mut AppState, action: Action) {
    // A dialog owns input without replacing the underlying screen or selection.
    if state.copy_dialog.is_some()
        && !matches!(
            action,
            Action::Quit
                | Action::ToggleHelp
                | Action::CloseHelp
                | Action::MoveHelp(_)
                | Action::ResizeHelpViewport { .. }
                | Action::OpenCopyDialog
                | Action::CloseCopyDialog
                | Action::MoveCopyDialog(_)
                | Action::CopyDialogSelection
                | Action::CompleteCopyDialog(_)
                | Action::UpdateGlyphPreview(_)
                | Action::ShowFooterStatus(_)
        )
    {
        return;
    }
    if state.property.is_some()
        && !matches!(
            action,
            Action::Quit
                | Action::ToggleHelp
                | Action::CloseHelp
                | Action::MoveHelp(_)
                | Action::ResizeHelpViewport { .. }
                | Action::CloseProperty
                | Action::MoveProperty(_)
                | Action::ResizePropertyViewport { .. }
                | Action::OpenPropertyReference
                | Action::CopyPropertyReference
                | Action::UpdateGlyphPreview(_)
                | Action::ShowFooterStatus(_)
        )
    {
        return;
    }
    if action.clears_footer_status() {
        state.footer_status = None;
    }
    match action {
        Action::ToggleHelp
        | Action::CloseHelp
        | Action::MoveHelp(_)
        | Action::ResizeHelpViewport { .. } => update_help(state, action),
        Action::CloseCopyDialog
        | Action::MoveCopyDialog(_)
        | Action::CopyDialogSelection
        | Action::CompleteCopyDialog(_) => update_copy_dialog(state, action),
        Action::OpenCopyDialog
            if !state.help.is_open()
                && state.copy_dialog.is_none()
                && matches!(state.view, View::Inspector | View::Sequence) =>
        {
            state.copy_dialog = Some(CopyDialogState::default());
        }
        Action::Quit => state.running = false,
        Action::CloseProperty
        | Action::MoveProperty(_)
        | Action::ResizePropertyViewport { .. }
        | Action::OpenPropertyReference
        | Action::CopyPropertyReference => update_property(state, action),
        Action::ResizeInspectorViewport {
            viewport_height,
            document_height,
            field_ranges,
            groups,
        } => {
            state
                .inspector
                .resize_viewport(viewport_height, document_height, field_ranges, groups)
        }
        Action::ResizeBookmarksViewport(height) => state.bookmarks.resize_viewport(height),
        Action::ToggleBookmark if !state.help.is_open() => {
            let code_point = match state.view {
                View::Inspector => Some(state.selected),
                View::Bookmarks => state.bookmarks.selected().map(Bookmark::code_point),
                _ => None,
            };
            if let Some(code_point) = code_point {
                state.bookmark_request = Some(BookmarkRequest {
                    code_point,
                    bookmarked: !state.bookmarks.contains(code_point),
                });
            }
        }
        Action::CompleteBookmark { entries, request } => {
            let previous_preview = state.preview_code_point();
            state.bookmarks.replace(entries);
            state.footer_status = Some(FooterStatus::info(format!(
                "{} {}",
                if request.bookmarked {
                    "Bookmarked"
                } else {
                    "Removed bookmark"
                },
                request.code_point,
            )));
            refresh_preview_for_change(state, previous_preview);
        }
        Action::ResizeBrowserViewport(height) => {
            if let Some(browse) = state.browse.as_mut() {
                browse.resize_list_viewports(height);
            }
        }
        Action::ResizeSearchViewport(height) => {
            if let Some(search) = state.search.as_mut() {
                search.resize_viewport(height);
            }
        }
        Action::ResizeSequenceViewport(height) => {
            if let Some(sequence) = state.sequence_mut() {
                sequence.resize_viewport(height);
            }
        }
        Action::UpdateGlyphPreview(update) => state.glyph_preview.apply(update),
        Action::ShowFooterStatus(status) => state.footer_status = Some(status),
        _ => match state.view {
            View::Inspector => update_inspector(state, action),
            View::Browser => update_browser(state, action),
            View::Search => update_search(state, action),
            View::Bookmarks => update_bookmarks(state, action),
            View::Sequence => update_sequence(state, action),
            View::Normalization => update_normalization(state, action),
        },
    }
}

fn update_help(state: &mut AppState, action: Action) {
    match action {
        Action::ToggleHelp => state.help.toggle(),
        Action::CloseHelp => state.help.close(),
        Action::MoveHelp(movement) if state.help.is_open() => {
            state.help.move_viewport(movement);
        }
        Action::ResizeHelpViewport {
            viewport_height,
            document_height,
        } if state.help.is_open() => state.help.resize_viewport(viewport_height, document_height),
        _ => {}
    }
}

fn update_property(state: &mut AppState, action: Action) {
    let Some(property) = state.property.as_mut() else {
        return;
    };
    if state.help.is_open() {
        return;
    }
    match action {
        Action::CloseProperty => state.property = None,
        Action::MoveProperty(movement) => property.viewport_mut().move_viewport(movement),
        Action::ResizePropertyViewport {
            viewport_height,
            document_height,
        } => property
            .viewport_mut()
            .resize_viewport(viewport_height, document_height),
        Action::OpenPropertyReference => state.reference_request = Some(property.reference().url),
        Action::CopyPropertyReference => {
            state.clipboard_request = Some(ClipboardRequest {
                label: "Reference URL",
                success_message: "Copied reference URL".to_owned(),
                value: Some(property.reference().url.to_owned()),
                from_copy_dialog: false,
            })
        }
        _ => {}
    }
}

fn update_inspector(state: &mut AppState, action: Action) {
    match action {
        Action::OpenProperty if !state.help.is_open() => {
            state.property =
                InspectorField::for_code_point(state.selected, state.inspector.display_mode())
                    .into_iter()
                    .nth(state.inspector.selected_index())
                    .map(|field| PropertyState::new(state.selected, field));
        }
        Action::MoveCodePoint(movement) => {
            let selected = match movement {
                CodePointMove::Previous => state.selected.previous(),
                CodePointMove::Next => state.selected.next(),
            };
            if let Some(selected) = selected {
                select_code_point(state, selected);
            }
        }
        Action::MoveInspector(movement) => {
            state.inspector.move_selection(movement);
        }
        Action::ToggleInspectorProperties => {
            state.inspector.toggle_display_mode(state.selected);
        }
        Action::CopyInspectorValue => {
            state.clipboard_request =
                InspectorField::for_code_point(state.selected, state.inspector.display_mode())
                    .into_iter()
                    .nth(state.inspector.selected_index())
                    .map(|field| ClipboardRequest {
                        label: field.label(),
                        success_message: if field.id() == InspectorFieldId::Character {
                            format!("Copied Character: {}", state.selected)
                        } else {
                            format!("Copied {}", field.label())
                        },
                        value: field.copy_value().map(str::to_owned),
                        from_copy_dialog: false,
                    });
        }
        Action::OpenBrowser(level) => {
            let previous_preview = state.preview_code_point();
            state.browse = Some(BrowseState::at(level, state.selected));
            state.view = View::Browser;
            refresh_preview_for_change(state, previous_preview);
        }
        Action::OpenBookmarks => {
            let previous_preview = state.preview_code_point();
            if !state.inspecting_bookmark {
                state.bookmark_origin = Some(state.selected);
            }
            state.bookmarks.select(state.selected);
            state.view = View::Bookmarks;
            refresh_preview_for_change(state, previous_preview);
        }
        Action::ReturnToBookmarks if state.inspecting_bookmark => {
            let previous_preview = state.preview_code_point();
            state.view = View::Bookmarks;
            refresh_preview_for_change(state, previous_preview);
        }
        Action::OpenSearch => {
            state.search.get_or_insert_with(SearchState::default);
            state.view = View::Search;
        }
        Action::ReturnToSequence if state.sequence.is_some() && !state.inspecting_bookmark => {
            let previous_preview = state.preview_code_point();
            state.view = View::Sequence;
            refresh_preview_for_change(state, previous_preview);
        }
        _ => {}
    }
}

fn update_browser(state: &mut AppState, action: Action) {
    match action {
        Action::AdvanceBrowser => {
            let previous_preview = state.preview_code_point();
            let (target, cursor) = {
                let browse = state
                    .browse
                    .as_mut()
                    .expect("the browser view always has browse state");
                (browse.advance(), browse.cursor())
            };
            if target == BrowseTarget::Inspector {
                select_code_point(state, cursor);
                state.view = View::Inspector;
                state.browse = None;
            }
            refresh_preview_for_change(state, previous_preview);
        }
        Action::BackBrowser => {
            let previous_preview = state.preview_code_point();
            let browse = state
                .browse
                .as_mut()
                .expect("the browser view always has browse state");
            if browse.back() == BrowseTarget::Inspector {
                state.view = View::Inspector;
                state.browse = None;
            }
            refresh_preview_for_change(state, previous_preview);
        }
        Action::CloseBrowser => {
            let previous_preview = state.preview_code_point();
            state.view = View::Inspector;
            state.browse = None;
            refresh_preview_for_change(state, previous_preview);
        }
        Action::MoveBrowser(movement) => {
            let previous_preview = state.preview_code_point();
            let browse = state
                .browse
                .as_mut()
                .expect("the browser view always has browse state");
            browse.move_cursor(movement);
            refresh_preview_for_change(state, previous_preview);
        }
        _ => {}
    }
}

fn update_search(state: &mut AppState, action: Action) {
    match action {
        Action::CloseSearch => state.view = View::Inspector,
        Action::InspectSearchResult => {
            if let Some(result) = state
                .search
                .as_ref()
                .expect("the search view always has search state")
                .selected_result()
            {
                select_code_point(state, result.code_point());
                state.view = View::Inspector;
            }
        }
        Action::EditSearch(request) => state
            .search
            .as_mut()
            .expect("the search view always has search state")
            .edit(request),
        Action::MoveSearch(movement) => {
            state
                .search
                .as_mut()
                .expect("the search view always has search state")
                .move_selection(movement);
        }
        _ => {}
    }
}

fn update_bookmarks(state: &mut AppState, action: Action) {
    match action {
        Action::SwitchBookmarkSort if !state.help.is_open() => state.bookmarks.switch_sort(),
        Action::CloseBookmarks => {
            let previous_preview = state.preview_code_point();
            if let Some(origin) = state.bookmark_origin.take() {
                select_code_point(state, origin);
            }
            state.inspecting_bookmark = false;
            state.view = View::Inspector;
            refresh_preview_for_change(state, previous_preview);
        }
        Action::MoveBookmarks(movement) => {
            let previous_preview = state.preview_code_point();
            state.bookmarks.move_selection(movement);
            refresh_preview_for_change(state, previous_preview);
        }
        Action::InspectBookmark => {
            if let Some(entry) = state.bookmarks.selected() {
                let previous_preview = state.preview_code_point();
                select_code_point(state, entry.code_point());
                state.inspecting_bookmark = true;
                state.view = View::Inspector;
                refresh_preview_for_change(state, previous_preview);
            }
        }
        _ => {}
    }
}

fn update_sequence(state: &mut AppState, action: Action) {
    match action {
        Action::MoveSequence(movement) => {
            let previous_preview = state.preview_code_point();
            state
                .sequence_mut()
                .expect("the sequence view always has sequence state")
                .move_selection(movement);
            follow_normalization_selection(state);
            refresh_preview_for_change(state, previous_preview);
        }
        Action::ResizeNormalizationOriginalViewport(height)
            if state.showing_normalization_result =>
        {
            state
                .normalization
                .as_mut()
                .unwrap()
                .comparison_mut()
                .resize_original_viewport(state.sequence.as_ref().unwrap().analysis(), height);
        }
        Action::InspectSequenceCodePoint => {
            state.inspecting_bookmark = false;
            let selected = state
                .sequence()
                .expect("the sequence view always has sequence state")
                .selected();
            select_code_point(state, selected);
            state.view = View::Inspector;
        }
        Action::OpenNormalization if !state.showing_normalization_result => {
            let previous_preview = state.preview_code_point();
            state.normalization.get_or_insert_with(|| {
                NormalizationState::new(state.sequence.as_ref().unwrap().analysis())
            });
            state.view = View::Normalization;
            refresh_preview_for_change(state, previous_preview);
        }
        Action::ReturnToNormalization if state.showing_normalization_result => {
            let previous_preview = state.preview_code_point();
            state.showing_normalization_result = false;
            state.view = View::Normalization;
            refresh_preview_for_change(state, previous_preview);
        }
        _ => {}
    }
}

fn update_normalization(state: &mut AppState, action: Action) {
    match action {
        Action::MoveNormalization(movement) => {
            state
                .normalization
                .as_mut()
                .unwrap()
                .move_selection(movement, state.sequence.as_ref().unwrap().analysis());
        }
        Action::CloseNormalization => {
            let previous_preview = state.preview_code_point();
            state.view = View::Sequence;
            refresh_preview_for_change(state, previous_preview);
        }
        Action::InspectNormalizationResult => {
            let previous_preview = state.preview_code_point();
            state.showing_normalization_result = true;
            state.view = View::Sequence;
            follow_normalization_selection(state);
            refresh_preview_for_change(state, previous_preview);
        }
        Action::CopyNormalizationResult => {
            let normalization = state.normalization.as_ref().unwrap();
            state.clipboard_request = Some(ClipboardRequest {
                label: normalization.form().label(),
                success_message: format!("Copied {} Result", normalization.form().label()),
                value: Some(
                    normalization
                        .comparison()
                        .result
                        .analysis()
                        .source()
                        .to_owned(),
                ),
                from_copy_dialog: false,
            });
        }
        _ => {}
    }
}

fn update_copy_dialog(state: &mut AppState, action: Action) {
    match action {
        Action::CloseCopyDialog if !state.help.is_open() => state.copy_dialog = None,
        Action::MoveCopyDialog(movement) if !state.help.is_open() => {
            if let Some(mut dialog) = state.copy_dialog {
                dialog.move_selection(movement, state.copy_source().unwrap());
                state.copy_dialog = Some(dialog);
            }
        }
        Action::CopyDialogSelection if !state.help.is_open() && state.copy_dialog.is_some() => {
            let source = state.copy_source().unwrap();
            let dialog = state.copy_dialog.as_ref().unwrap();
            let target = dialog.selected_target(source);
            let format = dialog.selected_format(source);
            let value = source.value(target).format(format);
            state.clipboard_request = Some(ClipboardRequest {
                label: target.label(),
                success_message: format!(
                    "Copied {} as {} ({})",
                    source.target_label(target),
                    format.label(),
                    source.origin_label(),
                ),
                value,
                from_copy_dialog: true,
            });
        }
        Action::CompleteCopyDialog(status) if state.copy_dialog.is_some() => {
            if status.level() == FooterStatusLevel::Info {
                state.copy_dialog = None;
            }
            state.footer_status = Some(status);
        }
        _ => {}
    }
}

fn select_code_point(state: &mut AppState, code_point: CodePoint) {
    if state.selected != code_point {
        state
            .inspector
            .follow_code_point(state.selected, code_point);
        state.selected = code_point;
        state.glyph_preview.selection_changed();
    }
}

fn follow_normalization_selection(state: &mut AppState) {
    if state.showing_normalization_result {
        state
            .normalization
            .as_mut()
            .unwrap()
            .comparison_mut()
            .follow_selection(state.sequence.as_ref().unwrap().analysis());
    }
}

fn refresh_preview_for_change(state: &mut AppState, previous: Option<CodePoint>) {
    if state.preview_code_point() != previous {
        state.glyph_preview.selection_changed();
    }
}
