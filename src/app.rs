use tui_input::InputRequest;

use crate::browser::{BrowseLevel, BrowseMove, BrowseState, BrowseTarget};
use crate::help::{HelpMove, HelpState};
use crate::inspector::{
    InspectorField, InspectorFieldId, InspectorGroup, InspectorMove, InspectorState,
};
use crate::normalization::{NormalizationMove, NormalizationState};
use crate::preview::{GlyphPreviewState, GlyphPreviewUpdate};
use crate::search::{SearchMove, SearchState};
use crate::sequence::{SequenceMove, SequenceState};
use crate::unicode::CodePoint;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Quit,
    ToggleHelp,
    CloseHelp,
    MoveHelp(HelpMove),
    ResizeHelpViewport {
        viewport_height: usize,
        document_height: usize,
    },
    MoveCodePoint(CodePointMove),
    MoveInspector(InspectorMove),
    CopyInspectorValue,
    ResizeInspectorViewport {
        viewport_height: usize,
        document_height: usize,
        field_ranges: Vec<std::ops::Range<usize>>,
        groups: Vec<InspectorGroup>,
    },
    OpenBrowser(BrowseLevel),
    AdvanceBrowser,
    BackBrowser,
    CloseBrowser,
    MoveBrowser(BrowseMove),
    ResizeBrowserViewport(usize),
    OpenSearch,
    CloseSearch,
    InspectSearchResult,
    EditSearch(InputRequest),
    MoveSearch(SearchMove),
    ResizeSearchViewport(usize),
    MoveSequence(SequenceMove),
    ResizeSequenceViewport(usize),
    ResizeNormalizationOriginalViewport(usize),
    InspectSequenceCodePoint,
    ReturnToSequence,
    OpenNormalization,
    MoveNormalization(NormalizationMove),
    CloseNormalization,
    InspectNormalizationResult,
    ReturnToNormalization,
    CopyNormalizationResult,
    UpdateGlyphPreview(GlyphPreviewUpdate),
    ShowFooterStatus(FooterStatus),
}

impl Action {
    fn clears_footer_status(&self) -> bool {
        !matches!(
            self,
            Self::ResizeHelpViewport { .. }
                | Self::ResizeInspectorViewport { .. }
                | Self::ResizeBrowserViewport(_)
                | Self::ResizeSearchViewport(_)
                | Self::ResizeSequenceViewport(_)
                | Self::ResizeNormalizationOriginalViewport(_)
                | Self::UpdateGlyphPreview(_)
                | Self::ShowFooterStatus(_)
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodePointMove {
    Previous,
    Next,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Inspector,
    Browser,
    Search,
    Sequence,
    Normalization,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FooterStatusLevel {
    Info,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FooterStatus {
    level: FooterStatusLevel,
    message: String,
}

impl FooterStatus {
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            level: FooterStatusLevel::Info,
            message: message.into(),
        }
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            level: FooterStatusLevel::Warning,
            message: message.into(),
        }
    }

    pub const fn level(&self) -> FooterStatusLevel {
        self.level
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardRequest {
    label: &'static str,
    success_message: String,
    value: Option<String>,
}

impl ClipboardRequest {
    pub fn success_message(&self) -> &str {
        &self.success_message
    }

    pub const fn label(&self) -> &'static str {
        self.label
    }

    pub fn value(&self) -> Option<&str> {
        self.value.as_deref()
    }
}

#[derive(Debug, Clone)]
pub struct AppState {
    running: bool,
    view: View,
    selected: CodePoint,
    inspector: InspectorState,
    browse: Option<BrowseState>,
    search: Option<SearchState>,
    sequence: Option<SequenceState>,
    normalization: Option<NormalizationState>,
    showing_normalization_result: bool,
    help: HelpState,
    glyph_preview: GlyphPreviewState,
    clipboard_request: Option<ClipboardRequest>,
    footer_status: Option<FooterStatus>,
}

impl AppState {
    pub fn new() -> Self {
        let selected = CodePoint::new(0x0041).expect("U+0041 is a valid code point");
        Self {
            running: true,
            view: View::Inspector,
            selected,
            inspector: InspectorState::new(),
            browse: None,
            search: None,
            sequence: None,
            normalization: None,
            showing_normalization_result: false,
            help: HelpState::new(),
            glyph_preview: GlyphPreviewState::new(),
            clipboard_request: None,
            footer_status: None,
        }
    }

    pub fn with_selected(selected: CodePoint) -> Self {
        let mut state = Self::new();
        state.selected = selected;
        state
    }

    pub fn with_sequence(source: String) -> Self {
        let sequence = SequenceState::new(source);
        let selected = sequence.selected();
        let mut state = Self::with_selected(selected);
        state.view = View::Sequence;
        state.sequence = Some(sequence);
        state
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn view(&self) -> View {
        self.view
    }

    pub fn selected(&self) -> CodePoint {
        self.selected
    }

    pub const fn inspector(&self) -> &InspectorState {
        &self.inspector
    }

    pub fn browse(&self) -> Option<&BrowseState> {
        self.browse.as_ref()
    }

    pub fn search(&self) -> Option<&SearchState> {
        self.search.as_ref()
    }

    pub fn sequence(&self) -> Option<&SequenceState> {
        if self.showing_normalization_result {
            self.normalization
                .as_ref()
                .map(|state| &state.comparison().result)
        } else {
            self.sequence.as_ref()
        }
    }

    pub fn original_sequence(&self) -> Option<&SequenceState> {
        self.sequence.as_ref()
    }

    fn sequence_mut(&mut self) -> Option<&mut SequenceState> {
        if self.showing_normalization_result {
            self.normalization
                .as_mut()
                .map(|state| &mut state.comparison_mut().result)
        } else {
            self.sequence.as_mut()
        }
    }

    pub const fn showing_normalization_result(&self) -> bool {
        self.showing_normalization_result
    }

    pub fn normalization(&self) -> Option<&NormalizationState> {
        self.normalization.as_ref()
    }

    pub const fn help(&self) -> HelpState {
        self.help
    }

    pub const fn glyph_preview(&self) -> &GlyphPreviewState {
        &self.glyph_preview
    }

    pub fn take_clipboard_request(&mut self) -> Option<ClipboardRequest> {
        self.clipboard_request.take()
    }

    pub const fn footer_status(&self) -> Option<&FooterStatus> {
        self.footer_status.as_ref()
    }

    pub fn preview_code_point(&self) -> Option<CodePoint> {
        if self.help.is_open() {
            return None;
        }

        match self.view {
            View::Inspector => Some(self.selected),
            View::Browser => self.browse.as_ref().and_then(|browse| {
                (browse.level() == BrowseLevel::CodePointTable).then(|| browse.cursor())
            }),
            View::Search => self
                .search
                .as_ref()
                .and_then(SearchState::selected_result)
                .map(|result| result.code_point()),
            View::Sequence => self.sequence().map(SequenceState::selected),
            View::Normalization => None,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn update(state: &mut AppState, action: Action) {
    if action.clears_footer_status() {
        state.footer_status = None;
    }
    match action {
        Action::Quit => state.running = false,
        Action::ToggleHelp => state.help.toggle(),
        Action::CloseHelp => state.help.close(),
        Action::MoveHelp(movement) if state.help.is_open() => {
            state.help.move_viewport(movement);
        }
        Action::MoveHelp(_) => {}
        Action::ResizeHelpViewport {
            viewport_height,
            document_height,
        } if state.help.is_open() => state.help.resize_viewport(viewport_height, document_height),
        Action::ResizeHelpViewport { .. } => {}
        Action::MoveCodePoint(movement) if state.view == View::Inspector => {
            let selected = match movement {
                CodePointMove::Previous => state.selected.previous(),
                CodePointMove::Next => state.selected.next(),
            };
            if let Some(selected) = selected {
                select_code_point(state, selected);
            }
        }
        Action::MoveCodePoint(_) => {}
        Action::MoveInspector(movement) if state.view == View::Inspector => {
            state.inspector.move_selection(movement);
        }
        Action::MoveInspector(_) => {}
        Action::CopyInspectorValue if state.view == View::Inspector => {
            state.clipboard_request = InspectorField::for_code_point(state.selected)
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
                });
        }
        Action::CopyInspectorValue => {}
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
        Action::OpenBrowser(level) if state.view == View::Inspector => {
            let previous_preview = state.preview_code_point();
            state.browse = Some(BrowseState::at(level, state.selected));
            state.view = View::Browser;
            refresh_preview_for_change(state, previous_preview);
        }
        Action::OpenBrowser(_) => {}
        Action::AdvanceBrowser if state.view == View::Browser => {
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
        Action::AdvanceBrowser => {}
        Action::BackBrowser if state.view == View::Browser => {
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
        Action::BackBrowser => {}
        Action::CloseBrowser if state.view == View::Browser => {
            let previous_preview = state.preview_code_point();
            state.view = View::Inspector;
            state.browse = None;
            refresh_preview_for_change(state, previous_preview);
        }
        Action::CloseBrowser => {}
        Action::MoveBrowser(movement) if state.view == View::Browser => {
            let previous_preview = state.preview_code_point();
            let browse = state
                .browse
                .as_mut()
                .expect("the browser view always has browse state");
            browse.move_cursor(movement);
            refresh_preview_for_change(state, previous_preview);
        }
        Action::MoveBrowser(_) => {}
        Action::ResizeBrowserViewport(height) => {
            if let Some(browse) = state.browse.as_mut() {
                browse.resize_list_viewports(height);
            }
        }
        Action::OpenSearch if state.view == View::Inspector => {
            state.search.get_or_insert_with(SearchState::new);
            state.view = View::Search;
        }
        Action::OpenSearch => {}
        Action::CloseSearch if state.view == View::Search => state.view = View::Inspector,
        Action::CloseSearch => {}
        Action::InspectSearchResult if state.view == View::Search => {
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
        Action::InspectSearchResult => {}
        Action::EditSearch(request) if state.view == View::Search => state
            .search
            .as_mut()
            .expect("the search view always has search state")
            .edit(request),
        Action::EditSearch(_) => {}
        Action::MoveSearch(movement) if state.view == View::Search => {
            state
                .search
                .as_mut()
                .expect("the search view always has search state")
                .move_selection(movement);
        }
        Action::MoveSearch(_) => {}
        Action::ResizeSearchViewport(height) => {
            if let Some(search) = state.search.as_mut() {
                search.resize_viewport(height);
            }
        }
        Action::MoveSequence(movement) if state.view == View::Sequence => {
            let previous_preview = state.preview_code_point();
            state
                .sequence_mut()
                .expect("the sequence view always has sequence state")
                .move_selection(movement);
            follow_normalization_selection(state);
            refresh_preview_for_change(state, previous_preview);
        }
        Action::MoveSequence(_) => {}
        Action::ResizeSequenceViewport(height) => {
            if let Some(sequence) = state.sequence_mut() {
                sequence.resize_viewport(height);
            }
        }
        Action::ResizeNormalizationOriginalViewport(height)
            if state.view == View::Sequence && state.showing_normalization_result =>
        {
            state
                .normalization
                .as_mut()
                .unwrap()
                .comparison_mut()
                .resize_original_viewport(state.sequence.as_ref().unwrap().analysis(), height);
        }
        Action::ResizeNormalizationOriginalViewport(_) => {}
        Action::InspectSequenceCodePoint if state.view == View::Sequence => {
            let selected = state
                .sequence()
                .expect("the sequence view always has sequence state")
                .selected();
            select_code_point(state, selected);
            state.view = View::Inspector;
        }
        Action::InspectSequenceCodePoint => {}
        Action::ReturnToSequence if state.view == View::Inspector && state.sequence.is_some() => {
            let previous_preview = state.preview_code_point();
            state.view = View::Sequence;
            refresh_preview_for_change(state, previous_preview);
        }
        Action::ReturnToSequence => {}
        Action::OpenNormalization
            if state.view == View::Sequence && !state.showing_normalization_result =>
        {
            let previous_preview = state.preview_code_point();
            state.normalization.get_or_insert_with(|| {
                NormalizationState::new(state.sequence.as_ref().unwrap().analysis())
            });
            state.view = View::Normalization;
            refresh_preview_for_change(state, previous_preview);
        }
        Action::OpenNormalization => {}
        Action::MoveNormalization(movement) if state.view == View::Normalization => {
            state
                .normalization
                .as_mut()
                .unwrap()
                .move_selection(movement, state.sequence.as_ref().unwrap().analysis());
        }
        Action::MoveNormalization(_) => {}
        Action::CloseNormalization if state.view == View::Normalization => {
            let previous_preview = state.preview_code_point();
            state.view = View::Sequence;
            refresh_preview_for_change(state, previous_preview);
        }
        Action::CloseNormalization => {}
        Action::InspectNormalizationResult if state.view == View::Normalization => {
            let previous_preview = state.preview_code_point();
            state.showing_normalization_result = true;
            state.view = View::Sequence;
            follow_normalization_selection(state);
            refresh_preview_for_change(state, previous_preview);
        }
        Action::InspectNormalizationResult => {}
        Action::ReturnToNormalization
            if state.view == View::Sequence && state.showing_normalization_result =>
        {
            let previous_preview = state.preview_code_point();
            state.showing_normalization_result = false;
            state.view = View::Normalization;
            refresh_preview_for_change(state, previous_preview);
        }
        Action::ReturnToNormalization => {}
        Action::CopyNormalizationResult if state.view == View::Normalization => {
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
            });
        }
        Action::CopyNormalizationResult => {}
        Action::UpdateGlyphPreview(update) => state.glyph_preview.apply(update),
        Action::ShowFooterStatus(status) => state.footer_status = Some(status),
    }
}

fn select_code_point(state: &mut AppState, code_point: CodePoint) {
    if state.selected != code_point {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalization_reference_follows_result_without_moving_the_input_cursor_or_viewport() {
        let mut state = AppState::with_sequence("ﬃ".repeat(50));
        update(&mut state, Action::ResizeSequenceViewport(4));
        for _ in 0..10 {
            update(&mut state, Action::MoveSequence(SequenceMove::Next));
        }
        let original_index = state.sequence().unwrap().selected_index();
        let original_visible = state.sequence().unwrap().visible_range();
        update(&mut state, Action::OpenNormalization);
        for _ in 0..2 {
            update(
                &mut state,
                Action::MoveNormalization(NormalizationMove::Next),
            );
        }
        update(&mut state, Action::InspectNormalizationResult);
        update(&mut state, Action::ResizeNormalizationOriginalViewport(4));
        update(&mut state, Action::ResizeSequenceViewport(4));
        let original = state.original_sequence().unwrap().analysis();
        let comparison = state.normalization().unwrap().comparison();
        assert_eq!(comparison.original_selection(original), 0..1);
        assert_eq!(comparison.original_visible_range(original), 0..4);
        update(&mut state, Action::MoveSequence(SequenceMove::Last));
        let original = state.original_sequence().unwrap().analysis();
        let comparison = state.normalization().unwrap().comparison();
        assert_eq!(comparison.original_selection(original), 49..50);
        assert_eq!(comparison.original_visible_range(original), 46..50);
        assert_eq!(
            state.original_sequence().unwrap().selected_index(),
            original_index
        );
        assert_eq!(
            state.original_sequence().unwrap().visible_range(),
            original_visible
        );
        update(&mut state, Action::ReturnToNormalization);
        update(&mut state, Action::CloseNormalization);
        assert_eq!(state.sequence().unwrap().selected_index(), original_index);
        assert_eq!(state.sequence().unwrap().visible_range(), original_visible);
    }

    #[test]
    fn reference_highlights_all_source_members_of_a_composed_grapheme() {
        let mut state = AppState::with_sequence("A\u{0301}B".to_owned());
        update(&mut state, Action::OpenNormalization);
        update(&mut state, Action::InspectNormalizationResult);
        update(&mut state, Action::ResizeNormalizationOriginalViewport(2));
        let original = state.original_sequence().unwrap().analysis();
        let comparison = state.normalization().unwrap().comparison();
        assert_eq!(comparison.original_selection(original), 0..2);
        assert_eq!(comparison.original_visible_range(original), 0..2);
        update(&mut state, Action::MoveSequence(SequenceMove::Next));
        let original = state.original_sequence().unwrap().analysis();
        let comparison = state.normalization().unwrap().comparison();
        assert_eq!(comparison.original_selection(original), 2..3);
        assert_eq!(comparison.original_visible_range(original), 1..3);
    }

    #[test]
    fn normalization_preserves_the_source_selection_and_has_no_glyph_target() {
        let mut state = AppState::with_sequence("A\u{0301} ①👩‍💻".to_owned());
        update(&mut state, Action::MoveSequence(SequenceMove::Last));
        let original = state.sequence().unwrap().analysis().clone();
        update(&mut state, Action::OpenNormalization);
        assert_eq!(state.view(), View::Normalization);
        assert_eq!(state.preview_code_point(), None);
        update(
            &mut state,
            Action::MoveNormalization(NormalizationMove::Last),
        );
        update(&mut state, Action::MoveSequence(SequenceMove::First));
        assert_eq!(state.sequence().unwrap().selected_index(), 6);
        update(&mut state, Action::CloseNormalization);
        assert_eq!(state.view(), View::Sequence);
        assert_eq!(state.sequence().unwrap().analysis(), &original);
        assert_eq!(state.sequence().unwrap().selected_index(), 6);
        update(&mut state, Action::OpenNormalization);
        assert_eq!(
            state.normalization().unwrap().form(),
            crate::unicode::text::NormalizationForm::Nfkd
        );
    }

    #[test]
    fn normalization_actions_are_scoped_to_their_views() {
        let mut state = AppState::new();
        update(&mut state, Action::OpenNormalization);
        update(
            &mut state,
            Action::MoveNormalization(NormalizationMove::Next),
        );
        update(&mut state, Action::CloseNormalization);
        update(&mut state, Action::InspectNormalizationResult);
        update(&mut state, Action::ReturnToNormalization);
        update(&mut state, Action::CopyNormalizationResult);
        assert_eq!(state.view(), View::Inspector);
        assert!(state.normalization().is_none());
        assert!(state.take_clipboard_request().is_none());
    }

    #[test]
    fn normalization_result_navigation_restores_both_cursors_and_the_form() {
        let mut state = AppState::with_sequence("A\u{0301} ①👩‍💻".to_owned());
        update(&mut state, Action::MoveSequence(SequenceMove::Last));
        update(&mut state, Action::OpenNormalization);
        for _ in 0..2 {
            update(
                &mut state,
                Action::MoveNormalization(NormalizationMove::Next),
            );
        }
        update(&mut state, Action::InspectNormalizationResult);
        assert!(state.showing_normalization_result());
        assert_eq!(state.sequence().unwrap().analysis().source(), "Á 1👩‍💻");
        update(&mut state, Action::ResizeSequenceViewport(2));
        update(&mut state, Action::MoveSequence(SequenceMove::Last));
        update(&mut state, Action::InspectSequenceCodePoint);
        assert_eq!(state.selected().value(), 0x1f4bb);
        update(&mut state, Action::MoveCodePoint(CodePointMove::Previous));
        update(&mut state, Action::ReturnToSequence);
        assert_eq!(state.preview_code_point().unwrap().value(), 0x1f4bb);
        assert_eq!(state.sequence().unwrap().selected_index(), 5);
        assert_eq!(state.sequence().unwrap().visible_range(), 4..6);
        update(&mut state, Action::OpenNormalization);
        assert_eq!(state.view(), View::Sequence); // Do not normalize a derived result again.
        update(&mut state, Action::ReturnToNormalization);
        assert_eq!(state.view(), View::Normalization);
        assert!(!state.showing_normalization_result());
        assert_eq!(state.sequence().unwrap().selected_index(), 6);
        update(&mut state, Action::InspectNormalizationResult);
        assert_eq!(state.sequence().unwrap().selected_index(), 5);
        update(&mut state, Action::ReturnToNormalization);
        update(&mut state, Action::CloseNormalization);
        assert_eq!(
            state.sequence().unwrap().analysis().source(),
            "A\u{0301} ①👩‍💻"
        );
        assert_eq!(state.sequence().unwrap().selected_index(), 6);
    }

    #[test]
    fn each_normalization_form_keeps_its_own_result_selection() {
        let mut state = AppState::with_sequence("A\u{0301} ①".to_owned());
        update(&mut state, Action::OpenNormalization);
        update(&mut state, Action::InspectNormalizationResult);
        update(&mut state, Action::MoveSequence(SequenceMove::Last));
        update(&mut state, Action::ReturnToNormalization);
        update(
            &mut state,
            Action::MoveNormalization(NormalizationMove::Next),
        );
        update(&mut state, Action::InspectNormalizationResult);
        assert_eq!(state.sequence().unwrap().selected_index(), 0);
        update(&mut state, Action::MoveSequence(SequenceMove::Last));
        assert_eq!(state.sequence().unwrap().selected_index(), 3);
        update(&mut state, Action::ReturnToNormalization);
        update(
            &mut state,
            Action::MoveNormalization(NormalizationMove::Previous),
        );
        update(&mut state, Action::InspectNormalizationResult);
        assert_eq!(state.sequence().unwrap().selected_index(), 2);
    }

    #[test]
    fn inspecting_a_single_code_point_result_stays_in_sequence_navigation() {
        let mut state = AppState::with_sequence("A\u{0301}".to_owned());
        update(&mut state, Action::OpenNormalization);
        update(&mut state, Action::InspectNormalizationResult);
        assert_eq!(state.sequence().unwrap().code_points().len(), 1);
        assert_eq!(state.preview_code_point().unwrap().value(), 0x00c1);
        update(&mut state, Action::InspectSequenceCodePoint);
        update(&mut state, Action::ReturnToSequence);
        update(&mut state, Action::ReturnToNormalization);
        assert_eq!(state.view(), View::Normalization);
    }

    #[test]
    fn copy_requests_preserve_exact_normalized_and_unchanged_text() {
        let source = "e\u{0301}\t\r\n\u{1b}\u{202e}①👩‍💻";
        let mut state = AppState::with_sequence(source.to_owned());
        update(&mut state, Action::OpenNormalization);
        for form in crate::unicode::text::NormalizationForm::ALL {
            update(&mut state, Action::CopyNormalizationResult);
            let request = state.take_clipboard_request().unwrap();
            assert_eq!(
                request.value(),
                Some(state.sequence().unwrap().analysis().normalized_text(form))
            );
            assert_eq!(
                request.success_message(),
                format!("Copied {} Result", form.label())
            );
            assert!(state.take_clipboard_request().is_none());
            update(
                &mut state,
                Action::MoveNormalization(NormalizationMove::Next),
            );
        }
    }

    #[test]
    fn starts_running() {
        let state = AppState::new();

        assert!(state.is_running());
        assert_eq!(state.selected().value(), 0x0041);
        assert_eq!(state.inspector().offset(), 0);
        assert_eq!(state.browse(), None);
        assert!(state.search().is_none());
        assert!(!state.help().is_open());
        assert_eq!(
            state.glyph_preview().status(),
            crate::preview::GlyphPreviewStatus::Detecting
        );
    }

    #[test]
    fn help_opens_without_changing_the_base_view_and_resets_when_closed() {
        let mut state = AppState::new();
        update(&mut state, Action::OpenSearch);

        update(&mut state, Action::ToggleHelp);
        update(
            &mut state,
            Action::ResizeHelpViewport {
                viewport_height: 5,
                document_height: 20,
            },
        );
        update(&mut state, Action::MoveHelp(HelpMove::Last));

        assert!(state.help().is_open());
        assert_eq!(state.help().offset(), 15);
        assert_eq!(state.view(), View::Search);
        assert_eq!(state.preview_code_point(), None);

        update(&mut state, Action::CloseHelp);

        assert!(!state.help().is_open());
        assert_eq!(state.help().offset(), 0);
        assert_eq!(state.view(), View::Search);
    }

    #[test]
    fn quit_action_stops_the_app() {
        let mut state = AppState::new();

        update(&mut state, Action::Quit);

        assert!(!state.is_running());
    }

    #[test]
    fn preview_target_follows_only_code_point_selections() {
        let mut state = AppState::new();
        assert_eq!(state.preview_code_point().unwrap().value(), 0x0041);

        update(&mut state, Action::OpenBrowser(BrowseLevel::Plane));
        assert_eq!(state.preview_code_point(), None);
        update(&mut state, Action::AdvanceBrowser);
        assert_eq!(state.preview_code_point(), None);
        update(&mut state, Action::AdvanceBrowser);
        assert_eq!(state.preview_code_point().unwrap().value(), 0x0041);
        update(&mut state, Action::AdvanceBrowser);

        update(&mut state, Action::OpenSearch);
        assert_eq!(state.preview_code_point(), None);
        edit_search_query(&mut state, "U+2192");
        assert_eq!(state.preview_code_point().unwrap().value(), 0x2192);
        assert_eq!(state.selected().value(), 0x0041);
    }

    #[test]
    fn starts_with_the_requested_selection() {
        let selected = CodePoint::new(0x1f600).unwrap();
        let state = AppState::with_selected(selected);

        assert_eq!(state.view(), View::Inspector);
        assert_eq!(state.selected(), selected);
    }

    #[test]
    fn inspector_movement_is_limited_to_the_inspector() {
        let mut state = AppState::new();
        resize_inspector(&mut state, 5, 20);

        update(
            &mut state,
            Action::MoveInspector(InspectorMove::PageForward),
        );
        assert_eq!(state.inspector().selected_index(), 5);
        assert_eq!(state.inspector().offset(), 1);

        update(&mut state, Action::OpenSearch);
        update(&mut state, Action::MoveInspector(InspectorMove::NextField));
        assert_eq!(state.inspector().selected_index(), 5);
        assert_eq!(state.inspector().offset(), 1);
    }

    #[test]
    fn inspector_moves_to_adjacent_code_points_and_preserves_the_selected_field() {
        let mut state = AppState::with_selected(CodePoint::new(0xd7ff).unwrap());
        resize_inspector(&mut state, 5, 20);
        update(&mut state, Action::MoveInspector(InspectorMove::Last));

        update(&mut state, Action::MoveCodePoint(CodePointMove::Next));
        assert_eq!(state.selected().value(), 0xd800);
        assert_eq!(state.inspector().selected_index(), 19);
        assert_eq!(state.inspector().offset(), 15);

        update(&mut state, Action::MoveCodePoint(CodePointMove::Previous));
        assert_eq!(state.selected().value(), 0xd7ff);
    }

    #[test]
    fn adjacent_code_point_movement_stops_at_the_unicode_code_space_boundaries() {
        let mut first = AppState::with_selected(CodePoint::new(CodePoint::MIN_VALUE).unwrap());
        let mut last = AppState::with_selected(CodePoint::new(CodePoint::MAX_VALUE).unwrap());

        update(&mut first, Action::MoveCodePoint(CodePointMove::Previous));
        update(&mut last, Action::MoveCodePoint(CodePointMove::Next));

        assert_eq!(first.selected().value(), CodePoint::MIN_VALUE);
        assert_eq!(last.selected().value(), CodePoint::MAX_VALUE);
    }

    #[test]
    fn adjacent_code_point_movement_is_limited_to_the_inspector() {
        let selected = CodePoint::new(0x0041).unwrap();
        let mut browser = AppState::with_selected(selected);
        let mut search = AppState::with_selected(selected);
        update(&mut browser, Action::OpenBrowser(BrowseLevel::Plane));
        update(&mut search, Action::OpenSearch);

        update(&mut browser, Action::MoveCodePoint(CodePointMove::Next));
        update(&mut search, Action::MoveCodePoint(CodePointMove::Next));

        assert_eq!(browser.selected(), selected);
        assert_eq!(search.selected(), selected);
    }

    #[test]
    fn search_inspection_preserves_the_selected_inspector_field() {
        let mut state = AppState::new();
        resize_inspector(&mut state, 5, 20);
        update(&mut state, Action::MoveInspector(InspectorMove::Last));
        assert_eq!(state.inspector().offset(), 15);

        update(&mut state, Action::OpenSearch);
        edit_search_query(&mut state, "A");
        update(&mut state, Action::InspectSearchResult);
        assert_eq!(state.selected().value(), 0x0041);
        assert_eq!(state.inspector().offset(), 15);

        update(&mut state, Action::OpenSearch);
        update(&mut state, Action::EditSearch(InputRequest::DeleteLine));
        edit_search_query(&mut state, "U+2192");
        update(&mut state, Action::InspectSearchResult);
        assert_eq!(state.selected().value(), 0x2192);
        assert_eq!(state.inspector().selected_index(), 19);
        assert_eq!(state.inspector().offset(), 15);
    }

    #[test]
    fn selection_change_invalidates_a_prepared_glyph_preview_only_when_needed() {
        use crate::{
            glyph::CanvasSize,
            graphics::{GraphicsAvailability, GraphicsProtocol},
            image::kitty::ImageId,
            preview::{GlyphPreviewGeometry, GlyphPreviewStatus},
        };

        let mut state = AppState::new();
        let image_id = ImageId::new(7).unwrap();
        let canvas = CanvasSize::new(80, 64).unwrap();
        let geometry = GlyphPreviewGeometry::new(10, 4, canvas);
        update(
            &mut state,
            Action::UpdateGlyphPreview(GlyphPreviewUpdate::Configure {
                availability: GraphicsAvailability::Available(GraphicsProtocol::Kitty),
                image_id: Some(image_id),
            }),
        );
        update(
            &mut state,
            Action::UpdateGlyphPreview(GlyphPreviewUpdate::Prepared {
                image_id: Some(image_id),
                geometry,
                status: GlyphPreviewStatus::Ready,
                font: None,
            }),
        );

        update(&mut state, Action::OpenSearch);
        edit_search_query(&mut state, "A");
        update(&mut state, Action::InspectSearchResult);
        assert_eq!(state.glyph_preview().status(), GlyphPreviewStatus::Ready);

        update(&mut state, Action::MoveCodePoint(CodePointMove::Next));
        assert_eq!(state.selected().value(), 0x0042);
        assert_eq!(state.glyph_preview().status(), GlyphPreviewStatus::Pending);
        assert_eq!(state.glyph_preview().geometry(), Some(geometry));
        assert_eq!(state.glyph_preview().image_id(), Some(image_id));
    }

    #[test]
    fn closing_and_reopening_other_views_preserves_the_inspector_offset() {
        let mut state = AppState::new();
        resize_inspector(&mut state, 5, 20);
        update(&mut state, Action::MoveInspector(InspectorMove::Last));

        update(&mut state, Action::OpenSearch);
        update(&mut state, Action::CloseSearch);
        assert_eq!(state.inspector().offset(), 15);

        update(&mut state, Action::OpenBrowser(BrowseLevel::Plane));
        update(&mut state, Action::CloseBrowser);
        assert_eq!(state.inspector().offset(), 15);
    }

    #[test]
    fn opening_a_browser_level_uses_the_shared_selection() {
        let selected = CodePoint::new(0x1_23ab).unwrap();
        let mut state = AppState::with_selected(selected);

        update(&mut state, Action::OpenBrowser(BrowseLevel::CodePointTable));

        assert_eq!(state.view(), View::Browser);
        assert_eq!(state.browse().unwrap().level(), BrowseLevel::CodePointTable);
        assert_eq!(state.browse().unwrap().cursor(), selected);
        assert_eq!(state.selected(), selected);
    }

    #[test]
    fn plane_and_range_movement_do_not_change_the_shared_selection() {
        let selected = CodePoint::new(0x1_23ab).unwrap();
        let mut state = AppState::with_selected(selected);
        update(&mut state, Action::OpenBrowser(BrowseLevel::Plane));

        update(&mut state, Action::MoveBrowser(BrowseMove::Down));
        assert_eq!(state.browse().unwrap().cursor().value(), 0x2_23ab);
        assert_eq!(state.selected(), selected);

        update(&mut state, Action::AdvanceBrowser);
        update(&mut state, Action::MoveBrowser(BrowseMove::Down));
        assert_eq!(state.browse().unwrap().cursor().value(), 0x2_24ab);
        assert_eq!(state.selected(), selected);
    }

    #[test]
    fn table_movement_is_tentative_until_enter() {
        let mut state = AppState::with_selected(CodePoint::new(0x0041).unwrap());
        resize_inspector(&mut state, 5, 20);
        update(&mut state, Action::MoveInspector(InspectorMove::Last));
        update(&mut state, Action::OpenBrowser(BrowseLevel::Plane));
        update(&mut state, Action::AdvanceBrowser);
        update(&mut state, Action::MoveBrowser(BrowseMove::Down));

        update(&mut state, Action::AdvanceBrowser);
        assert_eq!(state.selected().value(), 0x0041);
        assert_eq!(state.preview_code_point().unwrap().value(), 0x0141);
        assert_eq!(state.inspector().selected_index(), 19);
        assert_eq!(state.inspector().offset(), 15);

        update(&mut state, Action::MoveBrowser(BrowseMove::Right));
        assert_eq!(state.browse().unwrap().cursor().value(), 0x0142);
        assert_eq!(state.selected().value(), 0x0041);
        update(&mut state, Action::AdvanceBrowser);
        assert_eq!(state.selected().value(), 0x0142);
        assert_eq!(state.view(), View::Inspector);
    }

    #[test]
    fn closing_or_backing_out_of_browse_discards_the_tentative_selection() {
        let original = CodePoint::new(0x0041).unwrap();
        let mut state = AppState::with_selected(original);
        update(&mut state, Action::OpenBrowser(BrowseLevel::CodePointTable));
        update(&mut state, Action::MoveBrowser(BrowseMove::Right));
        assert_eq!(state.preview_code_point().unwrap().value(), 0x0042);
        update(&mut state, Action::CloseBrowser);
        assert_eq!(state.selected(), original);
        assert_eq!(state.preview_code_point(), Some(original));

        update(&mut state, Action::OpenBrowser(BrowseLevel::CodePointTable));
        update(&mut state, Action::MoveBrowser(BrowseMove::Right));
        update(&mut state, Action::BackBrowser);
        assert_eq!(state.view(), View::Inspector);
        assert_eq!(state.selected(), original);
        assert_eq!(state.preview_code_point(), Some(original));
    }

    #[test]
    fn moving_a_tentative_cursor_invalidates_its_glyph_preview() {
        use crate::{
            glyph::CanvasSize,
            graphics::{GraphicsAvailability, GraphicsProtocol},
            preview::{GlyphPreviewGeometry, GlyphPreviewStatus},
        };

        let mut state = AppState::new();
        update(&mut state, Action::OpenBrowser(BrowseLevel::CodePointTable));
        update(
            &mut state,
            Action::UpdateGlyphPreview(GlyphPreviewUpdate::Configure {
                availability: GraphicsAvailability::Available(GraphicsProtocol::Kitty),
                image_id: None,
            }),
        );
        update(
            &mut state,
            Action::UpdateGlyphPreview(GlyphPreviewUpdate::Prepared {
                image_id: None,
                geometry: GlyphPreviewGeometry::new(10, 4, CanvasSize::new(80, 64).unwrap()),
                status: GlyphPreviewStatus::Ready,
                font: None,
            }),
        );

        update(&mut state, Action::MoveBrowser(BrowseMove::Right));
        assert_eq!(state.glyph_preview().status(), GlyphPreviewStatus::Pending);
        assert_eq!(state.selected().value(), 0x0041);
    }

    #[test]
    fn backing_from_a_table_preserves_its_tentative_cursor_in_the_previous_list() {
        let original = CodePoint::new(0x0041).unwrap();
        let mut state = AppState::with_selected(original);
        update(&mut state, Action::OpenBrowser(BrowseLevel::Range));
        update(&mut state, Action::MoveBrowser(BrowseMove::Down));
        update(&mut state, Action::AdvanceBrowser);
        update(&mut state, Action::MoveBrowser(BrowseMove::Right));
        update(&mut state, Action::BackBrowser);

        assert_eq!(state.browse().unwrap().level(), BrowseLevel::Range);
        assert_eq!(state.browse().unwrap().cursor().value(), 0x0142);
        assert_eq!(state.selected(), original);
    }

    #[test]
    fn block_browse_from_an_unmapped_code_point_commits_only_on_table_enter() {
        let original = CodePoint::new(0x2fe0).unwrap();
        let mut state = AppState::with_selected(original);
        update(&mut state, Action::OpenBrowser(BrowseLevel::Block));
        assert_eq!(state.browse().unwrap().cursor().value(), 0x2ff0);
        assert_eq!(state.selected(), original);
        update(&mut state, Action::AdvanceBrowser);
        assert_eq!(state.preview_code_point().unwrap().value(), 0x2ff0);
        update(&mut state, Action::MoveBrowser(BrowseMove::Right));
        update(&mut state, Action::BackBrowser);
        assert_eq!(state.browse().unwrap().level(), BrowseLevel::Block);
        assert_eq!(state.selected(), original);
        update(&mut state, Action::AdvanceBrowser);
        update(&mut state, Action::AdvanceBrowser);
        assert_eq!(state.view(), View::Inspector);
        assert_eq!(state.selected().value(), 0x2ff1);
    }

    #[test]
    fn browser_enter_and_back_actions_follow_the_hierarchy() {
        let mut state = AppState::new();
        update(&mut state, Action::OpenBrowser(BrowseLevel::Plane));

        update(&mut state, Action::AdvanceBrowser);
        assert_eq!(state.browse().unwrap().level(), BrowseLevel::Range);
        update(&mut state, Action::AdvanceBrowser);
        assert_eq!(state.browse().unwrap().level(), BrowseLevel::CodePointTable);
        update(&mut state, Action::AdvanceBrowser);
        assert_eq!(state.view(), View::Inspector);

        update(&mut state, Action::OpenBrowser(BrowseLevel::Plane));
        update(&mut state, Action::AdvanceBrowser);
        update(&mut state, Action::AdvanceBrowser);
        update(&mut state, Action::BackBrowser);
        assert_eq!(state.browse().unwrap().level(), BrowseLevel::Range);
        update(&mut state, Action::BackBrowser);
        assert_eq!(state.browse().unwrap().level(), BrowseLevel::Plane);
        update(&mut state, Action::BackBrowser);
        assert_eq!(state.view(), View::Inspector);

        update(&mut state, Action::OpenBrowser(BrowseLevel::CodePointTable));
        update(&mut state, Action::BackBrowser);
        assert_eq!(state.view(), View::Inspector);

        update(&mut state, Action::OpenBrowser(BrowseLevel::Range));
        update(&mut state, Action::AdvanceBrowser);
        update(&mut state, Action::BackBrowser);
        assert_eq!(state.browse().unwrap().level(), BrowseLevel::Range);
        update(&mut state, Action::BackBrowser);
        assert_eq!(state.view(), View::Inspector);
    }

    #[test]
    fn reopening_code_points_uses_the_selection_from_search() {
        let mut state = AppState::new();
        update(&mut state, Action::OpenBrowser(BrowseLevel::CodePointTable));
        update(&mut state, Action::MoveBrowser(BrowseMove::Right));
        assert_eq!(state.selected().value(), 0x0041);
        update(&mut state, Action::AdvanceBrowser);
        assert_eq!(state.selected().value(), 0x0042);

        update(&mut state, Action::OpenSearch);
        edit_search_query(&mut state, "→");
        update(&mut state, Action::InspectSearchResult);
        assert_eq!(state.selected().value(), 0x2192);

        update(&mut state, Action::OpenBrowser(BrowseLevel::CodePointTable));

        assert_eq!(state.view(), View::Browser);
        assert_eq!(state.browse().unwrap().level(), BrowseLevel::CodePointTable);
        assert_eq!(state.browse().unwrap().cursor().value(), 0x2192);
        assert_eq!(state.selected().value(), 0x2192);
    }

    #[test]
    fn first_open_lazily_initializes_an_empty_search_without_changing_selection() {
        let selected = CodePoint::new(0x1f600).unwrap();
        let mut state = AppState::with_selected(selected);

        update(&mut state, Action::OpenSearch);

        assert_eq!(state.view(), View::Search);
        assert_eq!(state.selected(), selected);
        assert_eq!(state.search().unwrap().input().value(), "");
        assert_eq!(state.search().unwrap().selected_result(), None);
    }

    #[test]
    fn sequence_selection_opens_the_inspector_and_returns_to_the_same_position() {
        let mut state = AppState::with_sequence("A→B".to_owned());
        update(&mut state, Action::ResizeSequenceViewport(2));
        update(&mut state, Action::MoveSequence(SequenceMove::Next));

        update(&mut state, Action::InspectSequenceCodePoint);
        assert_eq!(state.view(), View::Inspector);
        assert_eq!(state.selected().value(), 0x2192);

        update(&mut state, Action::MoveCodePoint(CodePointMove::Next));
        assert_eq!(state.selected().value(), 0x2193);
        update(&mut state, Action::ReturnToSequence);

        assert_eq!(state.view(), View::Sequence);
        assert_eq!(state.sequence().unwrap().selected_index(), 1);
        assert_eq!(state.preview_code_point().unwrap().value(), 0x2192);
    }

    #[test]
    fn search_from_a_sequence_inspector_does_not_change_the_sequence_position() {
        let mut state = AppState::with_sequence("A→B".to_owned());
        update(&mut state, Action::MoveSequence(SequenceMove::Next));
        update(&mut state, Action::InspectSequenceCodePoint);
        update(&mut state, Action::OpenSearch);
        edit_search_query(&mut state, "Ω");
        update(&mut state, Action::InspectSearchResult);

        assert_eq!(state.view(), View::Inspector);
        assert_eq!(state.selected().value(), 0x03a9);
        assert_eq!(state.sequence().unwrap().selected_index(), 1);

        update(&mut state, Action::ReturnToSequence);
        assert_eq!(state.view(), View::Sequence);
        assert_eq!(state.preview_code_point().unwrap().value(), 0x2192);
    }

    #[test]
    fn inspecting_and_searching_preserve_the_original_sequence_analysis() {
        let mut state = AppState::with_sequence("A\u{0301} 👩‍💻".to_owned());
        let analysis = state.sequence().unwrap().analysis().clone();
        update(&mut state, Action::ResizeSequenceViewport(2));
        update(&mut state, Action::MoveSequence(SequenceMove::Last));
        update(&mut state, Action::MoveSequence(SequenceMove::Previous));
        update(&mut state, Action::InspectSequenceCodePoint);
        assert_eq!(state.selected().value(), 0x200d);

        update(&mut state, Action::OpenSearch);
        edit_search_query(&mut state, "Ω");
        update(&mut state, Action::InspectSearchResult);
        assert_eq!(state.selected().value(), 0x03a9);
        update(&mut state, Action::ReturnToSequence);

        assert_eq!(state.view(), View::Sequence);
        assert_eq!(state.preview_code_point().unwrap().value(), 0x200d);
        let sequence = state.sequence().unwrap();
        assert_eq!(sequence.selected_index(), 4);
        assert_eq!(sequence.visible_range(), 4..6);
        assert_eq!(sequence.analysis(), &analysis);
    }

    #[test]
    fn browse_from_a_sequence_inspector_returns_through_the_inspector() {
        let mut state = AppState::with_sequence("AB".to_owned());
        update(&mut state, Action::InspectSequenceCodePoint);
        update(&mut state, Action::OpenBrowser(BrowseLevel::CodePointTable));
        update(&mut state, Action::MoveBrowser(BrowseMove::Right));
        update(&mut state, Action::AdvanceBrowser);

        assert_eq!(state.view(), View::Inspector);
        assert_eq!(state.selected().value(), 0x0042);
        assert_eq!(state.sequence().unwrap().selected_index(), 0);

        update(&mut state, Action::ReturnToSequence);
        assert_eq!(state.view(), View::Sequence);
        assert_eq!(state.preview_code_point().unwrap().value(), 0x0041);
    }

    #[test]
    fn search_cannot_open_from_the_browser() {
        let mut browser = AppState::new();
        update(&mut browser, Action::OpenBrowser(BrowseLevel::Plane));

        update(&mut browser, Action::OpenSearch);

        assert_eq!(browser.view(), View::Browser);
        assert!(browser.search().is_none());
    }

    #[test]
    fn editing_and_moving_search_results_do_not_change_the_shared_selection() {
        let mut state = AppState::new();
        update(&mut state, Action::OpenSearch);
        edit_search_query(&mut state, "rightwards arrow");
        let first = state.search().unwrap().selected_result().unwrap();

        update(&mut state, Action::MoveSearch(SearchMove::Next));

        assert_ne!(state.search().unwrap().selected_result(), Some(first));
        assert_eq!(state.selected().value(), 0x0041);
    }

    #[test]
    fn inspecting_a_search_result_updates_the_shared_selection_and_closes_search() {
        let mut state = AppState::new();
        resize_inspector(&mut state, 5, 20);
        update(&mut state, Action::MoveInspector(InspectorMove::Last));
        update(&mut state, Action::OpenSearch);
        edit_search_query(&mut state, "→");

        update(&mut state, Action::InspectSearchResult);

        assert_eq!(state.view(), View::Inspector);
        assert_eq!(state.selected().value(), 0x2192);
        assert_eq!(state.inspector().selected_index(), 19);
        assert_eq!(state.inspector().offset(), 15);
        assert_eq!(state.search().unwrap().input().value(), "→");
    }

    #[test]
    fn search_accepts_short_explicit_code_point_notation() {
        let mut state = AppState::new();
        update(&mut state, Action::OpenSearch);
        edit_search_query(&mut state, "U+A");

        update(&mut state, Action::InspectSearchResult);

        assert_eq!(state.view(), View::Inspector);
        assert_eq!(state.selected().value(), 0x000a);
    }

    #[test]
    fn inspecting_without_a_result_keeps_search_open_and_preserves_selection() {
        let mut state = AppState::new();
        update(&mut state, Action::OpenSearch);
        edit_search_query(&mut state, "not a unicode name");

        update(&mut state, Action::InspectSearchResult);

        assert_eq!(state.view(), View::Search);
        assert_eq!(state.selected().value(), 0x0041);
    }

    #[test]
    fn closing_and_reopening_search_retains_query_selection_and_viewport() {
        let mut state = AppState::new();
        update(&mut state, Action::OpenSearch);
        edit_search_query(&mut state, "rightwards arrow");
        update(&mut state, Action::ResizeSearchViewport(2));
        update(&mut state, Action::MoveSearch(SearchMove::Next));
        update(&mut state, Action::MoveSearch(SearchMove::Next));
        let selected = state.search().unwrap().selected_result();
        let visible = state.search().unwrap().visible_result_range();

        update(&mut state, Action::CloseSearch);
        assert_eq!(state.view(), View::Inspector);
        assert_eq!(state.selected().value(), 0x0041);
        update(&mut state, Action::OpenSearch);

        assert_eq!(state.view(), View::Search);
        assert_eq!(state.search().unwrap().input().value(), "rightwards arrow");
        assert_eq!(state.search().unwrap().selected_result(), selected);
        assert_eq!(state.search().unwrap().visible_result_range(), visible);
    }

    #[test]
    fn footer_status_survives_internal_updates_and_clears_on_the_next_user_action() {
        let mut state = AppState::new();
        update(
            &mut state,
            Action::ShowFooterStatus(FooterStatus::info("Copied Code Point")),
        );

        resize_inspector(&mut state, 10, 22);
        update(
            &mut state,
            Action::UpdateGlyphPreview(GlyphPreviewUpdate::Hidden),
        );
        assert_eq!(
            state.footer_status().map(FooterStatus::message),
            Some("Copied Code Point")
        );

        update(&mut state, Action::MoveInspector(InspectorMove::NextField));
        assert_eq!(state.footer_status(), None);
    }

    fn resize_inspector(state: &mut AppState, viewport_height: usize, document_height: usize) {
        update(
            state,
            Action::ResizeInspectorViewport {
                viewport_height,
                document_height,
                field_ranges: (0..document_height).map(|index| index..index + 1).collect(),
                groups: Vec::new(),
            },
        );
    }

    fn edit_search_query(state: &mut AppState, value: &str) {
        for character in value.chars() {
            update(
                state,
                Action::EditSearch(InputRequest::InsertChar(character)),
            );
        }
    }
}
