use std::ops::Range;

use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::{
    app::AppState,
    normalization::TextMetrics,
    ui::{selectable_list_line, settings::UiSettings, theme::ColorTheme},
    unicode::{
        CodePoint, GeneralCategory, UnicodeDatabase,
        text::{NormalizationForm, TextAnalysis},
    },
};

pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    theme: &ColorTheme,
    ui: &UiSettings,
) {
    let original = state.sequence().unwrap().analysis();
    let normalization = state.normalization().unwrap();
    let comparison = normalization.comparison();
    let result = comparison.result.analysis();
    // All thirteen content rows remain visible at the minimum 60 × 16 size.
    let canvas = Rect::new(
        area.x + 1,
        area.y,
        area.width.saturating_sub(3),
        area.height,
    );
    let width = usize::from(canvas.width);
    let before_ranges = comparison
        .diff
        .regions()
        .iter()
        .map(|region| region.original.clone())
        .collect::<Vec<_>>();
    let after_ranges = comparison
        .diff
        .regions()
        .iter()
        .map(|region| region.result.clone())
        .collect::<Vec<_>>();
    let before = preview(original, &before_ranges, width.saturating_sub(1), theme);
    let after = preview(result, &after_ranges, width.saturating_sub(1), theme);
    let metrics = TextMetrics::original(original);
    let mut lines = vec![
        heading(
            "Original",
            &format!(
                "{} CP · {} Bytes · {} GC",
                metrics.code_points, metrics.bytes, metrics.graphemes
            ),
            width,
            theme,
        ),
        before.0,
        before.1,
        Line::default(),
    ];
    let metrics = NormalizationForm::ALL.map(|form| normalization.metrics(form));
    let cp_width = metrics
        .iter()
        .map(|value| value.code_points.to_string().len())
        .max()
        .unwrap()
        .max(2);
    let byte_width = metrics
        .iter()
        .map(|value| value.bytes.to_string().len())
        .max()
        .unwrap()
        .max(5);
    let gc_width = metrics
        .iter()
        .map(|value| value.graphemes.to_string().len())
        .max()
        .unwrap()
        .max(2);
    lines.push(table_row(
        "  Form  vs Original",
        &format!(
            "{:>cp_width$}  {:>byte_width$}  {:>gc_width$}",
            "CP", "Bytes", "GC"
        ),
        width,
        Style::new().fg(theme.muted),
    ));
    for (index, form) in NormalizationForm::ALL.into_iter().enumerate() {
        let selected = normalization.form() == form;
        let metrics = metrics[index];
        let changed = original.normalization().get(form).is_changed();
        let left = format!(
            "{} {:<4}  {}",
            ui.selection_marker(selected),
            form.label(),
            if changed { "Changed" } else { "Unchanged" }
        );
        let right = format!(
            "{:>cp_width$}  {:>byte_width$}  {:>gc_width$}",
            metrics.code_points, metrics.bytes, metrics.graphemes
        );
        lines.push(selectable_list_line(
            table_row(&left, &right, width, Style::default()),
            selected,
            canvas.width,
            theme.selection,
        ));
    }
    lines.push(Line::default());
    let changed = !comparison.diff.regions().is_empty();
    let status = if !changed {
        "Same as Original".to_owned()
    } else if comparison.diff.is_grouped() {
        "Changed span (grouped)".to_owned()
    } else {
        format!(
            "{} changed region{}",
            comparison.diff.regions().len(),
            if comparison.diff.regions().len() == 1 {
                ""
            } else {
                "s"
            }
        )
    };
    lines.push(heading(
        &format!("{} Result", normalization.form().label()),
        &status,
        width,
        theme,
    ));
    if changed {
        lines.extend([after.0, after.1]);
    } else {
        lines.extend([
            Line::styled(
                " Same text and code points as Original",
                Style::new().fg(theme.muted),
            ),
            Line::default(),
        ]);
    }
    if canvas.height > 13 && changed && (before.2 || after.2) {
        lines.push(Line::styled(
            " Preview near first change · … omitted",
            Style::new().fg(theme.muted),
        ));
    }
    frame.render_widget(Paragraph::new(lines), canvas);
}

fn heading(left: &str, right: &str, width: usize, theme: &ColorTheme) -> Line<'static> {
    let mut line = table_row(left, right, width, Style::new().fg(theme.muted));
    line.spans[0].style = theme.heading_style();
    line
}

fn table_row(left: &str, right: &str, width: usize, style: Style) -> Line<'static> {
    let padding = width.saturating_sub(Line::from(left).width() + Line::from(right).width());
    Line::from(vec![
        Span::styled(left.to_owned(), style),
        Span::styled(" ".repeat(padding), style),
        Span::styled(right.to_owned(), style),
    ])
}

fn preview(
    analysis: &TextAnalysis,
    changes: &[Range<usize>],
    width: usize,
    theme: &ColorTheme,
) -> (Line<'static>, Line<'static>, bool) {
    let first = changes.first().map_or(0, |range| range.start);
    let start = first.saturating_sub(2);
    let clusters = analysis
        .graphemes()
        .iter()
        .enumerate()
        .skip(start)
        .map(|(index, cluster)| {
            (
                safe_cluster(&analysis.source()[cluster.byte_range()]),
                changes.iter().any(|range| range.contains(&index)),
            )
        });
    let (text, clipped_text) = fit_preview(clusters, start > 0, width, theme.base_style(), theme);
    let first_point = analysis
        .graphemes()
        .get(first)
        .map_or(0, |cluster| cluster.code_point_range().start);
    let point_start = first_point.saturating_sub(2);
    let points = analysis
        .code_points()
        .iter()
        .skip(point_start)
        .map(|point| {
            (
                format!("{} ", point.code_point()),
                changes
                    .iter()
                    .any(|range| range.contains(&point.grapheme_index())),
            )
        });
    let (code_points, clipped_points) = fit_preview(
        points,
        point_start > 0,
        width,
        Style::new().fg(theme.muted),
        theme,
    );
    (text, code_points, clipped_text || clipped_points)
}

fn fit_preview(
    atoms: impl Iterator<Item = (String, bool)>,
    leading: bool,
    width: usize,
    base: Style,
    theme: &ColorTheme,
) -> (Line<'static>, bool) {
    let mut spans = vec![Span::raw(" ")];
    let mut used = 0;
    if leading {
        spans.push(Span::styled("… ", base));
        used = 2;
    }
    let mut truncated = false;
    for (text, changed) in atoms {
        let atom_width = Line::from(text.as_str()).width();
        if used + atom_width + 2 > width {
            truncated = true;
            break;
        }
        used += atom_width;
        spans.push(Span::styled(
            text,
            if changed {
                theme.difference_style()
            } else {
                base
            },
        ));
    }
    if truncated {
        spans.push(Span::styled(" …", base));
    }
    (Line::from(spans), leading || truncated)
}

/// Escape terminal controls and invisible formatting without splitting safe
/// clusters into dotted-circle representations of every combining code point.
fn safe_cluster(text: &str) -> String {
    use GeneralCategory as G;
    if text.len() > 1024 {
        return format!("<cluster: {} CP>", text.chars().count());
    }
    let has_base = text.chars().any(|character| {
        let record = UnicodeDatabase::lookup(CodePoint::from(character));
        !record.is_default_ignorable()
            && !matches!(
                record.general_category(),
                G::NonspacingMark
                    | G::SpacingMark
                    | G::EnclosingMark
                    | G::Control
                    | G::Format
                    | G::SpaceSeparator
                    | G::LineSeparator
                    | G::ParagraphSeparator
                    | G::Unassigned
                    | G::PrivateUse
            )
    });
    let mut output = String::new();
    for character in text.chars() {
        let point = CodePoint::from(character);
        let record = UnicodeDatabase::lookup(point);
        let shaping =
            matches!(point.value(), 0x200c | 0x200d | 0xfe00..=0xfe0f | 0xe0100..=0xe01ef);
        match character {
            ' ' => output.push('␠'),
            '\t' => output.push_str("<TAB>"),
            '\r' => output.push_str("<CR>"),
            '\n' => output.push_str("<LF>"),
            _ if shaping && has_base => output.push(character),
            _ if record.is_default_ignorable() => output.push_str(&format!("<{point}>")),
            _ if matches!(
                record.general_category(),
                G::NonspacingMark | G::SpacingMark | G::EnclosingMark
            ) =>
            {
                if output.is_empty() && !has_base {
                    output.push('◌');
                }
                output.push(character);
            }
            _ => {
                let display = record.display_representation();
                if display.as_str() == character.to_string() {
                    output.push(character);
                } else {
                    output.push_str(&format!("<{point}>"));
                }
            }
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{Action, update};

    #[test]
    fn preserves_safe_clusters_and_escapes_controls_and_bidi() {
        assert_eq!(safe_cluster("A\u{0301}"), "A\u{0301}");
        assert_eq!(safe_cluster("👩‍💻"), "👩‍💻");
        assert_eq!(safe_cluster("\r\n"), "<CR><LF>");
        assert_eq!(safe_cluster("\u{202e}"), "<U+202E>");
        assert_eq!(safe_cluster("\u{0301}"), "◌\u{0301}");
        assert_eq!(safe_cluster("\u{200d}"), "<U+200D>");
        assert_eq!(safe_cluster("\u{00a0}"), "<U+00A0>");
    }

    #[test]
    fn emphasizes_the_cluster_and_all_its_code_points_with_configured_colors() {
        let mut state = AppState::with_sequence("A\u{0301} ①👩‍💻".to_owned());
        update(&mut state, Action::OpenNormalization);
        let original = state.sequence().unwrap().analysis();
        let theme = ColorTheme::default();
        let (text, points, _) = preview(original, std::slice::from_ref(&(0..1)), 56, &theme);
        assert_eq!(
            text.spans
                .iter()
                .filter(|span| span.style == theme.difference_style())
                .count(),
            1
        );
        assert_eq!(
            points
                .spans
                .iter()
                .filter(|span| span.style == theme.difference_style())
                .count(),
            2
        );
        assert_ne!(theme.difference_style(), theme.selection.style());
    }
}
