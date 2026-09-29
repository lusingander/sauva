use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Padding},
};

use crate::{
    inspector::InspectorDetails,
    preview::GlyphPreviewState,
    ui::glyph_preview,
    ui::key_value::{self, KeyValue},
    ui::layout,
    ui::theme::ColorTheme,
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

    key_value::render(
        frame,
        area,
        " Selection Preview ",
        LABEL_WIDTH,
        &entries,
        color_theme,
    );
    if let Some(glyph) = glyph_area_for_entries(area, &entries) {
        glyph_preview::render_image_only(frame, glyph, glyph_preview_state, color_theme);
    }
}

pub fn glyph_area(area: Rect, code_point: CodePoint) -> Option<Rect> {
    let details = SelectionDetails::new(code_point);
    glyph_area_for_entries(area, &details.entries())
}

fn glyph_area_for_entries(area: Rect, entries: &[KeyValue<'_>]) -> Option<Rect> {
    let content = selection_block().inner(area);
    let details_height = key_value::required_height(content.width, LABEL_WIDTH, entries);
    let glyph_y = content
        .y
        .saturating_add(details_height)
        .saturating_add(GLYPH_GAP_HEIGHT);
    let glyph_height = content.bottom().saturating_sub(glyph_y);
    (glyph_height >= MINIMUM_GLYPH_HEIGHT).then(|| {
        layout::centered_glyph_area(Rect::new(content.x, glyph_y, content.width, glyph_height))
    })
}

fn selection_block() -> Block<'static> {
    Block::bordered()
        .title(" Selection Preview ")
        .padding(Padding::horizontal(1))
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
            Some(Rect::new(62, 12, 36, 16))
        );
    }

    #[test]
    fn omits_the_glyph_when_too_little_height_remains() {
        let area = Rect::new(60, 3, 40, 12);
        let code_point = CodePoint::new(0x0041).unwrap();

        assert_eq!(glyph_area(area, code_point), None);
    }
}
