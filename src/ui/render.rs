use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
};

use crate::{
    app::{AppState, FooterStatusLevel, View},
    input::context_for_state,
    keybindings::{Context, ResolvedKeymap},
    ui::{
        browser, glyph_preview, help, inspector,
        layout::{MINIMUM_SIZE, calculate},
        normalization, search, sequence,
        settings::UiSettings,
        theme::ColorTheme,
    },
};

pub fn render(
    frame: &mut Frame,
    state: &AppState,
    color_theme: &ColorTheme,
    ui: &UiSettings,
    keymap: &ResolvedKeymap,
) {
    let area = frame.area();
    frame.render_widget(Block::default().style(color_theme.base_style()), area);
    let Some(layout) = calculate(area) else {
        render_size_warning(frame, area, color_theme);
        return;
    };

    render_header(frame, layout.header, state, color_theme);
    if state.help().is_open() {
        help::render(frame, layout.main, state, keymap, color_theme);
    } else {
        match state.view() {
            View::Inspector => {
                let inspector_layout = crate::ui::layout::inspector(layout.main);
                if let Some(preview) = inspector_layout.preview {
                    glyph_preview::render(frame, preview, state.glyph_preview(), color_theme);
                }
                inspector::render(frame, inspector_layout.details, state, color_theme);
            }
            View::Browser => browser::render(frame, layout.main, state, color_theme),
            View::Search => search::render(frame, layout.main, state, color_theme, ui),
            View::Sequence => sequence::render(frame, layout.main, state, color_theme),
            View::Normalization => normalization::render(frame, layout.main, state, color_theme),
        }
    }
    render_footer(frame, layout.footer, state, keymap, color_theme);
}

fn render_header(frame: &mut Frame, area: Rect, state: &AppState, color_theme: &ColorTheme) {
    let context = context_for_state(state);
    let location = if state.help().is_open() {
        "Help".to_owned()
    } else {
        header_location(state, context)
    };
    let status = if state.help().is_open() {
        help::context_label(context).to_owned()
    } else {
        header_status(state)
    };
    let status_width = (Line::from(status.as_str()).width() + 1) as u16;
    let left = Rect::new(
        area.x,
        area.y,
        area.width.saturating_sub(status_width + 1),
        area.height,
    );
    let location = compact_location(&location, usize::from(left.width.saturating_sub(9)));

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" sauva", color_theme.accent_style()),
            Span::styled(format!(" / {location}"), color_theme.base_style()),
        ])),
        left,
    );
    frame.render_widget(
        Paragraph::new(Line::styled(
            format!("{status} "),
            Style::new().fg(color_theme.muted),
        ))
        .alignment(Alignment::Right),
        area,
    );
}

fn header_location(state: &AppState, context: Context) -> String {
    match context {
        Context::Inspector => state.sequence().map_or_else(
            || "Inspector".to_owned(),
            |sequence| {
                format!(
                    "{} {}/{} / Inspector",
                    if state.showing_normalization_result() {
                        format!(
                            "Sequence / {} Result",
                            state.normalization().unwrap().form().label()
                        )
                    } else {
                        "Sequence".to_owned()
                    },
                    sequence.selected_index() + 1,
                    sequence.code_points().len()
                )
            },
        ),
        Context::BrowsePlane => "Browse / Planes".to_owned(),
        Context::BrowseRange => "Browse / Ranges".to_owned(),
        Context::BrowseBlock => "Browse / Blocks".to_owned(),
        Context::BrowseCodePoints => "Browse / Code Points".to_owned(),
        Context::NormalizationResult => format!(
            "Sequence / {} Result",
            state.normalization().unwrap().form().label()
        ),
        Context::Normalization => "Sequence / Normalization".to_owned(),
        _ => help::context_label(context).to_owned(),
    }
}

/// Keep the current screen visible without letting a long breadcrumb overlap
/// the right-aligned status. Drop parent segments before shortening the leaf.
fn compact_location(location: &str, width: usize) -> String {
    if Line::from(location).width() <= width {
        return location.to_owned();
    }
    let mut leaf = location;
    while let Some((_, rest)) = leaf.split_once(" / ") {
        leaf = rest;
        let shortened = format!("… / {leaf}");
        if Line::from(shortened.as_str()).width() <= width {
            return shortened;
        }
    }
    let mut suffix = String::new();
    for character in leaf.chars().rev() {
        let candidate = format!("{character}{suffix}");
        if Line::from(candidate.as_str()).width() + 1 > width {
            break;
        }
        suffix = candidate;
    }
    if width == 0 {
        String::new()
    } else {
        format!("…{suffix}")
    }
}

fn header_status(state: &AppState) -> String {
    match state.view() {
        View::Inspector => state.selected().to_string(),
        View::Normalization => state.normalization().unwrap().form().label().to_owned(),
        View::Browser => state
            .browse()
            .map_or_else(String::new, |browse| browse.cursor().to_string()),
        View::Search => {
            let search = state
                .search()
                .expect("the search view always has search state");
            let count = search.outcome().results().len();
            search.selected_result().map_or_else(
                || crate::ui::search::result_count_label(count),
                |result| {
                    format!(
                        "{} · {}",
                        crate::ui::search::result_count_label(count),
                        result.code_point()
                    )
                },
            )
        }
        View::Sequence => {
            let sequence = state
                .sequence()
                .expect("the sequence view always has sequence state");
            format!(
                "{}/{} · {}",
                sequence.selected_index() + 1,
                sequence.code_points().len(),
                sequence.selected()
            )
        }
    }
}

fn render_size_warning(frame: &mut Frame, area: Rect, color_theme: &ColorTheme) {
    let (minimum_width, minimum_height) = MINIMUM_SIZE;
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            " sauva",
            color_theme.accent_style(),
        ))),
        Rect::new(area.x, area.y, area.width, area.height.min(1)),
    );

    let content = Rect::new(
        area.x.saturating_add(1),
        area.y.saturating_add(2),
        area.width.saturating_sub(2),
        area.height.saturating_sub(2),
    );
    let label = Style::new().fg(color_theme.muted);
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled(
                "Terminal too small",
                Style::new()
                    .fg(color_theme.status.warning)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::default(),
            Line::from(vec![
                Span::styled("Required  ", label),
                Span::raw(format!("{minimum_width} × {minimum_height}")),
            ]),
            Line::from(vec![
                Span::styled("Current   ", label),
                Span::raw(format!("{} × {}", area.width, area.height)),
            ]),
        ]),
        content,
    );
}

fn render_footer(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    keymap: &ResolvedKeymap,
    color_theme: &ColorTheme,
) {
    if let Some(status) = state.footer_status() {
        let fg = match status.level() {
            FooterStatusLevel::Info => color_theme.status.info,
            FooterStatusLevel::Warning => color_theme.status.warning,
        };
        frame.render_widget(
            Paragraph::new(format!(" {} ", status.message())).style(Style::new().fg(fg)),
            area,
        );
        return;
    }

    let context = if state.help().is_open() {
        Context::Help
    } else {
        context_for_state(state)
    };
    let content = help::footer(
        context,
        state.sequence().is_some(),
        area.width,
        keymap,
        color_theme,
    );
    frame.render_widget(Paragraph::new(content.left), area);
    frame.render_widget(
        Paragraph::new(content.right).alignment(Alignment::Right),
        area,
    );
}

#[cfg(test)]
mod tests {
    use ratatui::{
        Terminal,
        backend::TestBackend,
        buffer::Buffer,
        style::{Color, Modifier},
    };

    use super::*;
    use crate::{
        app::{Action, update},
        browser::BrowseMove,
        fixtures,
        inspector::InspectorMove,
        ui::inspector,
        ui::layout::{
            MINIMUM_SIZE, STANDARD_SIZE, WIDE_SIZE, browser_list_height, search_result_height,
            sequence_list_height,
        },
        ui::theme::{DifferenceColors, SelectionColors, StatusColors},
    };

    #[test]
    fn result_inspector_breadcrumb_preserves_sequence_parent() {
        let mut state = AppState::with_sequence("A\u{0301}".to_owned());
        update(&mut state, Action::OpenNormalization);
        update(&mut state, Action::InspectNormalizationResult);
        update(&mut state, Action::InspectSequenceCodePoint);
        let header = render_to_text(&state, 60, 16)
            .lines()
            .next()
            .unwrap()
            .to_owned();
        assert!(
            header.contains("sauva / Sequence / NFC Result 1/1 / Inspector"),
            "{header}"
        );
        assert!(header.ends_with("U+00C1"), "{header}");
    }

    #[test]
    fn long_breadcrumbs_keep_the_leaf_and_fit_the_available_width() {
        let location = "Sequence / NFC Result 100000/100000 / Inspector";
        assert_eq!(compact_location(location, 30), "… / Inspector");
        assert_eq!(compact_location("NFC Result", 7), "…Result");
        assert_eq!(compact_location(location, 0), "");
    }

    #[test]
    fn normalization_help_uses_the_screen_name_not_the_breadcrumb() {
        let mut state = AppState::with_sequence("A\u{0301}".to_owned());
        update(&mut state, Action::OpenNormalization);
        update(&mut state, Action::ToggleHelp);
        let text = render_to_text(&state, 100, 30);
        assert!(text.lines().next().unwrap().ends_with("Normalization"));
        assert!(text.contains("Keybindings · Normalization"));
        assert!(!text.contains("Sequence / Normalization"));
    }

    fn render_to_text(state: &AppState, width: u16, height: u16) -> String {
        let buffer = render_to_buffer(state, width, height, &ColorTheme::default());
        buffer_to_text(&buffer)
    }

    fn render_to_buffer(
        state: &AppState,
        width: u16,
        height: u16,
        color_theme: &ColorTheme,
    ) -> Buffer {
        let mut state = state.clone();
        let keymap = ResolvedKeymap::default();
        synchronize_inspector(&mut state, width, height);
        synchronize_help(&mut state, width, height, &keymap);
        update(
            &mut state,
            Action::ResizeBrowserViewport(browser_list_height(Rect::new(0, 0, width, height))),
        );
        update(
            &mut state,
            Action::ResizeSearchViewport(search_result_height(Rect::new(0, 0, width, height))),
        );
        update(
            &mut state,
            Action::ResizeSequenceViewport(sequence_list_height(Rect::new(0, 0, width, height))),
        );
        if state.showing_normalization_result() && state.view() == View::Sequence {
            let (original, result) = crate::ui::normalization_result::viewport_heights(
                Rect::new(0, 0, width, height),
                &state,
            );
            update(
                &mut state,
                Action::ResizeNormalizationOriginalViewport(original),
            );
            update(&mut state, Action::ResizeSequenceViewport(result));
        }
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| render(frame, &state, color_theme, &UiSettings::default(), &keymap))
            .unwrap();

        terminal.backend().buffer().clone()
    }

    fn synchronize_inspector(state: &mut AppState, width: u16, height: u16) {
        let metrics = inspector::viewport_metrics(Rect::new(0, 0, width, height), state);
        update(
            state,
            Action::ResizeInspectorViewport {
                viewport_height: metrics.viewport_height,
                document_height: metrics.document_height,
                field_ranges: metrics.field_ranges,
                groups: metrics.groups,
            },
        );
    }

    fn synchronize_help(state: &mut AppState, width: u16, height: u16, keymap: &ResolvedKeymap) {
        if !state.help().is_open() {
            return;
        }
        let metrics = help::viewport_metrics(Rect::new(0, 0, width, height), state, keymap);
        update(
            state,
            Action::ResizeHelpViewport {
                viewport_height: metrics.viewport_height,
                document_height: metrics.document_height,
            },
        );
    }

    fn render_after_inspector_move(
        state: &AppState,
        width: u16,
        height: u16,
        movement: InspectorMove,
    ) -> String {
        let mut state = state.clone();
        synchronize_inspector(&mut state, width, height);
        update(&mut state, Action::MoveInspector(movement));
        render_to_text(&state, width, height)
    }

    fn buffer_to_text(buffer: &Buffer) -> String {
        let area = buffer.area;
        (area.y..area.bottom())
            .map(|y| {
                let line = (area.x..area.right())
                    .filter_map(|x| buffer.cell((x, y)))
                    .map(|cell| cell.symbol())
                    .collect::<String>();
                line.trim_end().to_owned()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn test_color_theme() -> ColorTheme {
        ColorTheme {
            fg: Color::White,
            bg: Color::Blue,
            muted: Color::Magenta,
            accent: Color::Green,
            heading: Color::Cyan,
            border: Color::DarkGray,
            r#match: Color::LightGreen,
            key: Color::LightRed,
            link: Color::LightBlue,
            selection: SelectionColors {
                fg: Color::White,
                bg: Color::DarkGray,
            },
            difference: DifferenceColors {
                fg: Color::Black,
                bg: Color::Yellow,
            },
            status: StatusColors {
                info: Color::LightGreen,
                warning: Color::LightYellow,
            },
        }
    }

    #[test]
    fn applies_the_color_theme_to_the_screen_and_colored_elements() {
        let state = fixtures::glyph_disabled();
        let color_theme = test_color_theme();
        let (width, height) = STANDARD_SIZE;
        let buffer = render_to_buffer(&state, width, height, &color_theme);
        let unexpected_backgrounds = buffer
            .content
            .iter()
            .enumerate()
            .filter(|(_, cell)| ![color_theme.bg, color_theme.selection.bg].contains(&cell.bg))
            .take(10)
            .map(|(index, cell)| (index, cell.symbol().to_owned(), cell.bg))
            .collect::<Vec<_>>();
        assert!(
            buffer
                .content
                .iter()
                .all(|cell| [color_theme.bg, color_theme.selection.bg].contains(&cell.bg)),
            "unexpected backgrounds: {unexpected_backgrounds:?}"
        );
        assert_eq!(buffer.cell((2, 0)).unwrap().fg, color_theme.accent);
        assert_eq!(buffer.cell((1, 2)).unwrap().fg, color_theme.heading);
        assert_eq!(buffer.cell((3, 4)).unwrap().fg, color_theme.muted);
        assert_eq!(buffer.cell((60, 2)).unwrap().symbol(), "┃");
        assert_eq!(buffer.cell((60, 2)).unwrap().fg, color_theme.border);
        assert!(
            buffer
                .content
                .iter()
                .any(|cell| cell.symbol() == "G" && cell.fg == color_theme.status.warning)
        );
        assert!(
            buffer
                .content
                .iter()
                .any(|cell| cell.fg == color_theme.muted)
        );
        assert!(
            buffer
                .content
                .iter()
                .any(|cell| cell.fg == color_theme.muted)
        );
    }

    #[test]
    fn pane_dividers_leave_the_bottom_padding_empty() {
        let color_theme = test_color_theme();
        for (width, height) in [STANDARD_SIZE, WIDE_SIZE] {
            for state in [
                fixtures::startup(),
                fixtures::browse_planes(),
                fixtures::browse_ranges(),
                fixtures::browse_blocks(),
                fixtures::browse_code_points(),
                fixtures::search_name_results(),
                fixtures::search_empty(),
                fixtures::sequence(),
            ] {
                let buffer = render_to_buffer(&state, width, height, &color_theme);
                let divider_x = if state.browse().is_some_and(|browse| {
                    browse.level() == crate::browser::BrowseLevel::CodePointTable
                }) {
                    60
                } else {
                    width - 40
                };
                assert!(matches!(
                    buffer.cell((divider_x, height - 3)).unwrap().symbol(),
                    "│" | "┃"
                ));
                assert_eq!(buffer.cell((divider_x, height - 2)).unwrap().symbol(), " ");
            }
        }
    }

    #[test]
    fn search_divider_leaves_one_cell_at_the_right_edge() {
        let color_theme = test_color_theme();
        for (width, height) in [MINIMUM_SIZE, STANDARD_SIZE, WIDE_SIZE] {
            let buffer = render_to_buffer(&fixtures::search_empty(), width, height, &color_theme);
            let primary_width = if width >= 100 { width - 40 } else { width };
            for x in 0..primary_width - 1 {
                assert_eq!(buffer.cell((x, 3)).unwrap().symbol(), "─");
            }
            assert_eq!(buffer.cell((primary_width - 1, 3)).unwrap().symbol(), " ");
            if primary_width < width {
                assert_eq!(buffer.cell((primary_width, 3)).unwrap().symbol(), "│");
            }
        }
    }

    #[test]
    fn inspector_selection_keeps_inner_padding_and_leaves_outer_padding_uncolored() {
        let color_theme = test_color_theme();
        let (width, height) = STANDARD_SIZE;
        let buffer = render_to_buffer(&fixtures::startup(), width, height, &color_theme);

        for x in 1..58 {
            let cell = buffer.cell((x, 3)).unwrap();
            assert_eq!(cell.bg, color_theme.selection.bg);
        }
        assert_eq!(buffer.cell((1, 3)).unwrap().fg, color_theme.selection.fg);
        assert_eq!(buffer.cell((1, 3)).unwrap().symbol(), " ");
        assert_eq!(buffer.cell((2, 3)).unwrap().symbol(), "C");
        assert_eq!(buffer.cell((57, 3)).unwrap().symbol(), " ");
        assert_eq!(buffer.cell((1, 4)).unwrap().symbol(), " ");
        assert_eq!(buffer.cell((2, 4)).unwrap().symbol(), "C");
        assert_eq!(buffer.cell((0, 3)).unwrap().bg, color_theme.bg);
        assert_eq!(buffer.cell((58, 3)).unwrap().bg, color_theme.bg);

        let mut state = fixtures::startup();
        synchronize_inspector(&mut state, width, height);
        update(&mut state, Action::MoveInspector(InspectorMove::NextField));
        let moved = render_to_buffer(&state, width, height, &color_theme);
        assert_eq!(moved.cell((1, 3)).unwrap().bg, color_theme.bg);
        assert_eq!(moved.cell((1, 4)).unwrap().bg, color_theme.selection.bg);
    }

    #[test]
    fn returning_to_the_first_property_restores_the_identity_heading() {
        let (width, height) = MINIMUM_SIZE;

        for movement in [InspectorMove::PreviousField, InspectorMove::PageBackward] {
            let mut state = fixtures::details_canonical_decomposition();
            synchronize_inspector(&mut state, width, height);
            update(&mut state, Action::MoveInspector(InspectorMove::Last));
            while state.inspector().selected_index() > 0 {
                update(&mut state, Action::MoveInspector(movement));
            }

            let rendered = render_to_text(&state, width, height);
            assert!(
                rendered
                    .lines()
                    .take(4)
                    .any(|line| line.contains("Identity")),
                "{movement:?}"
            );
        }
    }

    #[test]
    fn footer_status_replaces_short_help_with_the_requested_severity_color() {
        let color_theme = test_color_theme();
        let (width, height) = STANDARD_SIZE;
        let mut state = fixtures::startup();

        for (status, expected_message, expected_color) in [
            (
                crate::app::FooterStatus::info("Copied Code Point"),
                "Copied Code Point",
                color_theme.status.info,
            ),
            (
                crate::app::FooterStatus::warning("No value to copy: Aliases"),
                "No value to copy: Aliases",
                color_theme.status.warning,
            ),
        ] {
            update(&mut state, Action::ShowFooterStatus(status));
            let buffer = render_to_buffer(&state, width, height, &color_theme);
            let footer = (0..width)
                .filter_map(|x| buffer.cell((x, height - 1)))
                .map(|cell| cell.symbol())
                .collect::<String>();

            assert_eq!(footer.trim(), expected_message);
            assert_eq!(buffer.cell((1, height - 1)).unwrap().fg, expected_color);
        }
    }

    #[test]
    fn list_selection_keeps_inner_padding_and_leaves_outer_padding_uncolored() {
        let color_theme = test_color_theme();
        for (width, height) in [MINIMUM_SIZE, STANDARD_SIZE, WIDE_SIZE] {
            let right = if width >= 100 { width - 42 } else { width - 2 };
            for (state, selected_y) in [
                (fixtures::search_name_results(), 5),
                (fixtures::browse_planes(), 3),
                (fixtures::browse_ranges(), 3),
                (fixtures::browse_blocks(), 3),
            ] {
                let buffer = render_to_buffer(&state, width, height, &color_theme);
                for x in 2..right {
                    let cell = buffer.cell((x, selected_y)).unwrap();
                    assert_eq!(cell.bg, color_theme.selection.bg);
                }
                assert_eq!(buffer.cell((2, selected_y)).unwrap().symbol(), " ");
                assert_eq!(buffer.cell((right - 1, selected_y)).unwrap().symbol(), " ");
                assert_ne!(buffer.cell((3, selected_y)).unwrap().symbol(), " ");
                assert_eq!(buffer.cell((2, selected_y + 1)).unwrap().symbol(), " ");
                assert_eq!(
                    buffer.cell((2, selected_y)).unwrap().fg,
                    color_theme.selection.fg
                );
                assert_eq!(buffer.cell((1, selected_y)).unwrap().bg, color_theme.bg);
                assert_eq!(buffer.cell((right, selected_y)).unwrap().bg, color_theme.bg);
            }
        }
    }

    #[test]
    fn search_matches_remain_visible_on_selected_and_unselected_rows() {
        let color_theme = test_color_theme();
        let (width, height) = STANDARD_SIZE;
        let buffer = render_to_buffer(
            &fixtures::search_name_results(),
            width,
            height,
            &color_theme,
        );

        assert!(buffer.content.iter().any(|cell| {
            cell.fg == color_theme.selection.fg && cell.bg == color_theme.selection.bg
        }));
        assert!(
            buffer
                .content
                .iter()
                .any(|cell| { cell.fg == color_theme.r#match && cell.bg == color_theme.bg })
        );
    }

    #[test]
    fn search_empty_states_use_dedicated_message_and_preview_colors() {
        let color_theme = test_color_theme();
        let (width, height) = STANDARD_SIZE;

        let empty = render_to_buffer(&fixtures::search_empty(), width, height, &color_theme);
        assert!(
            empty
                .cell((2, 5))
                .unwrap()
                .modifier
                .contains(Modifier::BOLD)
        );
        assert_eq!(empty.cell((2, 6)).unwrap().fg, color_theme.heading);
        assert_eq!(empty.cell((12, 6)).unwrap().fg, color_theme.muted);
        assert!(
            empty
                .content
                .iter()
                .any(|cell| { cell.symbol() == "N" && cell.fg == color_theme.muted })
        );

        let no_results =
            render_to_buffer(&fixtures::search_no_results(), width, height, &color_theme);
        assert!(
            no_results
                .cell((2, 5))
                .unwrap()
                .modifier
                .contains(Modifier::BOLD)
        );
        assert_eq!(no_results.cell((2, 6)).unwrap().fg, color_theme.muted);
    }

    #[test]
    fn list_scrollbar_does_not_follow_a_cursor_moving_within_the_viewport() {
        let color_theme = ColorTheme::default();
        let (width, height) = STANDARD_SIZE;
        let mut state = fixtures::browse_ranges();
        update(
            &mut state,
            Action::ResizeBrowserViewport(browser_list_height(Rect::new(0, 0, width, height))),
        );
        let before = render_to_buffer(&state, width, height, &color_theme);

        update(&mut state, Action::MoveBrowser(BrowseMove::Down));
        let after = render_to_buffer(&state, width, height, &color_theme);

        let scrollbar_before = (1..28)
            .map(|y| before.cell((58, y)).unwrap().symbol())
            .collect::<String>();
        let scrollbar_after = (1..28)
            .map(|y| after.cell((58, y)).unwrap().symbol())
            .collect::<String>();
        assert_eq!(scrollbar_after, scrollbar_before);
    }

    #[test]
    fn code_point_selection_uses_the_global_selection_fg_and_bg() {
        let color_theme = test_color_theme();
        let (width, height) = STANDARD_SIZE;
        let state = fixtures::browse_code_points();
        let buffer = render_to_buffer(&state, width, height, &color_theme);
        let selected_cells = buffer
            .content
            .iter()
            .filter(|cell| cell.bg == color_theme.selection.bg)
            .collect::<Vec<_>>();

        assert_eq!(selected_cells.len(), 3);
        assert!(
            selected_cells
                .iter()
                .all(|cell| cell.fg == color_theme.selection.fg)
        );
        assert_eq!(
            selected_cells
                .iter()
                .map(|cell| cell.symbol())
                .collect::<String>(),
            " A "
        );
    }

    #[test]
    fn code_point_table_axes_highlight_only_the_selected_row_and_column() {
        let color_theme = test_color_theme();
        let (width, height) = STANDARD_SIZE;
        let buffer = render_to_buffer(&fixtures::browse_code_points(), width, height, &color_theme);

        assert_eq!(buffer.cell((10, 3)).unwrap().fg, color_theme.muted);
        assert_eq!(buffer.cell((13, 3)).unwrap().fg, color_theme.accent);
        assert_eq!(buffer.cell((2, 4)).unwrap().fg, color_theme.muted);
        assert_eq!(buffer.cell((2, 8)).unwrap().fg, color_theme.accent);
        assert!(
            buffer
                .cell((13, 3))
                .unwrap()
                .modifier
                .contains(Modifier::BOLD)
        );
        assert!(
            buffer
                .cell((2, 8))
                .unwrap()
                .modifier
                .contains(Modifier::BOLD)
        );
        assert_eq!(buffer.cell((13, 8)).unwrap().bg, color_theme.selection.bg);
    }

    #[test]
    fn search_and_browse_detail_keys_use_the_key_value_label_color() {
        let color_theme = test_color_theme();
        let (width, height) = STANDARD_SIZE;

        for state in [fixtures::search_name_results(), fixtures::browse_planes()] {
            let buffer = render_to_buffer(&state, width, height, &color_theme);

            assert_eq!(buffer.cell((63, 3)).unwrap().fg, color_theme.muted);
        }
    }

    #[test]
    fn help_uses_dedicated_link_and_key_colors() {
        let color_theme = test_color_theme();
        let (width, height) = STANDARD_SIZE;
        let mut state = fixtures::startup();
        update(&mut state, Action::ToggleHelp);
        let buffer = render_to_buffer(&state, width, height, &color_theme);

        assert_eq!(buffer.cell((2, 7)).unwrap().symbol(), "h");
        assert_eq!(buffer.cell((2, 7)).unwrap().fg, color_theme.link);
        assert_eq!(buffer.cell((1, 8)).unwrap().symbol(), " ");
        assert_eq!(buffer.cell((2, 10)).unwrap().symbol(), "h");
        assert_eq!(buffer.cell((2, 10)).unwrap().fg, color_theme.key);
        assert_eq!(buffer.cell((5, 10)).unwrap().symbol(), "L");
        assert_eq!(buffer.cell((5, 10)).unwrap().fg, color_theme.key);
        assert_eq!(buffer.cell((1, height - 1)).unwrap().fg, color_theme.key);
        assert_eq!(buffer.cell((5, height - 1)).unwrap().fg, color_theme.muted);
    }

    #[test]
    fn startup_standard() {
        let state = fixtures::glyph_basic();
        let (width, height) = STANDARD_SIZE;

        insta::assert_snapshot!(render_to_text(&state, width, height));
    }

    #[test]
    fn startup_minimum() {
        let state = fixtures::startup();
        let (width, height) = MINIMUM_SIZE;

        insta::assert_snapshot!(render_to_text(&state, width, height));
    }

    #[test]
    fn sequence_standard() {
        let state = fixtures::sequence();
        let (width, height) = STANDARD_SIZE;

        insta::assert_snapshot!(render_to_text(&state, width, height));
    }

    #[test]
    fn sequence_minimum() {
        let state = fixtures::sequence();
        let (width, height) = MINIMUM_SIZE;

        insta::assert_snapshot!(render_to_text(&state, width, height));
    }

    #[test]
    fn normalization_standard() {
        let mut state = AppState::with_sequence("A\u{0301} ①👩‍💻".to_owned());
        update(&mut state, Action::OpenNormalization);
        update(
            &mut state,
            Action::MoveNormalization(crate::normalization::NormalizationMove::Next),
        );
        update(
            &mut state,
            Action::MoveNormalization(crate::normalization::NormalizationMove::Next),
        );
        insta::assert_snapshot!(render_to_text(&state, STANDARD_SIZE.0, STANDARD_SIZE.1));
    }

    #[test]
    fn normalization_minimum() {
        let mut state = AppState::with_sequence("A\u{0301} ①👩‍💻".to_owned());
        update(&mut state, Action::OpenNormalization);
        insta::assert_snapshot!(render_to_text(&state, MINIMUM_SIZE.0, MINIMUM_SIZE.1));
    }

    #[test]
    fn normalization_long_minimum() {
        let mut state = AppState::with_sequence(format!(
            "{}A\u{0301} ①{}",
            "ASCII ".repeat(100),
            " tail".repeat(100)
        ));
        update(&mut state, Action::OpenNormalization);
        insta::assert_snapshot!(render_to_text(&state, MINIMUM_SIZE.0, MINIMUM_SIZE.1));
    }

    #[test]
    fn normalization_unchanged_controls_minimum() {
        let mut state = AppState::with_sequence("é\t\r\n\u{1b}\u{202e}\u{fe0f}".to_owned());
        update(&mut state, Action::OpenNormalization);
        insta::assert_snapshot!(render_to_text(&state, MINIMUM_SIZE.0, MINIMUM_SIZE.1));
    }

    #[test]
    fn normalization_keeps_the_selection_and_difference_colors_separate() {
        let mut state = AppState::with_sequence("A\u{0301} ①👩‍💻".to_owned());
        update(&mut state, Action::OpenNormalization);
        let theme = test_color_theme();
        let buffer = render_to_buffer(&state, 60, 16, &theme);
        for (x, y) in [(2, 3), (2, 4), (2, 13), (2, 14)] {
            let cell = buffer.cell((x, y)).unwrap();
            assert_eq!(cell.bg, theme.difference.bg);
            assert_eq!(cell.fg, theme.difference.fg);
            assert!(cell.modifier.contains(Modifier::UNDERLINED));
        }
        for x in 1..58 {
            assert_eq!(buffer.cell((x, 7)).unwrap().bg, theme.selection.bg);
        }
        assert_eq!(buffer.cell((1, 7)).unwrap().symbol(), " ");
        assert_eq!(buffer.cell((57, 7)).unwrap().symbol(), " ");
        assert_eq!(buffer.cell((16, 4)).unwrap().bg, theme.bg);
    }

    fn nfkc_result(source: &str) -> AppState {
        let mut state = AppState::with_sequence(source.to_owned());
        update(&mut state, Action::OpenNormalization);
        for _ in 0..2 {
            update(
                &mut state,
                Action::MoveNormalization(crate::normalization::NormalizationMove::Next),
            );
        }
        update(&mut state, Action::InspectNormalizationResult);
        state
    }

    #[test]
    fn normalization_result_standard() {
        let mut state = nfkc_result("A\u{0301} ﬃ①👩‍💻");
        update(
            &mut state,
            Action::MoveSequence(crate::sequence::SequenceMove::Next),
        );
        update(
            &mut state,
            Action::MoveSequence(crate::sequence::SequenceMove::Next),
        );
        insta::assert_snapshot!(render_to_text(&state, 100, 30));
    }

    #[test]
    fn normalization_result_minimum() {
        let state = nfkc_result("ﬃ①");
        insta::assert_snapshot!(render_to_text(&state, 60, 16));
    }

    #[test]
    fn normalization_result_decomposed_minimum() {
        let mut state = AppState::with_sequence("éé".to_owned());
        update(&mut state, Action::OpenNormalization);
        update(
            &mut state,
            Action::MoveNormalization(crate::normalization::NormalizationMove::Next),
        );
        update(&mut state, Action::InspectNormalizationResult);
        update(
            &mut state,
            Action::MoveSequence(crate::sequence::SequenceMove::Next),
        );
        insta::assert_snapshot!(render_to_text(&state, 60, 16));
    }

    #[test]
    fn original_reference_marks_only_the_range_corresponding_to_the_result_selection() {
        let mut state = nfkc_result("ﬃ①");
        let theme = test_color_theme();
        for index in 0..4 {
            let buffer = render_to_buffer(&state, 60, 16, &theme);
            let original_y = if index < 3 { 3 } else { 4 };
            assert_eq!(buffer.cell((6, original_y)).unwrap().bg, theme.selection.bg);
            assert_eq!(
                buffer
                    .cell((6, if original_y == 3 { 4 } else { 3 }))
                    .unwrap()
                    .bg,
                theme.bg
            );
            let text = (2..22)
                .map(|x| buffer.cell((x, original_y)).unwrap().symbol())
                .collect::<String>();
            assert!(text.contains(if index < 3 { "U+FB03" } else { "U+2460" }));
            update(
                &mut state,
                Action::MoveSequence(crate::sequence::SequenceMove::Next),
            );
        }
    }

    #[test]
    fn sequence_wide() {
        let state = fixtures::sequence();
        let (width, height) = WIDE_SIZE;

        insta::assert_snapshot!(render_to_text(&state, width, height));
    }

    #[test]
    fn sequence_scrolled_minimum() {
        let mut state = AppState::with_sequence(format!("A{}B", "\u{0301}".repeat(14)));
        update(
            &mut state,
            Action::MoveSequence(crate::sequence::SequenceMove::Last),
        );
        update(
            &mut state,
            Action::MoveSequence(crate::sequence::SequenceMove::Previous),
        );
        let (width, height) = MINIMUM_SIZE;
        insta::assert_snapshot!(render_to_text(&state, width, height));
    }

    #[test]
    fn sequence_selected_combining_standard() {
        let mut state = fixtures::sequence();
        update(
            &mut state,
            Action::MoveSequence(crate::sequence::SequenceMove::Next),
        );
        let (width, height) = STANDARD_SIZE;
        insta::assert_snapshot!(render_to_text(&state, width, height));
    }

    #[test]
    fn sequence_wrapped_coordinate_standard() {
        let mut state =
            AppState::with_sequence(format!("{}A{}", "X".repeat(10), "\u{0301}".repeat(11)));
        update(
            &mut state,
            Action::MoveSequence(crate::sequence::SequenceMove::Last),
        );
        let (width, height) = STANDARD_SIZE;

        insta::assert_snapshot!(render_to_text(&state, width, height));
    }

    #[test]
    fn glyph_wide() {
        let state = fixtures::glyph_wide();
        let (width, height) = WIDE_SIZE;

        insta::assert_snapshot!(render_to_text(&state, width, height));
    }

    #[test]
    fn glyph_combining_standard() {
        let state = fixtures::glyph_combining();
        let (width, height) = STANDARD_SIZE;

        insta::assert_snapshot!(render_to_text(&state, width, height));
    }

    #[test]
    fn glyph_missing_standard() {
        let state = fixtures::glyph_missing();
        let (width, height) = STANDARD_SIZE;

        insta::assert_snapshot!(render_to_text(&state, width, height));
    }

    #[test]
    fn inspector_minimum_width_wraps_values_in_the_value_column() {
        let state = fixtures::inspector_plane_16_end();

        insta::assert_snapshot!(render_to_text(&state, MINIMUM_SIZE.0, MINIMUM_SIZE.1 + 1));
    }

    #[test]
    fn terminal_too_small() {
        let state = fixtures::startup();

        insta::assert_snapshot!(render_to_text(&state, 59, 15));
    }

    #[test]
    fn details_aliases_standard() {
        let state = fixtures::details_aliases();
        let (width, height) = STANDARD_SIZE;

        insta::assert_snapshot!(render_to_text(&state, width, height));
    }

    #[test]
    fn details_canonical_decomposition_standard_at_normalization() {
        let state = fixtures::details_canonical_decomposition();
        let (width, height) = STANDARD_SIZE;

        insta::assert_snapshot!(render_after_inspector_move(
            &state,
            width,
            height,
            InspectorMove::Last,
        ));
    }

    #[test]
    fn details_compatibility_decomposition_wide() {
        let state = fixtures::details_compatibility_decomposition();
        let (width, height) = WIDE_SIZE;

        insta::assert_snapshot!(render_to_text(&state, width, height));
    }

    #[test]
    fn details_canonical_decomposition_minimum_at_middle() {
        let state = fixtures::details_canonical_decomposition();
        let (width, height) = MINIMUM_SIZE;

        insta::assert_snapshot!(render_after_inspector_move(
            &state,
            width,
            height,
            InspectorMove::PageForward,
        ));
    }

    #[test]
    fn details_canonical_decomposition_minimum_at_end() {
        let state = fixtures::details_scrolled_minimum();
        let (width, height) = MINIMUM_SIZE;

        insta::assert_snapshot!(render_to_text(&state, width, height));
    }

    #[test]
    fn browse_planes_standard() {
        let state = fixtures::browse_planes();
        let (width, height) = STANDARD_SIZE;

        insta::assert_snapshot!(render_to_text(&state, width, height));
    }

    #[test]
    fn browse_planes_minimum_at_last_plane() {
        let state = fixtures::browse_planes_last();
        let (width, height) = MINIMUM_SIZE;

        insta::assert_snapshot!(render_to_text(&state, width, height));
    }

    #[test]
    fn browse_planes_wide_at_middle_plane() {
        let state = fixtures::browse_planes_middle();
        let (width, height) = WIDE_SIZE;

        insta::assert_snapshot!(render_to_text(&state, width, height));
    }

    #[test]
    fn browse_ranges_standard() {
        let state = fixtures::browse_ranges();
        let (width, height) = STANDARD_SIZE;

        insta::assert_snapshot!(render_to_text(&state, width, height));
    }

    #[test]
    fn browse_ranges_minimum_at_middle_range() {
        let state = fixtures::browse_ranges_middle();
        let (width, height) = MINIMUM_SIZE;

        insta::assert_snapshot!(render_to_text(&state, width, height));
    }

    #[test]
    fn browse_ranges_wide_at_last_range() {
        let state = fixtures::browse_ranges_last();
        let (width, height) = WIDE_SIZE;

        insta::assert_snapshot!(render_to_text(&state, width, height));
    }

    #[test]
    fn browse_ranges_minimum_after_reversing_at_the_bottom_edge() {
        let state = fixtures::browse_ranges_after_reverse();
        let (width, height) = MINIMUM_SIZE;

        insta::assert_snapshot!(render_to_text(&state, width, height));
    }

    #[test]
    fn browse_blocks_standard() {
        let state = fixtures::browse_blocks();
        let (width, height) = STANDARD_SIZE;

        insta::assert_snapshot!(render_to_text(&state, width, height));
    }

    #[test]
    fn browse_blocks_from_gap_minimum() {
        let state = fixtures::browse_blocks_from_gap();
        let (width, height) = MINIMUM_SIZE;

        insta::assert_snapshot!(render_to_text(&state, width, height));
    }

    #[test]
    fn browse_block_code_points_short_minimum() {
        let state = fixtures::browse_block_code_points_short();
        let (width, height) = MINIMUM_SIZE;

        insta::assert_snapshot!(render_to_text(&state, width, height));
    }

    #[test]
    fn browse_block_code_points_long_end_wide() {
        let state = fixtures::browse_block_code_points_long_end();
        let (width, height) = WIDE_SIZE;

        insta::assert_snapshot!(render_to_text(&state, width, height));
    }

    #[test]
    fn browse_code_points_standard() {
        let state = fixtures::browse_code_points();
        let (width, height) = STANDARD_SIZE;

        insta::assert_snapshot!(render_to_text(&state, width, height));
    }

    #[test]
    fn browse_code_points_minimum_at_last_row() {
        let state = fixtures::browse_code_points_minimum();
        let (width, height) = MINIMUM_SIZE;

        insta::assert_snapshot!(render_to_text(&state, width, height));
    }

    #[test]
    fn browse_code_points_special_values() {
        let state = fixtures::browse_special();
        let (width, height) = STANDARD_SIZE;

        insta::assert_snapshot!(render_to_text(&state, width, height));
    }

    #[test]
    fn browse_code_points_wide_at_plane_16_end() {
        let state = fixtures::browse_plane_16_end();
        let (width, height) = WIDE_SIZE;

        insta::assert_snapshot!(render_to_text(&state, width, height));
    }

    #[test]
    fn search_empty_standard() {
        let state = fixtures::search_empty();
        let (width, height) = STANDARD_SIZE;

        insta::assert_snapshot!(render_to_text(&state, width, height));
    }

    #[test]
    fn search_name_results_standard() {
        let state = fixtures::search_name_results();
        let (width, height) = STANDARD_SIZE;

        insta::assert_snapshot!(render_to_text(&state, width, height));
    }

    #[test]
    fn search_alias_results_standard() {
        let state = fixtures::search_alias_results();
        let (width, height) = STANDARD_SIZE;

        insta::assert_snapshot!(render_to_text(&state, width, height));
    }

    #[test]
    fn search_alias_results_wide() {
        let state = fixtures::search_alias_results();
        let (width, height) = WIDE_SIZE;

        insta::assert_snapshot!(render_to_text(&state, width, height));
    }

    #[test]
    fn search_name_results_minimum_with_scrolled_viewport() {
        let state = fixtures::search_name_results_minimum();
        let (width, height) = MINIMUM_SIZE;

        insta::assert_snapshot!(render_to_text(&state, width, height));
    }

    #[test]
    fn search_combined_results_standard() {
        let state = fixtures::search_combined_results();
        let (width, height) = STANDARD_SIZE;

        insta::assert_snapshot!(render_to_text(&state, width, height));
    }

    #[test]
    fn search_no_results_standard() {
        let state = fixtures::search_no_results();
        let (width, height) = STANDARD_SIZE;

        insta::assert_snapshot!(render_to_text(&state, width, height));
    }

    #[test]
    fn search_invalid_notation_standard() {
        let state = fixtures::search_invalid_notation();
        let (width, height) = STANDARD_SIZE;

        insta::assert_snapshot!(render_to_text(&state, width, height));
    }

    #[test]
    fn search_special_wide() {
        let state = fixtures::search_special();
        let (width, height) = WIDE_SIZE;

        insta::assert_snapshot!(render_to_text(&state, width, height));
    }

    #[test]
    fn inspector_help_standard() {
        let mut state = fixtures::startup();
        update(&mut state, Action::ToggleHelp);
        let (width, height) = STANDARD_SIZE;

        insta::assert_snapshot!(render_to_text(&state, width, height));
    }

    #[test]
    fn inspector_help_minimum_scrolled_to_end() {
        let mut state = fixtures::startup();
        let (width, height) = MINIMUM_SIZE;
        let keymap = ResolvedKeymap::default();
        update(&mut state, Action::ToggleHelp);
        synchronize_help(&mut state, width, height, &keymap);
        update(&mut state, Action::MoveHelp(crate::help::HelpMove::Last));

        insta::assert_snapshot!(render_to_text(&state, width, height));
    }

    #[test]
    fn help_hides_the_glyph_preview_request() {
        let mut state = fixtures::glyph_basic();
        update(&mut state, Action::ToggleHelp);

        assert_eq!(
            crate::ui::glyph_preview_request(Rect::new(0, 0, 100, 30), &state),
            None
        );
    }

    #[test]
    fn search_places_the_terminal_cursor_after_the_input() {
        let state = fixtures::search_name_results();
        let (width, height) = STANDARD_SIZE;
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                render(
                    frame,
                    &state,
                    &ColorTheme::default(),
                    &UiSettings::default(),
                    &ResolvedKeymap::default(),
                )
            })
            .unwrap();

        terminal.backend_mut().assert_cursor_position((25, 2));
    }

    #[test]
    fn text_input_cursor_hides_the_terminal_cursor_in_search_and_help() {
        let mut state = fixtures::search_name_results();
        let (width, height) = STANDARD_SIZE;
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        let ui = UiSettings {
            input_cursor: crate::ui::settings::InputCursor::Text("|".to_owned()),
        };

        terminal
            .draw(|frame| {
                render(
                    frame,
                    &state,
                    &ColorTheme::default(),
                    &ui,
                    &ResolvedKeymap::default(),
                )
            })
            .unwrap();
        assert!(!terminal.backend().cursor_visible());

        update(&mut state, Action::ToggleHelp);
        terminal
            .draw(|frame| {
                render(
                    frame,
                    &state,
                    &ColorTheme::default(),
                    &ui,
                    &ResolvedKeymap::default(),
                )
            })
            .unwrap();
        assert!(!terminal.backend().cursor_visible());
    }
}
