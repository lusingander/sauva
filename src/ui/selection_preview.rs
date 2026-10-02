use ratatui::{Frame, layout::Rect, widgets::Paragraph};

use crate::{
    inspector::InspectorDetails,
    preview::GlyphPreviewState,
    sequence::SequenceState,
    ui::glyph_preview,
    ui::key_value::{self, KeyValue},
    ui::layout,
    ui::theme::ColorTheme,
    ui::workspace,
    unicode::{CodePoint, NameAlias},
};

const LABEL_WIDTH: u16 = 17;
const GLYPH_GAP_HEIGHT: u16 = 1;
const MINIMUM_GLYPH_HEIGHT: u16 = 8;
const PREFERRED_SELECTION_HEIGHT: u16 = 9;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    code_point: CodePoint,
    matched_alias: Option<NameAlias>,
    glyph_preview_state: &GlyphPreviewState,
    color_theme: &ColorTheme,
) {
    let details = SelectionDetails::new(code_point, matched_alias);
    render_details(frame, area, details, glyph_preview_state, color_theme);
}

pub fn render_sequence(
    frame: &mut Frame,
    area: Rect,
    sequence: &SequenceState,
    glyph_preview_state: &GlyphPreviewState,
    color_theme: &ColorTheme,
) {
    let details = SelectionDetails::for_sequence(sequence);
    render_details(frame, area, details, glyph_preview_state, color_theme);
}

fn render_details(
    frame: &mut Frame,
    area: Rect,
    details: SelectionDetails,
    glyph_preview_state: &GlyphPreviewState,
    color_theme: &ColorTheme,
) {
    let entries = details.entries();

    let content = workspace::render_rail_heading(frame, area, "Selection", None, color_theme);
    let details_height =
        key_value::required_height(content.width, LABEL_WIDTH, &entries).min(content.height);
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

pub fn glyph_area(
    area: Rect,
    code_point: CodePoint,
    matched_alias: Option<NameAlias>,
) -> Option<Rect> {
    let details = SelectionDetails::new(code_point, matched_alias);
    glyph_area_for_entries(area, &details.entries())
}

pub fn glyph_area_for_sequence(area: Rect, sequence: &SequenceState) -> Option<Rect> {
    let details = SelectionDetails::for_sequence(sequence);
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
    let section = workspace::rail_section(area);
    let content = section.content;
    let details_height = key_value::required_height(content.width, LABEL_WIDTH, entries)
        .max(PREFERRED_SELECTION_HEIGHT);
    let divider_y = content.y.saturating_add(details_height);
    let heading_y = divider_y.saturating_add(GLYPH_GAP_HEIGHT);
    let glyph_y = heading_y.saturating_add(1);
    let glyph_height = content.bottom().saturating_sub(glyph_y);
    (glyph_height >= MINIMUM_GLYPH_HEIGHT).then(|| InlineGlyphLayout {
        divider: Rect::new(section.heading.x, divider_y, section.heading.width, 1),
        heading: Rect::new(section.heading.x, heading_y, section.heading.width, 1),
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
    matched_alias: Option<NameAlias>,
    grapheme: Option<String>,
}

impl SelectionDetails {
    fn new(code_point: CodePoint, matched_alias: Option<NameAlias>) -> Self {
        let details = InspectorDetails::for_code_point(code_point);
        let category = details.general_category();
        let category = format!("{} — {}", category.abbreviation(), category.name());
        let code_point = details.code_point().to_string();
        Self {
            details,
            category,
            code_point,
            matched_alias,
            grapheme: None,
        }
    }

    fn for_sequence(sequence: &SequenceState) -> Self {
        let mut details = Self::new(sequence.selected(), None);
        let point = &sequence.code_points()[sequence.selected_index()];
        let graphemes = sequence.analysis().graphemes();
        let grapheme = &graphemes[point.grapheme_index()];
        details.grapheme = Some(format!(
            "{}/{} · member {}/{}",
            point.grapheme_index() + 1,
            graphemes.len(),
            point.index_in_grapheme() + 1,
            grapheme.code_point_range().len(),
        ));
        details
    }

    fn entries(&self) -> Vec<KeyValue<'_>> {
        let mut entries = vec![
            KeyValue::new("Character", self.details.character().as_str()),
            KeyValue::new("Code Point", &self.code_point),
            KeyValue::new("Primary Name", self.details.primary_name()),
        ];
        if let Some(alias) = self.matched_alias {
            entries.extend([
                KeyValue::new("Matched Alias", alias.name()),
                KeyValue::new("Alias Type", alias.kind().label()),
            ]);
        }
        if let Some(grapheme) = &self.grapheme {
            entries.push(KeyValue::new("Grapheme", grapheme));
        }
        entries.extend([
            KeyValue::new("Block", self.details.block()),
            KeyValue::new("General Category", &self.category),
        ]);
        entries
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::sequence::SequenceMove;
    use crate::unicode::UnicodeDatabase;

    #[test]
    fn reserves_selection_space_before_placing_the_glyph() {
        let area = Rect::new(60, 2, 40, 27);
        let code_point = CodePoint::new(0x0041).unwrap();

        assert_eq!(
            glyph_area(area, code_point, None),
            Some(Rect::new(63, 14, 36, 14))
        );
    }

    #[test]
    fn keeps_the_glyph_position_stable_for_common_wrapping_differences() {
        let area = Rect::new(60, 2, 40, 27);

        assert_eq!(
            glyph_area(area, CodePoint::new(0x0041).unwrap(), None),
            glyph_area(area, CodePoint::new(0x2192).unwrap(), None)
        );
    }

    #[test]
    fn omits_the_glyph_when_too_little_height_remains() {
        let area = Rect::new(60, 3, 40, 12);
        let code_point = CodePoint::new(0x0041).unwrap();

        assert_eq!(glyph_area(area, code_point, None), None);
    }

    #[test]
    fn includes_the_matched_alias_and_reserves_its_height() {
        let area = Rect::new(60, 2, 40, 27);
        let code_point = CodePoint::new(0x01a2).unwrap();
        let alias = UnicodeDatabase::lookup(code_point).name_aliases()[0];
        let details = SelectionDetails::new(code_point, Some(alias));
        let document = key_value::Document::for_entries(
            36,
            LABEL_WIDTH,
            &details.entries(),
            &ColorTheme::default(),
        )
        .lines
        .into_iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>()
        .join("\n");

        assert!(document.contains("Matched Alias"));
        assert!(document.contains("LATIN CAPITAL LETTER GHA"));
        assert!(
            document
                .lines()
                .any(|line| line.contains("Alias Type") && line.contains("correction"))
        );

        let ordinary = glyph_area(area, code_point, None).unwrap();
        let matched = glyph_area(area, code_point, Some(alias)).unwrap();
        assert!(matched.y > ordinary.y);
        assert!(matched.height < ordinary.height);
    }

    #[rstest]
    #[case(0, "1/3 · member 1/2")]
    #[case(1, "1/3 · member 2/2")]
    #[case(2, "2/3 · member 1/1")]
    #[case(3, "3/3 · member 1/3")]
    #[case(4, "3/3 · member 2/3")]
    #[case(5, "3/3 · member 3/3")]
    fn sequence_details_show_the_selected_grapheme_and_member(
        #[case] selected_index: usize,
        #[case] expected: &str,
    ) {
        let mut sequence = SequenceState::new("A\u{0301} 👩‍💻".to_owned());
        for _ in 0..selected_index {
            sequence.move_selection(SequenceMove::Next);
        }
        let details = SelectionDetails::for_sequence(&sequence);
        let document = key_value::Document::for_entries(
            36,
            LABEL_WIDTH,
            &details.entries(),
            &ColorTheme::default(),
        )
        .lines
        .into_iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>()
        .join("\n");

        assert!(
            document
                .lines()
                .any(|line| line.contains("Grapheme") && line.contains(expected))
        );
        assert!(document.contains(&sequence.selected().to_string()));
    }

    #[test]
    fn ordinary_selection_details_do_not_include_a_grapheme_coordinate() {
        let details = SelectionDetails::new(CodePoint::from('A'), None);

        assert!(details.grapheme.is_none());
        assert_eq!(details.entries().len(), 5);
    }

    #[test]
    fn sequence_glyph_area_accounts_for_wrapped_coordinates() {
        let mut sequence =
            SequenceState::new(format!("{}A{}", "X".repeat(10), "\u{0301}".repeat(11)));
        sequence.move_selection(SequenceMove::Last);
        let area = Rect::new(60, 2, 40, 27);
        let ordinary = glyph_area(area, sequence.selected(), None).unwrap();
        let with_coordinate = glyph_area_for_sequence(area, &sequence).unwrap();

        assert!(with_coordinate.y > ordinary.y);
        assert!(with_coordinate.height < ordinary.height);
        assert_eq!(
            glyph_area_for_sequence(Rect::new(60, 2, 40, 16), &sequence),
            None
        );
    }
}
