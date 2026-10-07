use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::AppState;
use crate::inspector::InspectorField;
use crate::ui::{
    key_value, layout, padded_line, padded_line_content_width,
    scrollbar::{self, ViewportScrollbar},
    theme::ColorTheme,
    workspace,
};
use crate::unicode::CodePoint;

const LABEL_WIDTH: usize = 27;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ViewportMetrics {
    pub viewport_height: usize,
    pub document_height: usize,
    pub field_ranges: Vec<std::ops::Range<usize>>,
}

pub fn viewport_metrics(area: Rect, state: &AppState) -> ViewportMetrics {
    let Some(layout) = layout::calculate(area) else {
        return ViewportMetrics::default();
    };
    let details = layout::inspector(layout.main).details;
    let content = content_area(details);
    let document = InspectorDocument::for_code_point(state.selected(), usize::from(content.width));
    ViewportMetrics {
        viewport_height: usize::from(content.height),
        document_height: document.lines.len(),
        field_ranges: document.field_ranges,
    }
}

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, color_theme: &ColorTheme) {
    let content = content_area(area);
    let document = InspectorDocument::with_color_theme(
        state.selected(),
        usize::from(content.width),
        state.inspector().selected_index(),
        color_theme,
    );
    let range = state.inspector().visible_range();
    let start = range.start.min(document.lines.len());
    let end = range.end.max(start).min(document.lines.len());
    let lines = document.lines[start..end].to_vec();

    frame.render_widget(Paragraph::new(lines), content);
    frame.render_widget(
        ViewportScrollbar::new(document.lines.len(), start..end).style(color_theme.border_style()),
        scrollbar::area_for_primary(frame.area(), area, content),
    );
}

fn content_area(area: Rect) -> Rect {
    workspace::primary_canvas(area)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct InspectorDocument {
    lines: Vec<Line<'static>>,
    field_ranges: Vec<std::ops::Range<usize>>,
}

impl InspectorDocument {
    fn for_code_point(code_point: CodePoint, width: usize) -> Self {
        Self::with_color_theme(code_point, width, 0, &ColorTheme::default())
    }

    fn with_color_theme(
        code_point: CodePoint,
        width: usize,
        selected_index: usize,
        color_theme: &ColorTheme,
    ) -> Self {
        let mut builder = DocumentBuilder::new(width, color_theme);
        let mut field_ranges = Vec::new();
        let mut previous_section = None;
        for (index, field) in InspectorField::for_code_point(code_point)
            .into_iter()
            .enumerate()
        {
            if previous_section != Some(field.section()) {
                builder.section(field.section().label());
                previous_section = Some(field.section());
            }
            let start = builder.lines.len();
            let selected = index == selected_index;
            builder.fields(field.label(), field.values().iter().cloned(), selected);
            field_ranges.push(start..builder.lines.len());
        }

        Self {
            lines: builder.lines,
            field_ranges,
        }
    }
}

struct DocumentBuilder {
    lines: Vec<Line<'static>>,
    width: usize,
    label_width: usize,
    color_theme: ColorTheme,
}

impl DocumentBuilder {
    fn new(width: usize, color_theme: &ColorTheme) -> Self {
        let label_width = LABEL_WIDTH.min(padded_line_content_width(width).saturating_sub(1));
        Self {
            lines: Vec::new(),
            width,
            label_width,
            color_theme: *color_theme,
        }
    }

    fn section(&mut self, label: &str) {
        if !self.lines.is_empty() {
            self.lines.push(Line::from(Span::styled(
                "─".repeat(self.width),
                self.color_theme.border_style(),
            )));
        }
        self.lines.push(Line::from(Span::styled(
            label.to_owned(),
            self.color_theme.heading_style(),
        )));
    }

    fn fields(&mut self, label: &str, values: impl IntoIterator<Item = String>, selected: bool) {
        let selected_style = selected.then(|| self.color_theme.selection.style());
        let lines = key_value::property_lines(
            label,
            values,
            padded_line_content_width(self.width),
            self.label_width,
            Style::new().fg(self.color_theme.muted),
            Style::new(),
            selected_style,
        );
        self.lines.extend(lines.into_iter().map(|line| {
            let line = padded_line(line, self.width);
            if let Some(style) = selected_style {
                line.style(style)
            } else {
                line
            }
        }));
    }
}

#[cfg(test)]
mod tests {
    use ratatui::layout::Rect;

    use super::*;
    use crate::{
        app::{Action, update},
        fixtures,
        graphics::{GraphicsAvailability, GraphicsProtocol},
        preview::GlyphPreviewUpdate,
    };

    #[test]
    fn builds_grouped_basic_details_in_the_agreed_order() {
        let document = InspectorDocument::for_code_point(CodePoint::from('A'), 96);
        let text = document
            .lines
            .iter()
            .map(Line::to_string)
            .collect::<Vec<_>>();

        assert_eq!(
            text.iter()
                .filter(|line| matches!(
                    line.as_str(),
                    "Identity" | "Classification" | "Encoding" | "Normalization" | "Data"
                ))
                .collect::<Vec<_>>(),
            [
                "Identity",
                "Classification",
                "Encoding",
                "Normalization",
                "Data"
            ]
        );
        assert!(text.iter().any(|line| line.contains("Na — Narrow")));
        assert!(text.iter().any(|line| line.contains("L — Left To Right")));
        assert!(text.iter().any(|line| line.contains("\\u0041")));
        assert!(text.iter().any(|line| line.trim_end().ends_with("None")));
    }

    #[test]
    fn expands_aliases_and_decomposition_mappings_into_independent_lines() {
        let aliases = InspectorDocument::for_code_point(CodePoint::new(0).unwrap(), 96)
            .lines
            .into_iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>();
        let decomposition = InspectorDocument::for_code_point(CodePoint::new(0x00e9).unwrap(), 96)
            .lines
            .into_iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>();

        assert!(aliases.iter().any(|line| line.contains("NULL — control")));
        assert!(
            aliases
                .iter()
                .any(|line| line.trim() == "NUL — abbreviation")
        );
        assert!(
            decomposition
                .iter()
                .any(|line| line.contains("U+0065 LATIN SMALL LETTER E"))
        );
        assert!(decomposition.iter().any(|line| {
            line.trim_start()
                .starts_with("U+0301 COMBINING ACUTE ACCENT")
        }));
    }

    #[test]
    fn wrapped_properties_keep_padding_and_the_complete_value() {
        use ratatui::{buffer::Buffer, widgets::Widget};

        let theme = ColorTheme::default();
        let value = "ABCDEFGHIあe\u{0301}JKLMNOPQRST";
        for selected in [false, true] {
            let mut builder = DocumentBuilder::new(12, &theme);
            builder.fields("Value", [value.to_owned()], selected);
            let text = builder.lines[1..]
                .iter()
                .map(|line| line.to_string().trim().to_owned())
                .collect::<String>();
            assert_eq!(text, value);

            for line in builder.lines {
                let area = Rect::new(0, 0, 12, 1);
                let mut buffer = Buffer::empty(area);
                line.render(area, &mut buffer);
                for x in [0, 11] {
                    assert_eq!(buffer[(x, 0)].symbol(), " ");
                    if selected {
                        assert_eq!(buffer[(x, 0)].bg, theme.selection.bg);
                    }
                }
            }
        }
    }

    #[test]
    fn wraps_values_before_the_viewport_is_applied() {
        let wide = InspectorDocument::for_code_point(CodePoint::new(0xd800).unwrap(), 96);
        let narrow = InspectorDocument::for_code_point(CodePoint::new(0xd800).unwrap(), 56);

        assert!(narrow.lines.len() > wide.lines.len());
        assert_eq!(
            narrow
                .lines
                .iter()
                .filter(|line| line.to_string().contains("Not available"))
                .count(),
            6
        );
    }

    #[test]
    fn preview_layout_uses_fixed_details_width_for_viewport_metrics() {
        let mut state = fixtures::startup();
        update(
            &mut state,
            Action::UpdateGlyphPreview(GlyphPreviewUpdate::Configure {
                availability: GraphicsAvailability::Available(GraphicsProtocol::Kitty),
                image_id: None,
            }),
        );

        for (area, viewport_height, document_height) in [
            (Rect::new(0, 0, 60, 16), 12, 31),
            (Rect::new(0, 0, 99, 16), 12, 31),
            (Rect::new(0, 0, 100, 30), 26, 31),
            (Rect::new(0, 0, 140, 40), 36, 31),
        ] {
            let metrics = viewport_metrics(area, &state);
            assert_eq!(metrics.viewport_height, viewport_height);
            assert_eq!(metrics.document_height, document_height, "area: {area:?}");
            assert_eq!(metrics.field_ranges.len(), 22);
        }
    }

    #[test]
    fn widening_the_preview_layout_clamps_the_inspector_offset() {
        let mut state = fixtures::startup();
        update(
            &mut state,
            Action::UpdateGlyphPreview(GlyphPreviewUpdate::Configure {
                availability: GraphicsAvailability::Available(GraphicsProtocol::Kitty),
                image_id: None,
            }),
        );
        let narrow = viewport_metrics(Rect::new(0, 0, 100, 16), &state);
        update(
            &mut state,
            Action::ResizeInspectorViewport {
                viewport_height: narrow.viewport_height,
                document_height: narrow.document_height,
                field_ranges: narrow.field_ranges,
            },
        );
        update(
            &mut state,
            Action::MoveInspector(crate::inspector::InspectorMove::Last),
        );
        assert_eq!(state.inspector().offset(), 19);

        let wide = viewport_metrics(Rect::new(0, 0, 140, 40), &state);
        update(
            &mut state,
            Action::ResizeInspectorViewport {
                viewport_height: wide.viewport_height,
                document_height: wide.document_height,
                field_ranges: wide.field_ranges,
            },
        );
        assert_eq!(state.inspector().offset(), 0);
    }
}
