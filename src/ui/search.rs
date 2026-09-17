use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph},
};

use crate::{
    app::AppState,
    search::{SearchDirectMatchKind, SearchOutcome, SearchResult},
    ui::{
        layout,
        scrollbar::{self, ViewportScrollbar},
        selectable_list_line, selection_preview,
        settings::UiSettings,
        theme::ColorTheme,
    },
    unicode::UnicodeDatabase,
};

const CHARACTER_COLUMN_WIDTH: usize = 2;
const CODE_POINT_COLUMN_WIDTH: usize = 8;

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

    render_input(frame, layout.input, search.input());
    render_results(
        frame,
        layout.results,
        search.outcome(),
        search.selected_index(),
        search.visible_result_range(),
        color_theme,
        ui,
    );
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
}

fn render_input(frame: &mut Frame, area: Rect, input: &tui_input::Input) {
    let width = area.width.saturating_sub(4).max(1);
    let scroll = input.visual_scroll(usize::from(width));
    frame.render_widget(
        Paragraph::new(input.value())
            .scroll((0, scroll as u16))
            .block(
                Block::bordered()
                    .title(" Search ")
                    .padding(Padding::horizontal(1)),
            ),
        area,
    );

    let cursor = input.visual_cursor().max(scroll) - scroll;
    frame.set_cursor_position((area.x + cursor as u16 + 2, area.y + 1));
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
    let title = if matches!(outcome, SearchOutcome::Results { .. }) {
        format!(" Results · {} ", results.len())
    } else {
        " Results ".to_owned()
    };
    let block = Block::bordered()
        .title(title)
        .padding(Padding::horizontal(1));
    let content = block.inner(area);
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
                selectable_list_line(line, selected, content.width, color_theme.list.selection)
            })
            .collect(),
    };

    frame.render_widget(Paragraph::new(lines).block(block), area);
    frame.render_widget(
        ViewportScrollbar::new(results.len(), visible_results).style(color_theme.base_style()),
        scrollbar::area_after(content),
    );
}

fn search_prompt_lines(color_theme: &ColorTheme) -> Vec<Line<'static>> {
    vec![
        Line::styled(
            "Search by Unicode name, code point, or character",
            Style::new().add_modifier(Modifier::BOLD),
        ),
        Line::from(vec![
            Span::styled(
                "Examples",
                Style::new().fg(color_theme.search_message.example_label),
            ),
            Span::styled(
                "  rightwards arrow · U+2192 · →",
                Style::new().fg(color_theme.search_message.detail),
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
            Style::new().fg(color_theme.search_message.detail),
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
    let match_style = color_theme.search_match.style(selected);
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
    spans.extend(name_spans(
        UnicodeDatabase::primary_name_or_fallback(code_point),
        result.name_match().and(name_query),
        match_style,
    ));

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
    let block = Block::bordered()
        .title(" Selection Preview ")
        .padding(Padding::horizontal(1));
    let content = block.inner(area);
    frame.render_widget(block, area);

    let message = Rect::new(
        content.x,
        content.y + content.height.saturating_sub(1) / 2,
        content.width,
        1.min(content.height),
    );
    frame.render_widget(
        Paragraph::new("No preview")
            .style(Style::new().fg(color_theme.selection_preview.empty))
            .alignment(Alignment::Center),
        message,
    );
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::{
        search::{SearchDirectMatchKind, SearchNameMatchKind},
        unicode::CodePoint,
    };

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
        );

        assert_eq!(
            result_line(
                result,
                None,
                true,
                &ColorTheme::default(),
                &UiSettings {
                    selection_cursor: ">".to_owned(),
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
            .filter(|span| span.style == color_theme.search_match.style(false))
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
        );
        let line = result_line(result, None, true, &color_theme, &UiSettings::default());
        let highlighted = line
            .spans
            .iter()
            .filter(|span| span.style == color_theme.search_match.style(true))
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
        let spans = name_spans(
            "FACE TO FACE",
            Some("FACE"),
            color_theme.search_match.style(false),
        );
        let highlighted = spans
            .iter()
            .filter(|span| span.style == color_theme.search_match.style(false))
            .map(|span| span.content.as_ref())
            .collect::<Vec<_>>();

        assert_eq!(highlighted, ["FACE", "FACE"]);
    }
}
