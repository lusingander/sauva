use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::{
    app::AppState,
    search::{SearchDirectMatchKind, SearchOutcome, SearchResult},
    ui::{
        layout,
        scrollbar::{self, ViewportScrollbar},
        selectable_list_line, selection_preview,
        settings::{InputCursor, UiSettings},
        theme::ColorTheme,
        workspace,
    },
    unicode::UnicodeDatabase,
};

const CHARACTER_COLUMN_WIDTH: usize = 2;
const CODE_POINT_COLUMN_WIDTH: usize = 8;
const SEARCH_PREFIX: &str = "Search  ";
const SEARCH_PREFIX_MINIMUM_WIDTH: u16 = 24;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    color_theme: &ColorTheme,
    ui: &UiSettings,
) {
    let layout = layout::search(area);
    let search = state
        .search()
        .expect("the search view always has search state");

    if let Some(preview) = layout.preview {
        match search.selected_result() {
            Some(result) => {
                selection_preview::render(
                    frame,
                    preview,
                    result.code_point(),
                    state.glyph_preview(),
                    color_theme,
                );
            }
            None => render_empty_preview(frame, preview, color_theme),
        }
    }
    render_input(
        frame,
        layout.input,
        search.input(),
        &ui.input_cursor,
        color_theme,
    );
    render_results(
        frame,
        layout.results,
        search.outcome(),
        search.selected_index(),
        search.visible_result_range(),
        color_theme,
        ui,
    );
}

fn render_input(
    frame: &mut Frame,
    area: Rect,
    input: &tui_input::Input,
    cursor: &InputCursor,
    color_theme: &ColorTheme,
) {
    let content = workspace::primary_canvas(area);
    let prefix_text = if content.width >= SEARCH_PREFIX_MINIMUM_WIDTH {
        SEARCH_PREFIX
    } else {
        ""
    };
    let prefix_width = Line::from(prefix_text).width();
    let width = usize::from(content.width)
        .saturating_sub(prefix_width)
        .max(1);
    let scroll = match cursor {
        InputCursor::Native => input.visual_scroll(width),
        InputCursor::Text(_) => input.visual_scroll(width.saturating_sub(1).max(1)),
    };
    let input_content = match cursor {
        InputCursor::Native => Line::raw(input.value()),
        InputCursor::Text(text) => {
            let byte_index = input
                .value()
                .char_indices()
                .nth(input.cursor())
                .map_or(input.value().len(), |(index, _)| index);
            let (before, after) = input.value().split_at(byte_index);
            Line::from(vec![Span::raw(before), Span::raw(text), Span::raw(after)])
        }
    };
    let prefix_width = u16::try_from(prefix_width).unwrap_or(content.width);
    let prefix_area = Rect::new(content.x, content.y, prefix_width.min(content.width), 1);
    let input_area = Rect::new(
        prefix_area.right(),
        content.y,
        content.width.saturating_sub(prefix_area.width),
        1,
    );
    frame.render_widget(
        Paragraph::new(prefix_text).style(Style::new().fg(color_theme.muted)),
        prefix_area,
    );
    frame.render_widget(
        Paragraph::new(input_content).scroll((0, scroll as u16)),
        input_area,
    );
    workspace::render_divider(
        frame,
        Rect::new(area.x, area.bottom().saturating_sub(1), area.width, 1),
        color_theme,
    );

    if matches!(cursor, InputCursor::Native) {
        let cursor = input.visual_cursor().max(scroll) - scroll;
        frame.set_cursor_position((input_area.x + cursor as u16, input_area.y));
    }
}

fn render_results(
    frame: &mut Frame,
    area: Rect,
    outcome: &SearchOutcome,
    selected_index: Option<usize>,
    visible_results: std::ops::Range<usize>,
    color_theme: &ColorTheme,
    ui: &UiSettings,
) {
    let results = outcome.results();
    let count = if matches!(outcome, SearchOutcome::Results { .. }) {
        Some(result_count_label(results.len()))
    } else {
        None
    };
    let content =
        workspace::render_primary_heading(frame, area, "Results", count.as_deref(), color_theme);
    let lines = match outcome {
        SearchOutcome::Empty => search_prompt_lines(color_theme),
        SearchOutcome::InvalidNotation(error) => vec![Line::from(format!("Error: {error}"))],
        SearchOutcome::Results { .. } if results.is_empty() => no_results_lines(color_theme),
        SearchOutcome::Results { .. } => visible_results
            .clone()
            .map(|index| {
                let selected = selected_index == Some(index);
                let line = result_line(
                    results[index],
                    outcome.name_query(),
                    selected,
                    color_theme,
                    ui,
                );
                selectable_list_line(line, selected, content.width, color_theme.selection)
            })
            .collect(),
    };

    frame.render_widget(Paragraph::new(lines), content);
    frame.render_widget(
        ViewportScrollbar::new(results.len(), visible_results).style(color_theme.border_style()),
        scrollbar::area_for_primary(frame.area(), area, content),
    );
}

pub fn result_count_label(count: usize) -> String {
    let noun = if count == 1 { "result" } else { "results" };
    format!("{count} {noun}")
}

fn search_prompt_lines(color_theme: &ColorTheme) -> Vec<Line<'static>> {
    vec![
        Line::styled(
            "Search by Unicode name, code point, or character",
            Style::new().add_modifier(Modifier::BOLD),
        ),
        Line::from(vec![
            Span::styled("Examples", color_theme.heading_style()),
            Span::styled(
                "  rightwards arrow · U+2192 · →",
                Style::new().fg(color_theme.muted),
            ),
        ]),
    ]
}

fn no_results_lines(color_theme: &ColorTheme) -> Vec<Line<'static>> {
    vec![
        Line::styled(
            "No matching characters",
            Style::new().add_modifier(Modifier::BOLD),
        ),
        Line::styled(
            "Try another name, code point, or character",
            Style::new().fg(color_theme.muted),
        ),
    ]
}

fn result_line(
    result: SearchResult,
    name_query: Option<&str>,
    selected: bool,
    color_theme: &ColorTheme,
    ui: &UiSettings,
) -> Line<'static> {
    let code_point = result.code_point();
    let representation = UnicodeDatabase::display_representation(code_point);
    let representation = representation.as_str();
    let representation_width = Line::from(representation).width();
    let literal_match = result.direct_match() == Some(SearchDirectMatchKind::LiteralCharacter);
    let code_point_match = result.direct_match() == Some(SearchDirectMatchKind::CodePointNotation);
    let match_style = color_theme.match_style(selected);
    let mut spans = vec![Span::raw(format!("{} ", ui.selection_marker(selected)))];
    if matches!(representation_width, 1 | 2) {
        spans.push(match_span(
            representation.to_owned(),
            literal_match,
            match_style,
        ));
        spans.push(Span::raw(
            " ".repeat(CHARACTER_COLUMN_WIDTH - representation_width),
        ));
    } else {
        spans.push(match_span("·".to_owned(), literal_match, match_style));
        spans.push(Span::raw(" "));
    }
    spans.push(Span::raw("  "));

    let code_point_label = code_point.to_string();
    let code_point_padding = CODE_POINT_COLUMN_WIDTH - code_point_label.len();
    spans.push(match_span(code_point_label, code_point_match, match_style));
    spans.push(Span::raw(" ".repeat(code_point_padding)));
    spans.push(Span::raw("  "));

    if !matches!(representation_width, 1 | 2) {
        spans.push(match_span(
            representation.to_owned(),
            literal_match,
            match_style,
        ));
        spans.push(Span::raw(" — "));
    }
    if let Some(alias_match) = result.preferred_alias_match() {
        let alias = alias_match.alias();
        spans.extend(name_spans(alias.name(), name_query, match_style));
        spans.push(Span::styled(
            format!(" · {} alias", alias.kind().label()),
            if selected {
                Style::new()
            } else {
                Style::new().fg(color_theme.muted)
            },
        ));
    } else {
        spans.extend(name_spans(
            UnicodeDatabase::primary_name_or_fallback(code_point),
            result.name_match().and(name_query),
            match_style,
        ));
    }

    Line::from(spans)
}

fn match_span(text: String, matched: bool, style: Style) -> Span<'static> {
    if matched {
        Span::styled(text, style)
    } else {
        Span::raw(text)
    }
}

fn name_spans(name: &str, query: Option<&str>, style: Style) -> Vec<Span<'static>> {
    let Some(query) = query else {
        return vec![Span::raw(name.to_owned())];
    };

    let mut spans = Vec::new();
    let mut previous_end = 0;
    for (start, matched) in name.match_indices(query) {
        if previous_end < start {
            spans.push(Span::raw(name[previous_end..start].to_owned()));
        }
        spans.push(Span::styled(matched.to_owned(), style));
        previous_end = start + matched.len();
    }
    if previous_end < name.len() {
        spans.push(Span::raw(name[previous_end..].to_owned()));
    }
    spans
}

fn render_empty_preview(frame: &mut Frame, area: Rect, color_theme: &ColorTheme) {
    let content = workspace::render_rail_heading(frame, area, "Selection", None, color_theme);

    let message = Rect::new(
        content.x,
        content.y + content.height.saturating_sub(1) / 2,
        content.width,
        1.min(content.height),
    );
    frame.render_widget(
        Paragraph::new("No preview")
            .style(Style::new().fg(color_theme.muted))
            .alignment(Alignment::Center),
        message,
    );
}

#[cfg(test)]
mod tests {
    use ratatui::{Terminal, backend::TestBackend};
    use rstest::rstest;

    use super::*;
    use crate::{
        search::{SearchDirectMatchKind, SearchNameMatchKind},
        unicode::CodePoint,
    };

    fn rendered_input(input: &tui_input::Input, cursor: &InputCursor, width: u16) -> TestBackend {
        let backend = TestBackend::new(width, 3);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                render_input(frame, frame.area(), input, cursor, &ColorTheme::default());
            })
            .unwrap();
        terminal.backend().clone()
    }

    #[test]
    fn text_cursor_is_inserted_without_hiding_input_characters() {
        let input = tui_input::Input::new("abc".to_owned()).with_cursor(1);
        let backend = rendered_input(&input, &InputCursor::Text("|".to_owned()), 12);
        let buffer = backend.buffer();

        assert!(!backend.cursor_visible());
        for (x, symbol) in [(1, "a"), (2, "|"), (3, "b"), (4, "c")] {
            assert_eq!(buffer[(x, 0)].symbol(), symbol);
        }
        assert_eq!(input.value(), "abc");
    }

    #[test]
    fn text_cursor_handles_empty_and_wide_input() {
        let empty = rendered_input(
            &tui_input::Input::default(),
            &InputCursor::Text("|".to_owned()),
            12,
        );
        assert_eq!(empty.buffer()[(1, 0)].symbol(), "|");

        let input = tui_input::Input::new("a界b".to_owned()).with_cursor(2);
        let backend = rendered_input(&input, &InputCursor::Text("|".to_owned()), 12);
        let buffer = backend.buffer();
        assert_eq!(buffer[(1, 0)].symbol(), "a");
        assert_eq!(buffer[(2, 0)].symbol(), "界");
        assert_eq!(buffer[(4, 0)].symbol(), "|");
        assert_eq!(buffer[(5, 0)].symbol(), "b");
    }

    #[test]
    fn text_cursor_remains_visible_at_the_right_edge() {
        let input = tui_input::Input::new("abcde".to_owned());
        let backend = rendered_input(&input, &InputCursor::Text("|".to_owned()), 8);
        let buffer = backend.buffer();
        for (x, symbol) in [(1, "b"), (2, "c"), (3, "d"), (4, "e"), (5, "|")] {
            assert_eq!(buffer[(x, 0)].symbol(), symbol);
        }

        let wide_input = tui_input::Input::new("ab界c".to_owned()).with_cursor(3);
        let backend = rendered_input(&wide_input, &InputCursor::Text("|".to_owned()), 8);
        let buffer = backend.buffer();
        assert_eq!(buffer[(1, 0)].symbol(), "a");
        assert_eq!(buffer[(2, 0)].symbol(), "b");
        assert_eq!(buffer[(3, 0)].symbol(), "界");
        assert_eq!(buffer[(5, 0)].symbol(), "|");
    }

    #[rstest]
    #[case(0x2192, "> →   U+2192    RIGHTWARDS ARROW")]
    #[case(0x3042, "> あ  U+3042    HIRAGANA LETTER A")]
    #[case(0x1f600, "> 😀  U+1F600   GRINNING FACE")]
    #[case(0xd800, "> ·   U+D800    <SURROGATE> — No Primary Name")]
    #[case(0x115f, "> ·   U+115F    <DEFAULT IGNORABLE> — HANGUL CHOSEONG FILLER")]
    fn result_lines_align_columns_and_preserve_safe_representations(
        #[case] value: u32,
        #[case] expected: &str,
    ) {
        let result = SearchResult::new_for_test(
            CodePoint::new(value).unwrap(),
            Some(SearchDirectMatchKind::CodePointNotation),
            None,
            None,
        );

        assert_eq!(
            result_line(
                result,
                None,
                true,
                &ColorTheme::default(),
                &UiSettings {
                    selection_cursor: ">".to_owned(),
                    ..Default::default()
                },
            )
            .to_string(),
            expected
        );
    }

    #[test]
    fn highlights_every_match_target_in_a_combined_result() {
        let color_theme = ColorTheme::default();
        let result = SearchResult::new_for_test(
            CodePoint::new(0xface).unwrap(),
            Some(SearchDirectMatchKind::CodePointNotation),
            Some(SearchNameMatchKind::Substring),
            None,
        );
        let line = result_line(
            result,
            Some("FACE"),
            false,
            &color_theme,
            &UiSettings::default(),
        );
        let highlighted = line
            .spans
            .iter()
            .filter(|span| span.style == color_theme.match_style(false))
            .map(|span| span.content.as_ref())
            .collect::<Vec<_>>();

        assert_eq!(highlighted, ["U+FACE", "FACE"]);
    }

    #[test]
    fn highlights_a_literal_safe_representation_with_the_selected_style() {
        let color_theme = ColorTheme::default();
        let result = SearchResult::new_for_test(
            CodePoint::new(0x200d).unwrap(),
            Some(SearchDirectMatchKind::LiteralCharacter),
            None,
            None,
        );
        let line = result_line(result, None, true, &color_theme, &UiSettings::default());
        let highlighted = line
            .spans
            .iter()
            .filter(|span| span.style == color_theme.match_style(true))
            .map(|span| span.content.as_ref())
            .collect::<Vec<_>>();

        assert_eq!(highlighted, ["·", "<DEFAULT IGNORABLE>"]);
    }

    #[test]
    fn empty_selection_cursor_preserves_result_alignment() {
        let result = SearchResult::new_for_test(
            CodePoint::new(0x2192).unwrap(),
            Some(SearchDirectMatchKind::CodePointNotation),
            None,
            None,
        );

        assert_eq!(
            result_line(
                result,
                None,
                true,
                &ColorTheme::default(),
                &UiSettings::default(),
            )
            .to_string(),
            "  →   U+2192    RIGHTWARDS ARROW"
        );
    }

    #[test]
    fn highlights_every_occurrence_in_a_primary_name() {
        let color_theme = ColorTheme::default();
        let spans = name_spans("FACE TO FACE", Some("FACE"), color_theme.match_style(false));
        let highlighted = spans
            .iter()
            .filter(|span| span.style == color_theme.match_style(false))
            .map(|span| span.content.as_ref())
            .collect::<Vec<_>>();

        assert_eq!(highlighted, ["FACE", "FACE"]);
    }

    #[test]
    fn shows_the_matched_alias_before_its_type() {
        let SearchOutcome::Results {
            name_query,
            results,
        } = crate::search::search("latin capital letter gha")
        else {
            panic!("expected alias search results");
        };

        let line = result_line(
            results[0],
            name_query.as_deref(),
            true,
            &ColorTheme::default(),
            &UiSettings {
                selection_cursor: ">".to_owned(),
                ..Default::default()
            },
        );

        assert_eq!(
            line.to_string(),
            "> Ƣ   U+01A2    LATIN CAPITAL LETTER GHA · correction alias"
        );
    }

    #[test]
    fn highlights_the_alias_that_caused_the_result() {
        let color_theme = ColorTheme::default();
        let SearchOutcome::Results {
            name_query,
            results,
        } = crate::search::search("null")
        else {
            panic!("expected alias search results");
        };
        let line = result_line(
            results[0],
            name_query.as_deref(),
            false,
            &color_theme,
            &UiSettings::default(),
        );
        let highlighted = line
            .spans
            .iter()
            .filter(|span| span.style == color_theme.match_style(false))
            .map(|span| span.content.as_ref())
            .collect::<Vec<_>>();

        assert_eq!(highlighted, ["NULL"]);
    }
}
