use ratatui::{Frame, layout::Rect, style::Style, text::Line, widgets::Paragraph};

use crate::{
    app::AppState,
    inspector::InspectorFieldId,
    property::PropertyState,
    ui::{
        key_value::wrap_value, layout, scrollbar, scrollbar::ViewportScrollbar, theme::ColorTheme,
        workspace,
    },
};

pub fn viewport_metrics(area: Rect, state: &AppState) -> (usize, usize) {
    let Some(shell) = layout::calculate(area) else {
        return (0, 0);
    };
    let Some(property) = state.property() else {
        return (0, 0);
    };
    let content = workspace::primary_canvas(shell.main);
    (
        usize::from(content.height),
        document(property, usize::from(content.width), &ColorTheme::default()).len(),
    )
}

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: &ColorTheme) {
    let Some(property) = state.property() else {
        return;
    };
    let content = workspace::primary_canvas(area);
    let lines = document(property, usize::from(content.width), theme);
    let visible = property.viewport().visible_range();
    let start = visible.start.min(lines.len());
    let end = visible.end.max(start).min(lines.len());
    frame.render_widget(Paragraph::new(lines[start..end].to_vec()), content);
    frame.render_widget(
        ViewportScrollbar::new(lines.len(), start..end).style(theme.border_style()),
        scrollbar::area_after(content),
    );
}

fn document(property: &PropertyState, width: usize, theme: &ColorTheme) -> Vec<Line<'static>> {
    let field = property.field();
    let reference = property.reference();
    let mut builder = DocumentBuilder {
        lines: Vec::new(),
        width,
    };
    builder.heading(&format!("Current value · {}", property.code_point()), theme);
    for value in field.values() {
        builder.text(value, theme.accent_style(), "  ");
    }
    let current = field
        .values()
        .first()
        .map(|value| value.split(" — ").next().unwrap_or(value));
    if field.id() == InspectorFieldId::Age && current != Some("Unassigned") {
        if let Some(value) = current {
            builder.text(
                &format!("First assigned or designated in Unicode {value}."),
                Style::new().fg(theme.muted),
                "  ",
            );
        }
    } else if let Some((_, description)) = reference
        .values
        .iter()
        .find(|(value, description)| Some(*value) == current && !description.is_empty())
    {
        builder.text(description, Style::new().fg(theme.muted), "  ");
    }
    builder.section("Overview", theme);
    builder.text(reference.overview, Style::new(), "  ");
    builder.section(reference.values_heading, theme);
    for (value, description) in reference.values {
        if description.is_empty() {
            builder.lines.push(Line::default());
            builder.text(value, theme.heading_style(), "  ");
        } else {
            let selected = Some(*value) == current;
            let style = if selected {
                theme.accent_style()
            } else {
                Style::new()
            };
            let first = builder.lines.len();
            builder.text(&format!("{value} — {description}"), style, "  ");
            if selected && let Some(line) = builder.lines.get_mut(first) {
                line.spans[0].content = "> ".into();
            }
        }
    }
    if !reference.notes.is_empty() {
        builder.section("Notes", theme);
        builder.text(reference.notes, Style::new(), "  ");
    }
    builder.section("Reference", theme);
    builder.text(reference.source, Style::new().fg(theme.link), "  ");
    builder.text(reference.url, Style::new().fg(theme.link), "  ");
    builder.lines
}

struct DocumentBuilder {
    lines: Vec<Line<'static>>,
    width: usize,
}

impl DocumentBuilder {
    fn heading(&mut self, text: &str, theme: &ColorTheme) {
        self.text(text, theme.heading_style(), "");
    }

    fn section(&mut self, text: &str, theme: &ColorTheme) {
        self.lines.push(Line::default());
        self.heading(text, theme);
    }

    fn text(&mut self, text: &str, style: Style, indent: &str) {
        for line in wrap_value(text, self.width.saturating_sub(indent.len())) {
            self.lines.push(Line::from(vec![
                ratatui::text::Span::styled(indent.to_owned(), style),
                ratatui::text::Span::styled(line, style),
            ]));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        inspector::{InspectorDisplayMode, InspectorField},
        unicode::CodePoint,
    };

    #[test]
    fn every_property_fits_the_minimum_width_and_keeps_the_complete_current_value() {
        let theme = ColorTheme::default();
        for value in [0, 0x41, 0xdf, 0xe9, 0x301, 0x200d, 0xd800, 0x10ffff] {
            let code_point = CodePoint::new(value).unwrap();
            for field in InspectorField::for_code_point(code_point, InspectorDisplayMode::All) {
                let property = PropertyState::new(code_point, field);
                let lines = document(&property, 57, &theme);
                assert!(
                    lines.iter().all(|line| line.width() <= 57),
                    "{:?}",
                    property.field().id()
                );
                let text = lines
                    .iter()
                    .map(Line::to_string)
                    .collect::<Vec<_>>()
                    .join("\n");
                assert!(text.starts_with(&format!("Current value · {code_point}")));
                assert!(text.contains("Overview\n"));
                assert!(text.contains("Reference\n"));
                let unwrapped = text.split_whitespace().collect::<String>();
                for value in property.field().values() {
                    assert!(unwrapped.contains(&value.split_whitespace().collect::<String>()));
                }
            }
        }
    }
}
