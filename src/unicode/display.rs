use std::fmt;

use crate::unicode::{CodePoint, CodePointStructure};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayKind {
    Graphic,
    CombiningMark,
    Space,
    LineSeparator,
    ParagraphSeparator,
    Control,
    Format,
    DefaultIgnorable,
    Unassigned,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayRepresentation(String);

impl DisplayRepresentation {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DisplayRepresentation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl CodePoint {
    pub fn display_representation(self, kind: DisplayKind) -> DisplayRepresentation {
        let representation = match self.structure() {
            CodePointStructure::Surrogate => "<SURROGATE>".to_owned(),
            CodePointStructure::PrivateUse => "<PRIVATE USE>".to_owned(),
            CodePointStructure::Noncharacter => "<NONCHARACTER>".to_owned(),
            CodePointStructure::Other => self.to_char().map_or_else(
                || "<SURROGATE>".to_owned(),
                |character| display_scalar(character, kind),
            ),
        };

        DisplayRepresentation(representation)
    }
}

fn display_scalar(character: char, kind: DisplayKind) -> String {
    match kind {
        DisplayKind::Graphic => character.to_string(),
        DisplayKind::CombiningMark => format!("\u{25cc}{character}"),
        DisplayKind::Space => "<SPACE>".to_owned(),
        DisplayKind::LineSeparator => "<LINE SEPARATOR>".to_owned(),
        DisplayKind::ParagraphSeparator => "<PARAGRAPH SEPARATOR>".to_owned(),
        DisplayKind::Control => "<CONTROL>".to_owned(),
        DisplayKind::Format => "<FORMAT>".to_owned(),
        DisplayKind::DefaultIgnorable => "<DEFAULT IGNORABLE>".to_owned(),
        DisplayKind::Unassigned => "<UNASSIGNED>".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(0x0041, DisplayKind::Graphic, "A")]
    #[case(0x0301, DisplayKind::CombiningMark, "\u{25cc}\u{0301}")]
    #[case(0x0020, DisplayKind::Space, "<SPACE>")]
    #[case(0x2028, DisplayKind::LineSeparator, "<LINE SEPARATOR>")]
    #[case(0x2029, DisplayKind::ParagraphSeparator, "<PARAGRAPH SEPARATOR>")]
    #[case(0x000a, DisplayKind::Control, "<CONTROL>")]
    #[case(0x200d, DisplayKind::Format, "<FORMAT>")]
    #[case(0x202e, DisplayKind::Format, "<FORMAT>")]
    #[case(0x115f, DisplayKind::DefaultIgnorable, "<DEFAULT IGNORABLE>")]
    #[case(0x0378, DisplayKind::Unassigned, "<UNASSIGNED>")]
    fn represents_data_dependent_display_kinds(
        #[case] value: u32,
        #[case] kind: DisplayKind,
        #[case] expected: &str,
    ) {
        let representation = CodePoint::new(value).unwrap().display_representation(kind);

        assert_eq!(representation.as_str(), expected);
        assert_eq!(representation.to_string(), expected);
    }

    #[rstest]
    #[case(0xd800, "<SURROGATE>")]
    #[case(0xe000, "<PRIVATE USE>")]
    #[case(0xfdd0, "<NONCHARACTER>")]
    fn structural_labels_override_a_graphic_display_kind(
        #[case] value: u32,
        #[case] expected: &str,
    ) {
        let representation = CodePoint::new(value)
            .unwrap()
            .display_representation(DisplayKind::Graphic);

        assert_eq!(representation.as_str(), expected);
        assert!(representation.as_str().is_ascii());
    }
}
