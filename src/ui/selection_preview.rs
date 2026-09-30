use ratatui::{Frame, layout::Rect, widgets::Paragraph};

use crate::{
    inspector::InspectorDetails,
    preview::GlyphPreviewState,
    ui::glyph_preview,
    ui::key_value::{self, KeyValue},
    ui::layout,
    ui::theme::ColorTheme,
    ui::workspace,
    unicode::CodePoint,
};

const LABEL_WIDTH: u16 = 17;
const GLYPH_GAP_HEIGHT: u16 = 1;
const MINIMUM_GLYPH_HEIGHT: u16 = 8;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    code_point: CodePoint,
    glyph_preview_state: &GlyphPreviewState,
    color_theme: &ColorTheme,
) {
    let details = SelectionDetails::new(code_point);
    let entries = details.entries();

    let content = workspace::render_rail_heading(frame, area, "Selection", None, color_theme);
    let details_height = key_value::required_height(content.width, LABEL_WIDTH, &entries);
    key_value::render_entries(
        frame,
        Rect::new(content.x, content.y, content.width, details_height),
        LABEL_WIDTH,
        &entries,
        color_theme,
    );
    if let Some(glyph_layout) = glyph_layout_for_entries(area, &entries) {
        workspace::render_divider(frame, glyph_layout.divider, color_theme);
        frame.render_widget(
            Paragraph::new("Glyph").style(color_theme.heading_style()),
            glyph_layout.heading,
        );
        glyph_preview::render_image_only(
            frame,
            glyph_layout.glyph,
            glyph_preview_state,
            color_theme,
        );
    }
}

pub fn glyph_area(area: Rect, code_point: CodePoint) -> Option<Rect> {
    let details = SelectionDetails::new(code_point);
    glyph_area_for_entries(area, &details.entries())
}

fn glyph_area_for_entries(area: Rect, entries: &[KeyValue<'_>]) -> Option<Rect> {
    glyph_layout_for_entries(area, entries).map(|layout| layout.glyph)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct InlineGlyphLayout {
    divider: Rect,
    heading: Rect,
    glyph: Rect,
}

fn glyph_layout_for_entries(area: Rect, entries: &[KeyValue<'_>]) -> Option<InlineGlyphLayout> {
    let content = workspace::rail_section(area).content;
    let details_height = key_value::required_height(content.width, LABEL_WIDTH, entries);
    let divider_y = content.y.saturating_add(details_height);
    let heading_y = divider_y.saturating_add(GLYPH_GAP_HEIGHT);
    let glyph_y = heading_y.saturating_add(1);
    let glyph_height = content.bottom().saturating_sub(glyph_y);
    (glyph_height >= MINIMUM_GLYPH_HEIGHT).then(|| InlineGlyphLayout {
        divider: Rect::new(content.x, divider_y, content.width, 1),
        heading: Rect::new(content.x, heading_y, content.width, 1),
        glyph: layout::centered_glyph_area(Rect::new(
            content.x,
            glyph_y,
            content.width,
            glyph_height,
        )),
    })
}

struct SelectionDetails {
    details: InspectorDetails,
    category: String,
    code_point: String,
}

impl SelectionDetails {
    fn new(code_point: CodePoint) -> Self {
        let details = InspectorDetails::for_code_point(code_point);
        let category = details.general_category();
        let category = format!("{} — {}", category.abbreviation(), category.name());
        let code_point = details.code_point().to_string();
        Self {
            details,
            category,
            code_point,
        }
    }

    fn entries(&self) -> [KeyValue<'_>; 5] {
        [
            KeyValue::new("Character", self.details.character().as_str()),
            KeyValue::new("Code Point", &self.code_point),
            KeyValue::new("Primary Name", self.details.primary_name()),
            KeyValue::new("Block", self.details.block()),
            KeyValue::new("General Category", &self.category),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_the_space_below_uncompressed_details_for_the_glyph() {
        let area = Rect::new(60, 3, 40, 26);
        let code_point = CodePoint::new(0x0041).unwrap();

        assert_eq!(
            glyph_area(area, code_point),
            Some(Rect::new(62, 13, 36, 15))
        );
    }

    #[test]
    fn omits_the_glyph_when_too_little_height_remains() {
        let area = Rect::new(60, 3, 40, 12);
        let code_point = CodePoint::new(0x0041).unwrap();

        assert_eq!(glyph_area(area, code_point), None);
    }
}
