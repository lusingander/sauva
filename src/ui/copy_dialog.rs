use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    app::AppState,
    copy_dialog::CopyTarget,
    sequence::SequenceState,
    ui::{selectable_list_line, text_preview::safe_cluster, theme::ColorTheme},
    unicode::CodePoint,
};

pub(super) fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: &ColorTheme) {
    let dialog = state.copy_dialog().expect("the copy dialog is open");
    let sequence = state.sequence().expect("copy candidates have a sequence");
    let width = area.width.saturating_sub(4).min(64);
    let height = area.height.min(13);
    let area = Rect::new(
        area.x + (area.width - width) / 2,
        area.y + (area.height - height) / 2,
        width,
        height,
    );
    let muted = Style::new().fg(theme.muted);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .style(theme.base_style())
        .title(Line::styled(" Copy ", theme.heading_style()));
    let inner = block.inner(area);
    let content = Rect::new(
        inner.x + 1,
        inner.y,
        inner.width.saturating_sub(2),
        inner.height,
    );
    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    let source = if state.showing_normalization_result() {
        format!("{} Result", state.normalization().unwrap().form().label())
    } else {
        "Sequence".to_owned()
    };
    let mut lines = vec![
        Line::styled(
            format!(
                "{source} · CP {}/{} · {}",
                sequence.selected_index() + 1,
                sequence.code_points().len(),
                sequence.selected()
            ),
            muted,
        ),
        Line::default(),
    ];
    for (index, target) in CopyTarget::ALL.into_iter().enumerate() {
        let label = if target == CopyTarget::WholeText {
            if state.showing_normalization_result() {
                format!(
                    "Whole {} Result",
                    state.normalization().unwrap().form().label()
                )
            } else {
                "Whole Input".to_owned()
            }
        } else {
            target.label().to_owned()
        };
        let summary = if target == CopyTarget::CodePoint {
            sequence.selected().to_string()
        } else {
            let count = code_point_count(target, sequence);
            format!("{count} code point{}", if count == 1 { "" } else { "s" })
        };
        let row_width = usize::from(content.width.saturating_sub(2));
        let gap = row_width.saturating_sub(label.len() + summary.len());
        lines.push(selectable_list_line(
            Line::from(format!("{label}{}{summary}", " ".repeat(gap))),
            index == dialog.selected_index(),
            content.width,
            theme.selection,
        ));
    }
    let target = dialog.selected_target();
    let text = target.text(sequence);
    let count = code_point_count(target, sequence);
    let preview_width = usize::from(content.width);
    lines.extend([
        Line::default(),
        Line::from(vec![
            Span::styled("Preview", theme.heading_style()),
            Span::styled(
                format!(
                    " · {count} CP · {} byte{}",
                    text.len(),
                    if text.len() == 1 { "" } else { "s" }
                ),
                muted,
            ),
        ]),
        Line::from(fit_atoms(
            text.graphemes(true).map(safe_cluster),
            "",
            preview_width,
        )),
        Line::styled(
            fit_atoms(
                text.chars()
                    .map(|character| CodePoint::from(character).to_string()),
                " ",
                preview_width,
            ),
            muted,
        ),
        Line::default(),
        Line::styled("Display aids are not copied", muted),
    ]);
    frame.render_widget(Paragraph::new(lines), content);
}

fn code_point_count(target: CopyTarget, sequence: &SequenceState) -> usize {
    match target {
        CopyTarget::CodePoint => 1,
        CopyTarget::Grapheme => {
            let point = &sequence.code_points()[sequence.selected_index()];
            sequence.analysis().graphemes()[point.grapheme_index()]
                .code_point_range()
                .len()
        }
        CopyTarget::WholeText => sequence.code_points().len(),
    }
}

/// Clip between complete display atoms without retaining a full rendering of a
/// potentially large input. Ellipses are display aids only.
fn fit_atoms(atoms: impl Iterator<Item = String>, separator: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let mut atoms = atoms.peekable();
    let mut output = String::new();
    let mut used = 0;
    while let Some(atom) = atoms.next() {
        let gap = if output.is_empty() { "" } else { separator };
        let atom_width = Line::from(atom.as_str()).width() + gap.len();
        let reserve = usize::from(atoms.peek().is_some());
        if used + atom_width + reserve > width {
            output.push('…');
            break;
        }
        output.push_str(gap);
        output.push_str(&atom);
        used += atom_width;
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_clips_at_cluster_boundaries_and_never_emits_terminal_controls() {
        let text = "A\u{0301}👩‍💻\r\n\u{1b}\u{202e}";
        assert_eq!(
            fit_atoms(text.graphemes(true).map(safe_cluster), "", 100),
            "A\u{0301}👩‍💻<CR><LF><U+001B><U+202E>"
        );
        assert_eq!(
            fit_atoms(text.graphemes(true).map(safe_cluster), "", 2),
            "A\u{0301}…"
        );
        assert_eq!(fit_atoms(["👩‍💻".to_owned()].into_iter(), "", 2), "👩‍💻");
        assert_eq!(
            fit_atoms(
                ["U+0041".to_owned(), "U+0301".to_owned()].into_iter(),
                " ",
                8
            ),
            "U+0041…"
        );
    }
}
