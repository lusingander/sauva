use ratatui::{Frame, layout::Rect, text::Line, widgets::Paragraph};

use crate::{
    app::AppState,
    ui::{
        layout,
        scrollbar::{self, ViewportScrollbar},
        selectable_list_line, sequence,
        theme::ColorTheme,
        workspace,
    },
    unicode::text::TextAnalysis,
};

fn original_width(original: &TextAnalysis) -> u16 {
    // Padding, boundary gutter, CP position, spacing and the longest U+XXXXXX.
    (19 + original.code_points().len().to_string().len()
        + original.graphemes().len().to_string().len())
    .max(24) as u16
}

pub fn viewport_heights(area: Rect, state: &AppState) -> (usize, usize) {
    let Some(shell) = layout::calculate(area) else {
        return (0, 0);
    };
    let panes = layout::normalization_result(
        shell.main,
        original_width(state.original_sequence().unwrap().analysis()),
    );
    (
        usize::from(workspace::primary_section(panes.original).content.height),
        usize::from(workspace::primary_section(panes.result).content.height),
    )
}

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: &ColorTheme) {
    let original = state.original_sequence().unwrap().analysis();
    let normalization = state.normalization().unwrap();
    let comparison = normalization.comparison();
    let panes = layout::normalization_result(area, original_width(original));
    render_original(frame, panes.original, state, theme);
    sequence::render_list(
        frame,
        panes.result,
        &comparison.result,
        &format!("{} Result", normalization.form().label()),
        theme,
    );
}

fn render_original(frame: &mut Frame, area: Rect, state: &AppState, theme: &ColorTheme) {
    let original = state.original_sequence().unwrap().analysis();
    let comparison = state.normalization().unwrap().comparison();
    let count = original.code_points().len();
    let counts = format!("{count} CP");
    let secondary = (8 + 1 + counts.len() <= usize::from(area.width.saturating_sub(3)))
        .then_some(counts.as_str());
    let content = workspace::render_primary_heading(frame, area, "Original", secondary, theme);
    let selected = comparison.original_selection(original);
    let marked = if selected.is_empty() {
        0..0
    } else {
        original.code_points()[selected.start].grapheme_index()
            ..original.code_points()[selected.end - 1].grapheme_index() + 1
    };
    let visible = comparison.original_visible_range(original);
    let grapheme_width = original.graphemes().len().to_string().len();
    let position_width = count.to_string().len();
    let gutter_width = grapheme_width as u16 + 3;
    let gutter = Rect::new(content.x, content.y, gutter_width, content.height);
    let body = Rect::new(
        gutter.right(),
        content.y,
        content.width.saturating_sub(gutter_width),
        content.height,
    );
    let (gutters, rows): (Vec<_>, Vec<_>) = visible
        .clone()
        .map(|index| {
            let point = &original.code_points()[index];
            let gutter = sequence::grapheme_gutter(
                original,
                index,
                visible.start,
                &marked,
                grapheme_width,
                theme,
            );
            let row = selectable_list_line(
                Line::from(format!(
                    "{:>position_width$}  {}",
                    index + 1,
                    point.code_point()
                )),
                selected.contains(&index),
                body.width,
                theme.selection,
            );
            (gutter, row)
        })
        .unzip();
    frame.render_widget(Paragraph::new(gutters), gutter);
    frame.render_widget(Paragraph::new(rows), body);
    frame.render_widget(
        ViewportScrollbar::new(count, visible).style(theme.border_style()),
        scrollbar::area_after(content),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn original_columns_fit_all_positions_and_scalar_notations() {
        for (source, expected_width, result_height) in [
            ("A\u{0301}".to_owned(), 24, 13),
            ("\u{10ffff}".repeat(10_000), 29, 8),
        ] {
            let original = TextAnalysis::new(source);
            let width = original_width(&original);
            assert_eq!(width, expected_width);
            let panes = layout::normalization_result(Rect::new(0, 2, 60, 13), width);
            assert_eq!(panes.result.height, result_height);
            assert!(panes.result.width >= 32);
            let content = workspace::primary_section(panes.original).content;
            let gutter_width = original.graphemes().len().to_string().len() as u16 + 3;
            let row = selectable_list_line(
                Line::from(format!("{}  U+10FFFF", original.code_points().len())),
                true,
                content.width.saturating_sub(gutter_width),
                ColorTheme::default().selection,
            );
            assert!(row.to_string().contains("U+10FFFF"));
            assert!(row.to_string().starts_with(' '));
            assert!(row.to_string().ends_with(' '));
        }
    }
}
