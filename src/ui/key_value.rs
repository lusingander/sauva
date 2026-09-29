use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::ui::{theme::ColorTheme, workspace};

const STACKED_VALUE_INDENT: usize = 2;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub lines: Vec<Line<'static>>,
}

impl Document {
    pub fn for_entries(
        content_width: u16,
        label_width: u16,
        entries: &[KeyValue<'_>],
        color_theme: &ColorTheme,
    ) -> Self {
        let width = usize::from(content_width);
        let label_width = effective_label_width(content_width, label_width, entries);
        let mut lines = Vec::new();
        for entry in entries {
            lines.extend(property_lines(
                entry.label,
                [entry.value.to_owned()],
                width,
                label_width,
                "",
                Style::new().fg(color_theme.muted),
                Style::new(),
                None,
            ));
        }
        Self { lines }
    }

    pub fn height(&self) -> u16 {
        u16::try_from(self.lines.len()).unwrap_or(u16::MAX)
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
    let content = workspace::render_rail_heading(frame, area, title, None, color_theme);
    render_entries(frame, content, label_width, entries, color_theme);
}

pub fn render_entries(
    frame: &mut Frame,
    area: Rect,
    label_width: u16,
    entries: &[KeyValue<'_>],
    color_theme: &ColorTheme,
) {
    let document = Document::for_entries(area.width, label_width, entries, color_theme);
    frame.render_widget(Paragraph::new(document.lines), area);
}

pub fn required_height(content_width: u16, label_width: u16, entries: &[KeyValue<'_>]) -> u16 {
    Document::for_entries(content_width, label_width, entries, &ColorTheme::default()).height()
}

#[allow(clippy::too_many_arguments)]
pub fn property_lines(
    label: &str,
    values: impl IntoIterator<Item = String>,
    width: usize,
    label_width: usize,
    label_prefix: &str,
    label_style: Style,
    value_style: Style,
    selected_style: Option<Style>,
) -> Vec<Line<'static>> {
    if width == 0 {
        return Vec::new();
    }

    let mut values = values.into_iter().collect::<Vec<_>>();
    if values.is_empty() {
        values.push(String::new());
    }
    let label_width = label_width.min(width);
    let inline_value_width = width.saturating_sub(label_width);
    let stacked = inline_value_width == 0
        || values
            .iter()
            .any(|value| value.contains('\n') || wrap_value(value, inline_value_width).len() > 1);

    if stacked {
        stacked_property_lines(
            label,
            &values,
            width,
            label_prefix,
            label_style,
            value_style,
            selected_style,
        )
    } else {
        inline_property_lines(
            label,
            &values,
            width,
            label_width,
            label_prefix,
            label_style,
            value_style,
            selected_style,
        )
    }
}

#[allow(clippy::too_many_arguments)]
fn inline_property_lines(
    label: &str,
    values: &[String],
    width: usize,
    label_width: usize,
    label_prefix: &str,
    label_style: Style,
    value_style: Style,
    selected_style: Option<Style>,
) -> Vec<Line<'static>> {
    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let prefix = if index == 0 {
                padded_label(label, label_prefix, label_width)
            } else {
                " ".repeat(label_width)
            };
            property_line(
                prefix,
                value.clone(),
                width,
                label_style,
                value_style,
                selected_style,
            )
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn stacked_property_lines(
    label: &str,
    values: &[String],
    width: usize,
    label_prefix: &str,
    label_style: Style,
    value_style: Style,
    selected_style: Option<Style>,
) -> Vec<Line<'static>> {
    let indent_width = STACKED_VALUE_INDENT.min(width.saturating_sub(1));
    let value_width = width.saturating_sub(indent_width).max(1);
    let mut lines = vec![property_line(
        format!("{label_prefix}{label}"),
        String::new(),
        width,
        label_style,
        value_style,
        selected_style,
    )];
    for value in values {
        lines.extend(wrap_value(value, value_width).into_iter().map(|value| {
            property_line(
                " ".repeat(indent_width),
                value,
                width,
                label_style,
                value_style,
                selected_style,
            )
        }));
    }
    lines
}

fn property_line(
    prefix: String,
    value: String,
    width: usize,
    label_style: Style,
    value_style: Style,
    selected_style: Option<Style>,
) -> Line<'static> {
    let (label_style, value_style) =
        selected_style.map_or((label_style, value_style), |style| (style, style));
    let mut line = Line::from(vec![
        Span::styled(prefix, label_style),
        Span::styled(value, value_style),
    ]);
    if let Some(style) = selected_style {
        let padding = width.saturating_sub(line.width());
        line.push_span(Span::styled(" ".repeat(padding), style));
    }
    line
}

fn padded_label(label: &str, prefix: &str, width: usize) -> String {
    let mut output = format!("{prefix}{label}");
    let padding = width.saturating_sub(text_width(&output));
    output.push_str(&" ".repeat(padding));
    output
}

fn effective_label_width(content_width: u16, label_width: u16, entries: &[KeyValue<'_>]) -> usize {
    let required_label_width = entries
        .iter()
        .map(|entry| text_width(entry.label))
        .max()
        .unwrap_or_default()
        .saturating_add(1);
    usize::from(label_width)
        .max(required_label_width)
        .min(usize::from(content_width))
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
    fn long_values_move_below_the_label_and_use_the_full_width() {
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
        assert_eq!(buffer.cell((2, 1)).unwrap().symbol(), "P");
        assert_eq!(buffer.cell((2, 2)).unwrap().symbol(), " ");
        assert_eq!(buffer.cell((4, 2)).unwrap().symbol(), "A");
        assert_eq!(buffer.cell((2, 3)).unwrap().symbol(), "B");
        assert_eq!(buffer.cell((15, 3)).unwrap().symbol(), "B");
    }

    #[test]
    fn short_values_remain_inline() {
        let lines = property_lines(
            "Block",
            ["Basic Latin".to_owned()],
            26,
            13,
            "",
            Style::new(),
            Style::new(),
            None,
        );

        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].to_string(), "Block        Basic Latin");
    }

    #[test]
    fn wrapping_prefers_word_boundaries_and_splits_words_that_exceed_the_column() {
        let wrapped = wrap_value("ALPHA BETA GAMMA_SUPERLONG", 11);

        assert_eq!(wrapped, ["ALPHA BETA", "GAMMA_SUPER", "LONG"]);
    }

    #[test]
    fn measures_the_height_from_the_same_document_used_for_rendering() {
        let entries = [
            KeyValue::new("Primary Name", "ALPHA BETA GAMMA"),
            KeyValue::new("Block", "Basic Latin"),
        ];

        let document = Document::for_entries(26, 13, &entries, &ColorTheme::default());
        assert_eq!(required_height(26, 13, &entries), document.height());
        assert_eq!(document.height(), 3);
    }
}
