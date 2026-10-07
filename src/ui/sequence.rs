use std::ops::Range;

use ratatui::{Frame, layout::Rect, style::Style, text::Line, widgets::Paragraph};

use crate::{
    app::AppState,
    sequence::SequenceState,
    ui::{
        layout,
        scrollbar::{self, ViewportScrollbar},
        selectable_list_line, selection_preview,
        theme::ColorTheme,
        workspace,
    },
    unicode::{UnicodeDatabase, text::TextAnalysis},
};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, color_theme: &ColorTheme) {
    if state.showing_normalization_result() {
        super::normalization_result::render(frame, area, state, color_theme);
        return;
    }
    let layout = layout::sequence(area);
    let sequence = state
        .sequence()
        .expect("the sequence view always has sequence state");
    if let Some(context) = layout.context {
        selection_preview::render_sequence(
            frame,
            context,
            sequence,
            state.glyph_preview(),
            color_theme,
        );
    }
    render_list(
        frame,
        layout.navigator,
        sequence,
        "Code Points",
        color_theme,
    );
}

pub(super) fn render_list(
    frame: &mut Frame,
    area: Rect,
    sequence: &SequenceState,
    title: &str,
    color_theme: &ColorTheme,
) {
    let count = sequence.code_points().len();
    let grapheme_count = sequence.analysis().graphemes().len();
    let grapheme_unit = if grapheme_count == 1 {
        "grapheme"
    } else {
        "graphemes"
    };
    let code_point_unit = if count == 1 {
        "code point"
    } else {
        "code points"
    };
    let count_label = format!("{count} {code_point_unit} · {grapheme_count} {grapheme_unit}");
    let count_label = if title.len() + 1 + Line::from(count_label.as_str()).width()
        > usize::from(area.width.saturating_sub(3))
    {
        format!("{count} CP · {grapheme_count} GC")
    } else {
        count_label
    };
    let content =
        workspace::render_primary_heading(frame, area, title, Some(&count_label), color_theme);
    let position_width = count.to_string().len();
    let grapheme_width = grapheme_count.to_string().len();
    let gutter_width = (grapheme_width as u16 + 3).min(content.width);
    let gutter = Rect::new(content.x, content.y, gutter_width, content.height);
    let body = Rect::new(
        gutter.right(),
        content.y,
        content.width.saturating_sub(gutter_width),
        content.height,
    );
    let visible = sequence.visible_range();
    let selected_grapheme = sequence.code_points()[sequence.selected_index()].grapheme_index();
    let (gutter_rows, rows): (Vec<_>, Vec<_>) = visible
        .clone()
        .map(|index| {
            let point = &sequence.code_points()[index];
            let gutter_row = grapheme_gutter(
                sequence.analysis(),
                index,
                visible.start,
                &(selected_grapheme..selected_grapheme + 1),
                grapheme_width,
                color_theme,
            );
            let code_point = point.code_point();
            let selected = sequence.selected_index() == index;
            let representation = UnicodeDatabase::display_representation(code_point);
            let name = UnicodeDatabase::primary_name_or_fallback(code_point);
            let code_point = format!("{code_point:<8}", code_point = code_point.to_string());
            let row = selectable_list_line(
                Line::from(format!(
                    "{:>position_width$}  {code_point}  {representation} — {name}",
                    index + 1
                )),
                selected,
                body.width,
                color_theme.selection,
            );
            (gutter_row, row)
        })
        .unzip();

    frame.render_widget(Paragraph::new(gutter_rows), gutter);
    frame.render_widget(Paragraph::new(rows), body);
    frame.render_widget(
        ViewportScrollbar::new(count, visible).style(color_theme.border_style()),
        scrollbar::area_for_primary(frame.area(), area, content),
    );
}

pub(super) fn grapheme_gutter(
    analysis: &TextAnalysis,
    index: usize,
    viewport_start: usize,
    marked: &Range<usize>,
    width: usize,
    theme: &ColorTheme,
) -> Line<'static> {
    let point = &analysis.code_points()[index];
    let range = analysis.graphemes()[point.grapheme_index()].code_point_range();
    let boundary = match (index == range.start, index + 1 == range.end) {
        (true, true) => "•",
        (true, false) => "┌",
        (false, true) => "└",
        (false, false) => "│",
    };
    // Repeat the cluster number at the viewport's top without inventing a
    // cluster boundary when its beginning has scrolled offscreen.
    let label = if index == range.start || index == viewport_start {
        (point.grapheme_index() + 1).to_string()
    } else {
        String::new()
    };
    Line::styled(
        format!("{label:>width$} {boundary} "),
        if marked.contains(&point.grapheme_index()) {
            theme.accent_style()
        } else {
            Style::new().fg(theme.muted)
        },
    )
}

#[cfg(test)]
mod tests {
    use ratatui::{
        Terminal,
        backend::TestBackend,
        buffer::Buffer,
        style::{Color, Modifier},
        widgets::Block,
    };
    use rstest::rstest;

    use super::*;
    use crate::{
        app::{Action, AppState, update},
        sequence::SequenceMove,
        ui::theme::SelectionColors,
    };

    fn render_to_buffer(
        state: &AppState,
        width: u16,
        height: u16,
        color_theme: &ColorTheme,
    ) -> Buffer {
        let mut state = state.clone();
        let area = Rect::new(0, 0, width, height);
        let content = workspace::primary_section(layout::sequence(area).navigator).content;
        update(
            &mut state,
            Action::ResizeSequenceViewport(usize::from(content.height)),
        );
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal
            .draw(|frame| {
                frame.render_widget(Block::default().style(color_theme.base_style()), area);
                render(frame, area, &state, color_theme);
            })
            .unwrap();
        terminal.backend().buffer().clone()
    }

    fn row_text(buffer: &Buffer, x: u16, y: u16, width: u16) -> String {
        (x..x + width)
            .map(|x| buffer.cell((x, y)).unwrap().symbol())
            .collect()
    }

    #[test]
    fn renders_positions_code_points_and_names_in_input_order() {
        let mut state = AppState::with_sequence("A→A".to_owned());
        update(&mut state, Action::ResizeSequenceViewport(27));
        let backend = TestBackend::new(100, 29);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                render(frame, frame.area(), &state, &ColorTheme::default());
            })
            .unwrap();
        let text =
            terminal
                .backend()
                .buffer()
                .content()
                .iter()
                .fold(String::new(), |mut text, cell| {
                    text.push_str(cell.symbol());
                    text
                });

        assert!(text.contains("Code Points"));
        assert!(text.contains("3 code points"));
        assert!(text.contains("1  U+0041"));
        assert!(text.contains("2  U+2192"));
        assert_eq!(text.matches("LATIN CAPITAL LETTER A").count(), 3);
    }

    #[test]
    fn shows_cluster_boundaries_separately_from_code_point_positions() {
        let state = AppState::with_sequence("A\u{0301} 👩‍💻".to_owned());
        let buffer = render_to_buffer(&state, 60, 16, &ColorTheme::default());

        let heading = row_text(&buffer, 0, 0, 60);
        assert!(heading.contains("6 code points · 3 graphemes"));
        for (row, prefix) in [
            "1 ┌  1  U+0041",
            "  └  2  U+0301",
            "2 •  3  U+0020",
            "3 ┌  4  U+1F469",
            "  │  5  U+200D",
            "  └  6  U+1F4BB",
        ]
        .into_iter()
        .enumerate()
        {
            assert!(row_text(&buffer, 2, row as u16 + 1, 56).starts_with(prefix));
        }
    }

    #[rstest]
    fn highlights_only_the_body_background(
        #[values(60, 100)] width: u16,
        #[values(0, 1, 2, 3, 4, 5)] selected_index: usize,
    ) {
        let mut state = AppState::with_sequence("A\u{0301} 👩‍💻".to_owned());
        for _ in 0..selected_index {
            update(&mut state, Action::MoveSequence(SequenceMove::Next));
        }
        let theme = ColorTheme {
            fg: Color::White,
            bg: Color::Blue,
            muted: Color::Yellow,
            accent: Color::Magenta,
            selection: SelectionColors {
                fg: Color::Black,
                bg: Color::Green,
            },
            ..Default::default()
        };
        let buffer = render_to_buffer(&state, width, 16, &theme);
        let grapheme_indices = [0, 0, 1, 2, 2, 2];
        for (row, grapheme_index) in grapheme_indices.into_iter().enumerate() {
            let y = row as u16 + 1;
            let selected = row == selected_index;
            let grapheme_selected = grapheme_index == grapheme_indices[selected_index];
            for x in 2..6 {
                let cell = buffer.cell((x, y)).unwrap();
                assert_eq!(
                    cell.fg,
                    if grapheme_selected {
                        theme.accent
                    } else {
                        theme.muted
                    }
                );
                assert_eq!(cell.modifier.contains(Modifier::BOLD), grapheme_selected);
                assert_eq!(cell.bg, theme.bg);
            }
            assert_eq!(buffer.cell((6, y)).unwrap().symbol(), " ");
            assert_eq!(buffer.cell((7, y)).unwrap().symbol(), (row + 1).to_string());
            assert_eq!(buffer.cell((57, y)).unwrap().symbol(), " ");
            let mut x = 6;
            while x < 58 {
                let cell = buffer.cell((x, y)).unwrap();
                assert_eq!(
                    cell.bg,
                    if selected {
                        theme.selection.bg
                    } else {
                        theme.bg
                    }
                );
                assert_eq!(
                    cell.fg,
                    if selected {
                        theme.selection.fg
                    } else {
                        theme.fg
                    }
                );
                assert!(!cell.modifier.contains(Modifier::BOLD));
                // Ratatui resets cells hidden by wide glyphs; they are not
                // independently drawn by the terminal.
                x += Line::from(cell.symbol()).width().max(1) as u16;
            }
        }
    }

    #[test]
    fn keeps_the_column_width_stable_for_multi_digit_cluster_numbers() {
        let state = AppState::with_sequence("A".repeat(11));
        let buffer = render_to_buffer(&state, 60, 16, &ColorTheme::default());

        assert_eq!(row_text(&buffer, 2, 1, 5), " 1 • ");
        assert_eq!(row_text(&buffer, 2, 10, 5), "10 • ");
        assert_eq!(row_text(&buffer, 2, 11, 5), "11 • ");
        for (row, position) in (1..=11).enumerate() {
            assert_eq!(
                row_text(&buffer, 8, row as u16 + 1, 2),
                format!("{position:>2}")
            );
        }
    }

    #[test]
    fn a_single_grapheme_still_lists_each_code_point() {
        let state = AppState::with_sequence("🇯🇵".to_owned());
        let buffer = render_to_buffer(&state, 60, 16, &ColorTheme::default());

        assert!(row_text(&buffer, 0, 0, 60).contains("2 code points · 1 grapheme"));
        assert_eq!(row_text(&buffer, 2, 1, 4), "1 ┌ ");
        assert_eq!(row_text(&buffer, 2, 2, 4), "  └ ");
        assert!(row_text(&buffer, 6, 1, 52).contains("U+1F1EF"));
        assert!(row_text(&buffer, 6, 2, 52).contains("U+1F1F5"));
    }

    #[test]
    fn scrolling_does_not_invent_cluster_boundaries() {
        let mut state = AppState::with_sequence("A\u{0301}\u{0300}\u{0323}Z".to_owned());
        for (movement, expected, highlighted) in [
            (SequenceMove::First, ["1 ┌ ", "  │ "], [true, true]),
            (SequenceMove::Next, ["1 ┌ ", "  │ "], [true, true]),
            (SequenceMove::Next, ["1 │ ", "  │ "], [true, true]),
            (SequenceMove::Next, ["1 │ ", "  └ "], [true, true]),
            (SequenceMove::Next, ["1 └ ", "2 • "], [false, true]),
        ] {
            update(&mut state, Action::MoveSequence(movement));
            let buffer = render_to_buffer(&state, 60, 4, &ColorTheme::default());
            assert_eq!(row_text(&buffer, 2, 1, 4), expected[0]);
            assert_eq!(row_text(&buffer, 2, 2, 4), expected[1]);
            let theme = ColorTheme::default();
            for (row, highlighted) in highlighted.into_iter().enumerate() {
                for x in 2..6 {
                    let cell = buffer.cell((x, row as u16 + 1)).unwrap();
                    assert_eq!(
                        cell.fg,
                        if highlighted {
                            theme.accent
                        } else {
                            theme.muted
                        }
                    );
                    assert_eq!(cell.modifier.contains(Modifier::BOLD), highlighted);
                    assert_eq!(cell.bg, theme.bg);
                }
            }
        }
    }

    #[rstest]
    fn tiny_areas_do_not_panic(
        #[values(0, 1, 2, 3, 4, 8)] width: u16,
        #[values(0, 1, 2, 3)] height: u16,
    ) {
        render_to_buffer(
            &AppState::with_sequence("A\u{0301}".to_owned()),
            width,
            height,
            &ColorTheme::default(),
        );
    }
}
