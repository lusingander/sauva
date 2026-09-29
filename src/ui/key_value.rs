use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::Line,
    widgets::{Block, Padding, Paragraph},
};

use crate::ui::theme::ColorTheme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyValue<'a> {
    label: &'a str,
    value: &'a str,
}

impl<'a> KeyValue<'a> {
    pub const fn new(label: &'a str, value: &'a str) -> Self {
        Self { label, value }
    }
}

pub fn render(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    label_width: u16,
    entries: &[KeyValue<'_>],
    color_theme: &ColorTheme,
) {
    let block = Block::bordered()
        .title(title.to_owned())
        .padding(Padding::horizontal(1));
    let content = block.inner(area);
    frame.render_widget(block, area);

    let label_width = effective_label_width(content.width, label_width, entries);
    let value_width = content.width.saturating_sub(label_width);
    if value_width == 0 {
        return;
    }

    let mut y = content.y;
    for entry in entries {
        let remaining_height = content.bottom().saturating_sub(y);
        if remaining_height == 0 {
            break;
        }

        let wrapped_value = wrap_value(entry.value, usize::from(value_width));
        let height = wrapped_value.len().min(usize::from(remaining_height)) as u16;
        let label_area = Rect::new(content.x, y, label_width, 1);
        let value_area = Rect::new(content.x + label_width, y, value_width, height);
        let value_lines = wrapped_value
            .into_iter()
            .map(Line::from)
            .collect::<Vec<_>>();

        frame.render_widget(
            Paragraph::new(entry.label).style(Style::new().fg(color_theme.muted)),
            label_area,
        );
        frame.render_widget(Paragraph::new(value_lines), value_area);
        y += height;
    }
}

pub fn required_height(content_width: u16, label_width: u16, entries: &[KeyValue<'_>]) -> u16 {
    let label_width = effective_label_width(content_width, label_width, entries);
    let value_width = content_width.saturating_sub(label_width);
    if value_width == 0 {
        return 0;
    }

    entries.iter().fold(0, |height, entry| {
        height.saturating_add(wrap_value(entry.value, usize::from(value_width)).len() as u16)
    })
}

fn effective_label_width(content_width: u16, label_width: u16, entries: &[KeyValue<'_>]) -> u16 {
    let required_label_width = entries
        .iter()
        .map(|entry| text_width(entry.label))
        .max()
        .unwrap_or_default()
        .saturating_add(1);
    usize::from(label_width)
        .max(required_label_width)
        .min(usize::from(content_width)) as u16
}

pub fn wrap_value(value: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return Vec::new();
    }

    let mut lines = Vec::new();
    for paragraph in value.split('\n') {
        let mut current = String::new();
        for word in paragraph.split_whitespace() {
            let separator_width = usize::from(!current.is_empty());
            if text_width(&current) + separator_width + text_width(word) <= width {
                if separator_width == 1 {
                    current.push(' ');
                }
                current.push_str(word);
                continue;
            }

            if !current.is_empty() {
                lines.push(std::mem::take(&mut current));
            }
            let mut chunks = split_to_width(word, width);
            if let Some(last) = chunks.pop() {
                lines.extend(chunks);
                current = last;
            }
        }

        if !current.is_empty() || paragraph.is_empty() {
            lines.push(current);
        }
    }

    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn split_to_width(value: &str, width: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();
    let mut current_width = 0;

    for character in value.chars() {
        let character_width = text_width(&character.to_string());
        if current_width > 0 && current_width + character_width > width {
            chunks.push(std::mem::take(&mut current));
            current_width = 0;
        }
        current.push(character);
        current_width += character_width;
    }
    if !current.is_empty() {
        chunks.push(current);
    }

    chunks
}

pub fn text_width(value: &str) -> usize {
    Line::from(value).width()
}

#[cfg(test)]
mod tests {
    use ratatui::{Terminal, backend::TestBackend};

    use super::*;

    #[test]
    fn wrapped_values_stay_in_the_value_column_and_push_following_entries_down() {
        let backend = TestBackend::new(30, 8);
        let mut terminal = Terminal::new(backend).unwrap();
        let entries = [
            KeyValue::new("Primary Name", "ALPHA BETA GAMMA"),
            KeyValue::new("Block", "Basic Latin"),
        ];

        terminal
            .draw(|frame| {
                render(
                    frame,
                    frame.area(),
                    " Details ",
                    13,
                    &entries,
                    &ColorTheme::default(),
                );
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        for x in 2..15 {
            assert_eq!(buffer.cell((x, 2)).unwrap().symbol(), " ");
        }
        assert_eq!(buffer.cell((15, 2)).unwrap().symbol(), "G");
        assert_eq!(buffer.cell((2, 3)).unwrap().symbol(), "B");
        assert_eq!(buffer.cell((15, 3)).unwrap().symbol(), "B");
    }

    #[test]
    fn wrapping_prefers_word_boundaries_and_splits_words_that_exceed_the_column() {
        let wrapped = wrap_value("ALPHA BETA GAMMA_SUPERLONG", 11);

        assert_eq!(wrapped, ["ALPHA BETA", "GAMMA_SUPER", "LONG"]);
    }

    #[test]
    fn measures_the_height_of_wrapped_values() {
        let entries = [
            KeyValue::new("Primary Name", "ALPHA BETA GAMMA"),
            KeyValue::new("Block", "Basic Latin"),
        ];

        assert_eq!(required_height(26, 13, &entries), 3);
    }
}
