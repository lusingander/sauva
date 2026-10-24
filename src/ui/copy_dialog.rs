use ratatui::{
    Frame,
    layout::{Margin, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    app::AppState,
    copy_dialog::{CopyTarget, CopyValue},
    ui::{selectable_list_line, text_preview::safe_cluster, theme::ColorTheme},
    unicode::representation::TextFormat,
};

const LABEL_WIDTH: u16 = 8;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: &ColorTheme) {
    let dialog = state.copy_dialog().expect("the copy dialog is open");
    let source = state.copy_source().expect("the copy source is available");
    let targets = source.targets();
    let format = dialog.selected_format(source);
    let width = area.width.saturating_sub(2).min(66);
    let height = area.height.min(if targets.len() > 1 { 12 } else { 10 });
    let background = Rect::new(
        area.x + (area.width - width) / 2,
        area.y + (area.height - height) / 2,
        width,
        height,
    );
    let area = background.inner(Margin::new(1, 1));
    let muted = Style::new().fg(theme.muted);
    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .style(theme.base_style())
        .title(Line::styled(" Copy ", theme.heading_style()));
    if let Some(context) = source.context_label() {
        block = block.title(Line::styled(format!(" {context} "), muted).right_aligned());
    }
    let inner = block.inner(area);
    let content = Rect::new(
        inner.x + 1,
        inner.y,
        inner.width.saturating_sub(2),
        inner.height,
    );
    frame.render_widget(Clear, background);
    frame.render_widget(Block::default().style(theme.base_style()), background);
    frame.render_widget(block, area);

    let mut lines = Vec::new();
    let target_width = content.width.saturating_sub(LABEL_WIDTH);
    for (index, &target) in targets.iter().enumerate() {
        let label = source.target_label(target);
        let mut line = Line::from(field_label(if index == 0 { "Target" } else { "" }, theme));
        if targets.len() == 1 {
            line.spans.push(Span::raw(format!(" {label}")));
            lines.push(line);
            continue;
        }
        let value = source.value(target);
        let count = value.code_point_count();
        let summary = if target == CopyTarget::CodePoint {
            value.format(TextFormat::CodePointList).unwrap()
        } else {
            format!("{count} CP")
        };
        let row_width = usize::from(target_width.saturating_sub(2));
        let gap = row_width.saturating_sub(label.len() + summary.len());
        let candidate = selectable_list_line(
            Line::from(format!("{label}{}{summary}", " ".repeat(gap))),
            index == dialog.selected_index(),
            target_width,
            theme.selection,
        );
        line.spans
            .extend(candidate.spans.into_iter().map(|mut span| {
                span.style = candidate.style.patch(span.style);
                span
            }));
        lines.push(line);
    }
    lines.push(Line::default());
    let formats = source.formats();
    let mut format_line = vec![field_label("Format", theme)];
    if formats.len() > 1 {
        format_line.push(Span::styled(
            format!(" < {} >", format.label()),
            theme.accent_style(),
        ));
    } else {
        format_line.push(Span::raw(format!(" {}", format.label())));
    }
    lines.push(Line::from(format_line));
    lines.push(Line::default());
    let value = source.value(dialog.selected_target(source));
    let count = value.code_point_count();
    let text = value.text();
    let bytes = text.as_ref().map_or_else(String::new, |text| {
        format!(
            " · {} UTF-8 byte{}",
            text.len(),
            if text.len() == 1 { "" } else { "s" }
        )
    });
    lines.push(Line::from(vec![
        Span::styled("Preview", theme.heading_style()),
        Span::styled(format!(" · {count} CP{bytes}"), muted),
    ]));
    let preview_width = usize::from(content.width);
    lines.push(Line::from(preview(value, format, preview_width)));
    frame.render_widget(Paragraph::new(lines), content);
}

fn field_label(label: &str, theme: &ColorTheme) -> Span<'static> {
    Span::styled(
        format!("{label:<width$}", width = usize::from(LABEL_WIDTH)),
        theme.heading_style(),
    )
}

fn preview(value: CopyValue<'_>, format: TextFormat, width: usize) -> String {
    match value.text() {
        Some(text) if format == TextFormat::Text => {
            fit_atoms(text.graphemes(true).map(safe_cluster), width)
        }
        Some(text) => fit_atoms(format.atoms(&text), width),
        None => fit_atoms(value.format(format).into_iter(), width),
    }
}

/// Clip between complete display atoms without retaining a full rendering of a
/// potentially large input. Ellipses are display aids only.
fn fit_atoms(atoms: impl Iterator<Item = String>, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let mut atoms = atoms.peekable();
    let mut output = String::new();
    let mut used = 0;
    while let Some(atom) = atoms.next() {
        let atom_width = Line::from(atom.as_str()).width();
        let reserve = usize::from(atoms.peek().is_some());
        if used + atom_width + reserve > width {
            output.push('…');
            break;
        }
        output.push_str(&atom);
        used += atom_width;
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formatted_previews_stop_between_complete_escapes_and_copy_values_stay_whole() {
        let value = CopyValue::Text {
            text: "A😀B",
            code_point_count: 3,
        };
        assert_eq!(preview(value, TextFormat::JsonString, 8), r#""A…"#);
        assert_eq!(preview(value, TextFormat::RustString, 8), r#""A…"#);
        assert_eq!(preview(value, TextFormat::Utf8Hex, 8), "41…");
        assert_eq!(
            value.format(TextFormat::JsonString).unwrap(),
            r#""A\uD83D\uDE00B""#
        );
    }

    #[test]
    fn preview_clips_at_cluster_boundaries_and_never_emits_terminal_controls() {
        let text = "A\u{0301}👩‍💻\r\n\u{1b}\u{202e}";
        assert_eq!(
            fit_atoms(text.graphemes(true).map(safe_cluster), 100),
            "A\u{0301}👩‍💻<CR><LF><U+001B><U+202E>"
        );
        assert_eq!(
            fit_atoms(text.graphemes(true).map(safe_cluster), 2),
            "A\u{0301}…"
        );
        assert_eq!(fit_atoms(["👩‍💻".to_owned()].into_iter(), 2), "👩‍💻");
    }
}
