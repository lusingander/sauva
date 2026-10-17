use std::ops::Range;

use crate::unicode::{
    BidiClass, CanonicalCombiningClass, CaseMapping, CodePoint, Decomposition, DecompositionType,
    DisplayRepresentation, EastAsianWidth, GeneralCategory, NameAlias, NumericType, Plane,
    UnicodeDatabase, UnicodeRecord, UnicodeScalarEncoding,
};

const UNASSIGNED: &str = "Unassigned";
const NO_BLOCK: &str = "No Block";
pub const UNAVAILABLE_ENCODING: &str = "Not available — not a Unicode scalar value";
const NONE: &str = "None";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum InspectorDisplayMode {
    #[default]
    Relevant,
    All,
}

impl InspectorDisplayMode {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Relevant => "Relevant",
            Self::All => "All",
        }
    }

    pub const fn toggle(self) -> Self {
        match self {
            Self::Relevant => Self::All,
            Self::All => Self::Relevant,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectorMove {
    PreviousField,
    NextField,
    PreviousGroup,
    NextGroup,
    PageBackward,
    PageForward,
    First,
    Last,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InspectorGroup {
    pub first_field: usize,
    pub heading_line: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InspectorState {
    display_mode: InspectorDisplayMode,
    selected_index: usize,
    offset: usize,
    viewport_height: usize,
    document_height: usize,
    field_ranges: Vec<Range<usize>>,
    groups: Vec<InspectorGroup>,
    align_group: bool,
}

impl InspectorState {
    pub const fn display_mode(&self) -> InspectorDisplayMode {
        self.display_mode
    }

    pub fn toggle_display_mode(&mut self, code_point: CodePoint) {
        let previous = InspectorField::for_code_point(code_point, self.display_mode);
        self.display_mode = self.display_mode.toggle();
        let fields = InspectorField::for_code_point(code_point, self.display_mode);
        self.follow_fields(&previous, &fields);
    }

    pub fn follow_code_point(&mut self, previous: CodePoint, code_point: CodePoint) {
        let previous = InspectorField::for_code_point(previous, self.display_mode);
        let fields = InspectorField::for_code_point(code_point, self.display_mode);
        self.follow_fields(&previous, &fields);
    }

    fn follow_fields(&mut self, previous: &[InspectorField], fields: &[InspectorField]) {
        let Some(selected) = previous.get(self.selected_index) else {
            self.selected_index = 0;
            return;
        };
        // Field IDs follow display order. Prefer the same section, then the
        // nearest property, choosing the following property when distances tie.
        let selected_order = selected.id() as usize;
        self.selected_index = fields
            .iter()
            .enumerate()
            .min_by_key(|(_, field)| {
                let order = field.id() as usize;
                (
                    field.section() != selected.section(),
                    order.abs_diff(selected_order),
                    order < selected_order,
                )
            })
            .map_or(0, |(index, _)| index);
    }

    #[cfg(test)]
    pub const fn offset(&self) -> usize {
        self.offset
    }

    pub const fn selected_index(&self) -> usize {
        self.selected_index
    }

    pub fn visible_range(&self) -> Range<usize> {
        let end = self
            .offset
            .saturating_add(self.viewport_height)
            .min(self.document_height);
        self.offset..end
    }

    pub fn resize_viewport(
        &mut self,
        viewport_height: usize,
        document_height: usize,
        field_ranges: Vec<Range<usize>>,
        groups: Vec<InspectorGroup>,
    ) {
        self.viewport_height = viewport_height;
        self.document_height = document_height;
        self.field_ranges = field_ranges;
        self.groups = groups;
        self.selected_index = self
            .selected_index
            .min(self.field_ranges.len().saturating_sub(1));
        self.ensure_selection_visible();
    }

    pub fn move_selection(&mut self, movement: InspectorMove) {
        if self.field_ranges.is_empty() {
            return;
        }

        let last = self.field_ranges.len() - 1;
        if !matches!(
            movement,
            InspectorMove::PreviousGroup | InspectorMove::NextGroup
        ) {
            self.align_group = false;
        }
        self.selected_index = match movement {
            InspectorMove::PreviousField => self.selected_index.saturating_sub(1),
            InspectorMove::NextField => self.selected_index.saturating_add(1).min(last),
            InspectorMove::PreviousGroup | InspectorMove::NextGroup => {
                let Some(current) = self
                    .groups
                    .partition_point(|group| group.first_field <= self.selected_index)
                    .checked_sub(1)
                else {
                    return;
                };
                let target = if movement == InspectorMove::NextGroup {
                    Some(current + 1)
                } else {
                    current.checked_sub(1)
                };
                let Some(group) = target.and_then(|index| self.groups.get(index)) else {
                    return;
                };
                self.align_group = true;
                group.first_field
            }
            InspectorMove::PageBackward => self.page_backward(),
            InspectorMove::PageForward => self.page_forward(),
            InspectorMove::First => 0,
            InspectorMove::Last => last,
        };
        self.ensure_selection_visible();
    }

    const fn maximum_offset(&self) -> usize {
        if self.viewport_height == 0 {
            0
        } else {
            self.document_height.saturating_sub(self.viewport_height)
        }
    }

    fn page_backward(&self) -> usize {
        let target = self.field_ranges[self.selected_index]
            .start
            .saturating_sub(self.viewport_height.max(1));
        self.field_ranges[..self.selected_index]
            .iter()
            .rposition(|range| range.start <= target)
            .unwrap_or(0)
    }

    fn page_forward(&self) -> usize {
        let target = self.field_ranges[self.selected_index]
            .start
            .saturating_add(self.viewport_height.max(1));
        self.field_ranges
            .iter()
            .enumerate()
            .skip(self.selected_index + 1)
            .find_map(|(index, range)| (range.start >= target).then_some(index))
            .unwrap_or(self.field_ranges.len() - 1)
    }

    fn ensure_selection_visible(&mut self) {
        let Some(selected) = self.field_ranges.get(self.selected_index) else {
            self.offset = 0;
            return;
        };
        if self.viewport_height == 0 {
            self.offset = 0;
            return;
        }
        if self.align_group
            && let Some(group) = self
                .groups
                .iter()
                .find(|group| group.first_field == self.selected_index)
        {
            self.offset = if self.viewport_height > 1 {
                group.heading_line
            } else {
                selected.start
            }
            .min(self.maximum_offset());
            return;
        }
        if self.selected_index == 0 && selected.end <= self.viewport_height {
            self.offset = 0;
            return;
        }

        if selected.start < self.offset {
            self.offset = selected.start;
        } else if selected.end > self.offset.saturating_add(self.viewport_height) {
            self.offset = if selected.len() >= self.viewport_height {
                selected.start
            } else {
                selected.end.saturating_sub(self.viewport_height)
            };
        }
        self.offset = self.offset.min(self.maximum_offset());
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectorDetails {
    record: UnicodeRecord,
    character: DisplayRepresentation,
    primary_name: &'static str,
    plane: Plane,
    block: &'static str,
    age: &'static str,
}

impl InspectorDetails {
    pub fn for_code_point(code_point: CodePoint) -> Self {
        let record = UnicodeDatabase::lookup(code_point);

        Self {
            record,
            character: record.display_representation(),
            primary_name: record.primary_name_or_fallback(),
            plane: Plane::for_code_point(code_point),
            block: record.block().unwrap_or(NO_BLOCK),
            age: record.age().unwrap_or(UNASSIGNED),
        }
    }

    pub fn character(&self) -> &DisplayRepresentation {
        &self.character
    }

    pub const fn code_point(&self) -> CodePoint {
        self.record.code_point()
    }

    pub const fn primary_name(&self) -> &'static str {
        self.primary_name
    }

    pub const fn plane(&self) -> Plane {
        self.plane
    }

    pub const fn block(&self) -> &'static str {
        self.block
    }

    pub const fn general_category(&self) -> GeneralCategory {
        self.record.general_category()
    }

    pub const fn script(&self) -> &'static str {
        self.record.script()
    }

    pub const fn age(&self) -> &'static str {
        self.age
    }

    pub const fn is_default_ignorable(&self) -> bool {
        self.record.is_default_ignorable()
    }

    pub const fn name_aliases(&self) -> &'static [NameAlias] {
        self.record.name_aliases()
    }

    pub const fn east_asian_width(&self) -> EastAsianWidth {
        self.record.east_asian_width()
    }

    pub const fn canonical_combining_class(&self) -> CanonicalCombiningClass {
        self.record.canonical_combining_class()
    }

    pub const fn bidi_class(&self) -> BidiClass {
        self.record.bidi_class()
    }

    pub const fn encoding(&self) -> Option<UnicodeScalarEncoding> {
        self.record.encoding()
    }

    pub const fn decomposition(&self) -> Option<Decomposition> {
        self.record.decomposition()
    }

    pub const fn numeric_type(&self) -> NumericType {
        self.record.numeric_type()
    }

    pub fn case_folding(&self) -> CaseMapping {
        self.record.case_folding()
    }

    pub fn uppercase_mapping(&self) -> CaseMapping {
        self.record.uppercase_mapping()
    }

    pub fn lowercase_mapping(&self) -> CaseMapping {
        self.record.lowercase_mapping()
    }

    pub fn titlecase_mapping(&self) -> CaseMapping {
        self.record.titlecase_mapping()
    }

    pub const fn numeric_value(&self) -> Option<&'static str> {
        self.record.numeric_value()
    }

    pub const fn unicode_version(&self) -> &'static str {
        UnicodeDatabase::version()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectorSection {
    Identity,
    Classification,
    Case,
    Numeric,
    Encoding,
    Normalization,
    Data,
}

impl InspectorSection {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Identity => "Identity",
            Self::Classification => "Classification",
            Self::Case => "Case",
            Self::Numeric => "Numeric",
            Self::Encoding => "Encoding",
            Self::Normalization => "Normalization",
            Self::Data => "Data",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InspectorFieldId {
    Character,
    CodePoint,
    PrimaryName,
    Aliases,
    Plane,
    Block,
    GeneralCategory,
    Script,
    Age,
    EastAsianWidth,
    CanonicalCombiningClass,
    BidiClass,
    DefaultIgnorable,
    UppercaseMapping,
    LowercaseMapping,
    TitlecaseMapping,
    CaseFolding,
    NumericType,
    NumericValue,
    Utf8,
    Utf16,
    HtmlDecimal,
    HtmlHex,
    RustChar,
    UnicodeEscape,
    DecompositionType,
    Decomposition,
    UnicodeVersion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectorField {
    id: InspectorFieldId,
    section: InspectorSection,
    label: &'static str,
    values: Vec<String>,
    copy_value: Option<String>,
}

impl InspectorField {
    pub fn for_code_point(code_point: CodePoint, mode: InspectorDisplayMode) -> Vec<Self> {
        let details = InspectorDetails::for_code_point(code_point);
        let mut fields = vec![
            Self::new(
                InspectorFieldId::Character,
                InspectorSection::Identity,
                "Character",
                vec![details.character().as_str().to_owned()],
                code_point.to_char().map(|character| character.to_string()),
            ),
            Self::copyable(
                InspectorFieldId::CodePoint,
                InspectorSection::Identity,
                "Code Point",
                [details.code_point().to_string()],
            ),
            Self::copyable(
                InspectorFieldId::PrimaryName,
                InspectorSection::Identity,
                "Primary Name",
                [details.primary_name().to_owned()],
            ),
        ];

        let aliases = details.name_aliases();
        if aliases.is_empty() {
            fields.push(Self::unavailable(
                InspectorFieldId::Aliases,
                InspectorSection::Identity,
                "Aliases",
                NONE,
            ));
        } else {
            fields.push(Self::copyable(
                InspectorFieldId::Aliases,
                InspectorSection::Identity,
                "Aliases",
                aliases
                    .iter()
                    .map(|alias| format!("{} — {}", alias.name(), alias.kind().label())),
            ));
        }

        let plane = details.plane();
        fields.extend([
            Self::copyable(
                InspectorFieldId::Plane,
                InspectorSection::Identity,
                "Plane",
                [plane.name().map_or_else(
                    || plane.number().to_string(),
                    |name| format!("{} — {name}", plane.number()),
                )],
            ),
            Self::copyable(
                InspectorFieldId::Block,
                InspectorSection::Identity,
                "Block",
                [details.block().to_owned()],
            ),
        ]);

        let category = details.general_category();
        let east_asian_width = details.east_asian_width();
        let combining_class = details.canonical_combining_class();
        let bidi_class = details.bidi_class();
        fields.extend([
            Self::copyable(
                InspectorFieldId::GeneralCategory,
                InspectorSection::Classification,
                "General Category",
                [format!("{} — {}", category.abbreviation(), category.name())],
            ),
            Self::copyable(
                InspectorFieldId::Script,
                InspectorSection::Classification,
                "Script",
                [details.script().to_owned()],
            ),
            Self::copyable(
                InspectorFieldId::Age,
                InspectorSection::Classification,
                "Age",
                [details.age().to_owned()],
            ),
            Self::copyable(
                InspectorFieldId::EastAsianWidth,
                InspectorSection::Classification,
                "East Asian Width",
                [format!(
                    "{} — {}",
                    east_asian_width.abbreviation(),
                    east_asian_width.name()
                )],
            ),
            Self::copyable(
                InspectorFieldId::CanonicalCombiningClass,
                InspectorSection::Classification,
                "Canonical Combining Class",
                [format!(
                    "{} — {}",
                    combining_class.value(),
                    combining_class.name()
                )],
            ),
            Self::copyable(
                InspectorFieldId::BidiClass,
                InspectorSection::Classification,
                "Bidi Class",
                [format!(
                    "{} — {}",
                    bidi_class.abbreviation(),
                    bidi_class.name()
                )],
            ),
            Self::copyable(
                InspectorFieldId::DefaultIgnorable,
                InspectorSection::Classification,
                "Default Ignorable",
                [if details.is_default_ignorable() {
                    "Yes".to_owned()
                } else {
                    "No".to_owned()
                }],
            ),
        ]);

        for (id, label, mapping) in [
            (
                InspectorFieldId::UppercaseMapping,
                "Uppercase Mapping",
                details.uppercase_mapping(),
            ),
            (
                InspectorFieldId::LowercaseMapping,
                "Lowercase Mapping",
                details.lowercase_mapping(),
            ),
            (
                InspectorFieldId::TitlecaseMapping,
                "Titlecase Mapping",
                details.titlecase_mapping(),
            ),
            (
                InspectorFieldId::CaseFolding,
                "Case Folding",
                details.case_folding(),
            ),
        ] {
            fields.push(Self::code_point_mapping(
                id,
                InspectorSection::Case,
                label,
                mapping.mapping(),
            ));
        }

        fields.push(Self::copyable(
            InspectorFieldId::NumericType,
            InspectorSection::Numeric,
            "Numeric Type",
            [details.numeric_type().name().to_owned()],
        ));
        fields.push(if let Some(value) = details.numeric_value() {
            Self::copyable(
                InspectorFieldId::NumericValue,
                InspectorSection::Numeric,
                "Numeric Value",
                [value.to_owned()],
            )
        } else {
            Self::unavailable(
                InspectorFieldId::NumericValue,
                InspectorSection::Numeric,
                "Numeric Value",
                "NaN",
            )
        });

        if let Some(encoding) = details.encoding() {
            fields.extend([
                Self::copyable(
                    InspectorFieldId::Utf8,
                    InspectorSection::Encoding,
                    "UTF-8",
                    [encoding.utf8()],
                ),
                Self::copyable(
                    InspectorFieldId::Utf16,
                    InspectorSection::Encoding,
                    "UTF-16",
                    [encoding.utf16()],
                ),
                Self::copyable(
                    InspectorFieldId::HtmlDecimal,
                    InspectorSection::Encoding,
                    "HTML Decimal",
                    [encoding.html_decimal()],
                ),
                Self::copyable(
                    InspectorFieldId::HtmlHex,
                    InspectorSection::Encoding,
                    "HTML Hex",
                    [encoding.html_hex()],
                ),
                Self::copyable(
                    InspectorFieldId::RustChar,
                    InspectorSection::Encoding,
                    "Rust char",
                    [encoding.rust_char()],
                ),
                Self::copyable(
                    InspectorFieldId::UnicodeEscape,
                    InspectorSection::Encoding,
                    "Unicode Escape",
                    [encoding.unicode_escape()],
                ),
            ]);
        } else {
            for (id, label) in [
                (InspectorFieldId::Utf8, "UTF-8"),
                (InspectorFieldId::Utf16, "UTF-16"),
                (InspectorFieldId::HtmlDecimal, "HTML Decimal"),
                (InspectorFieldId::HtmlHex, "HTML Hex"),
                (InspectorFieldId::RustChar, "Rust char"),
                (InspectorFieldId::UnicodeEscape, "Unicode Escape"),
            ] {
                fields.push(Self::unavailable(
                    id,
                    InspectorSection::Encoding,
                    label,
                    UNAVAILABLE_ENCODING,
                ));
            }
        }

        if let Some(decomposition) = details.decomposition() {
            let decomposition_type = decomposition.decomposition_type();
            fields.push(Self::copyable(
                InspectorFieldId::DecompositionType,
                InspectorSection::Normalization,
                "Decomposition Type",
                [match decomposition_type {
                    DecompositionType::Canonical => decomposition_type.label().to_owned(),
                    DecompositionType::Compatibility(tag) => {
                        format!("{} — {tag}", decomposition_type.label())
                    }
                }],
            ));
            fields.push(Self::code_point_mapping(
                InspectorFieldId::Decomposition,
                InspectorSection::Normalization,
                "Decomposition",
                decomposition.mapping(),
            ));
        } else {
            fields.extend([
                Self::unavailable(
                    InspectorFieldId::DecompositionType,
                    InspectorSection::Normalization,
                    "Decomposition Type",
                    NONE,
                ),
                Self::unavailable(
                    InspectorFieldId::Decomposition,
                    InspectorSection::Normalization,
                    "Decomposition",
                    NONE,
                ),
            ]);
        }

        fields.push(Self::copyable(
            InspectorFieldId::UnicodeVersion,
            InspectorSection::Data,
            "Unicode Version",
            [details.unicode_version().to_owned()],
        ));
        if mode == InspectorDisplayMode::Relevant {
            fields.retain(|field| field.is_relevant(&details));
        }
        fields
    }

    fn is_relevant(&self, details: &InspectorDetails) -> bool {
        match self.id {
            InspectorFieldId::Aliases => !details.name_aliases().is_empty(),
            InspectorFieldId::CanonicalCombiningClass => {
                details.canonical_combining_class().value() != 0
                    || matches!(
                        details.general_category(),
                        GeneralCategory::NonspacingMark
                            | GeneralCategory::SpacingMark
                            | GeneralCategory::EnclosingMark
                    )
            }
            InspectorFieldId::DefaultIgnorable => details.is_default_ignorable(),
            InspectorFieldId::UppercaseMapping => !details.uppercase_mapping().is_identity(),
            InspectorFieldId::LowercaseMapping => !details.lowercase_mapping().is_identity(),
            InspectorFieldId::TitlecaseMapping => !details.titlecase_mapping().is_identity(),
            InspectorFieldId::CaseFolding => !details.case_folding().is_identity(),
            InspectorFieldId::NumericType | InspectorFieldId::NumericValue => {
                details.numeric_type() != NumericType::None
            }
            InspectorFieldId::Utf8
            | InspectorFieldId::Utf16
            | InspectorFieldId::HtmlDecimal
            | InspectorFieldId::HtmlHex
            | InspectorFieldId::RustChar
            | InspectorFieldId::UnicodeEscape => details.encoding().is_some(),
            InspectorFieldId::DecompositionType | InspectorFieldId::Decomposition => {
                details.decomposition().is_some()
            }
            _ => true,
        }
    }

    fn code_point_mapping(
        id: InspectorFieldId,
        section: InspectorSection,
        label: &'static str,
        mapping: impl Iterator<Item = CodePoint>,
    ) -> Self {
        Self::copyable(
            id,
            section,
            label,
            mapping.map(|code_point| {
                let name = UnicodeDatabase::primary_name_or_fallback(code_point);
                format!("{code_point} {name}")
            }),
        )
    }

    fn copyable(
        id: InspectorFieldId,
        section: InspectorSection,
        label: &'static str,
        values: impl IntoIterator<Item = String>,
    ) -> Self {
        let values = values.into_iter().collect::<Vec<_>>();
        let copy_value = Some(values.join("\n"));
        Self::new(id, section, label, values, copy_value)
    }

    fn unavailable(
        id: InspectorFieldId,
        section: InspectorSection,
        label: &'static str,
        value: &'static str,
    ) -> Self {
        Self::new(id, section, label, vec![value.to_owned()], None)
    }

    fn new(
        id: InspectorFieldId,
        section: InspectorSection,
        label: &'static str,
        values: Vec<String>,
        copy_value: Option<String>,
    ) -> Self {
        Self {
            id,
            section,
            label,
            values,
            copy_value,
        }
    }

    pub const fn id(&self) -> InspectorFieldId {
        self.id
    }

    pub const fn section(&self) -> InspectorSection {
        self.section
    }

    pub const fn label(&self) -> &'static str {
        self.label
    }

    pub fn values(&self) -> &[String] {
        &self.values
    }

    pub fn copy_value(&self) -> Option<&str> {
        self.copy_value.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures;

    #[test]
    fn case_fields_preserve_full_mapping_order_and_show_identity_only_in_all_mode() {
        let fields =
            InspectorField::for_code_point(CodePoint::from('ß'), InspectorDisplayMode::Relevant);
        for (id, expected) in [
            (
                InspectorFieldId::UppercaseMapping,
                "U+0053 LATIN CAPITAL LETTER S\nU+0053 LATIN CAPITAL LETTER S",
            ),
            (
                InspectorFieldId::TitlecaseMapping,
                "U+0053 LATIN CAPITAL LETTER S\nU+0073 LATIN SMALL LETTER S",
            ),
            (
                InspectorFieldId::CaseFolding,
                "U+0073 LATIN SMALL LETTER S\nU+0073 LATIN SMALL LETTER S",
            ),
        ] {
            let field = fields.iter().find(|field| field.id() == id).unwrap();
            assert_eq!(field.copy_value(), Some(expected));
        }
        assert!(
            fields
                .iter()
                .all(|field| field.id() != InspectorFieldId::LowercaseMapping)
        );
        let all = InspectorField::for_code_point(CodePoint::from('A'), InspectorDisplayMode::All);
        let uppercase = all
            .iter()
            .find(|field| field.id() == InspectorFieldId::UppercaseMapping)
            .unwrap();
        assert_eq!(uppercase.values(), ["U+0041 LATIN CAPITAL LETTER A"]);
        assert_eq!(
            uppercase.copy_value(),
            Some("U+0041 LATIN CAPITAL LETTER A")
        );
        let uncased =
            InspectorField::for_code_point(CodePoint::from('あ'), InspectorDisplayMode::Relevant);
        assert!(
            uncased
                .iter()
                .all(|field| field.section() != InspectorSection::Case)
        );
    }

    #[test]
    fn numeric_fields_show_exact_values_and_hide_only_when_absent() {
        for (character, kind, value) in [
            ('0', "Decimal", "0"),
            ('½', "Numeric", "1/2"),
            ('四', "Numeric", "4"),
        ] {
            for mode in [InspectorDisplayMode::Relevant, InspectorDisplayMode::All] {
                let fields = InspectorField::for_code_point(CodePoint::from(character), mode);
                let numeric = fields
                    .iter()
                    .filter(|field| field.section() == InspectorSection::Numeric)
                    .collect::<Vec<_>>();
                assert_eq!(numeric.len(), 2);
                assert_eq!(numeric[0].copy_value(), Some(kind));
                assert_eq!(numeric[1].values(), [value]);
                assert_eq!(numeric[1].copy_value(), Some(value));
            }
        }
        let relevant =
            InspectorField::for_code_point(CodePoint::from('A'), InspectorDisplayMode::Relevant);
        assert!(
            relevant
                .iter()
                .all(|field| field.section() != InspectorSection::Numeric)
        );
        let all = InspectorField::for_code_point(CodePoint::from('A'), InspectorDisplayMode::All);
        let numeric = all
            .iter()
            .filter(|field| field.section() == InspectorSection::Numeric)
            .collect::<Vec<_>>();
        assert_eq!(numeric[0].values(), ["None"]);
        assert_eq!(numeric[1].values(), ["NaN"]);
        assert_eq!(numeric[1].copy_value(), None);
    }

    #[test]
    fn selection_follows_property_identity_and_falls_back_to_nearby_visible_fields() {
        use InspectorDisplayMode::{All, Relevant};
        use InspectorFieldId as F;

        let cases = [
            (
                0x0041,
                All,
                F::GeneralCategory,
                0x0041,
                Relevant,
                F::GeneralCategory,
            ),
            (0x0041, All, F::Aliases, 0x0041, Relevant, F::Plane),
            (
                0x0041,
                All,
                F::CanonicalCombiningClass,
                0x0041,
                Relevant,
                F::BidiClass,
            ),
            (
                0x0041,
                All,
                F::DefaultIgnorable,
                0x0041,
                Relevant,
                F::BidiClass,
            ),
            (
                0x0041,
                All,
                F::DecompositionType,
                0x0041,
                Relevant,
                F::UnicodeEscape,
            ),
            (
                0x0041,
                All,
                F::Decomposition,
                0x0041,
                Relevant,
                F::UnicodeVersion,
            ),
            (0xd800, All, F::Utf8, 0xd800, Relevant, F::BidiClass),
            (
                0x0021,
                Relevant,
                F::GeneralCategory,
                0x0020,
                Relevant,
                F::GeneralCategory,
            ),
            (
                0x00e9,
                Relevant,
                F::Decomposition,
                0x00e8,
                Relevant,
                F::Decomposition,
            ),
            (
                0x00e9,
                Relevant,
                F::Decomposition,
                0x0041,
                Relevant,
                F::UnicodeVersion,
            ),
            (0xd7ff, Relevant, F::Utf8, 0xd800, Relevant, F::BidiClass),
            (0x034f, Relevant, F::Aliases, 0x0350, Relevant, F::Plane),
        ];
        for (value, mode, selected, next, next_mode, expected) in cases {
            let code_point = CodePoint::new(value).unwrap();
            let fields = InspectorField::for_code_point(code_point, mode);
            let mut state = InspectorState {
                display_mode: mode,
                selected_index: fields
                    .iter()
                    .position(|field| field.id() == selected)
                    .unwrap(),
                ..InspectorState::default()
            };
            let next = CodePoint::new(next).unwrap();
            state.follow_code_point(code_point, next);
            if mode != next_mode {
                state.toggle_display_mode(next);
            }
            let fields = InspectorField::for_code_point(next, next_mode);
            assert_eq!(
                fields[state.selected_index()].id(),
                expected,
                "{code_point} {selected:?}"
            );
        }
    }

    #[test]
    fn moves_between_fields_and_across_pages() {
        let mut state = InspectorState::default();
        state.resize_viewport(4, 12, vec![1..2, 2..4, 5..6, 8..10, 10..11], Vec::new());

        state.move_selection(InspectorMove::NextField);
        assert_eq!(state.selected_index(), 1);
        assert_eq!(state.offset(), 0);
        state.move_selection(InspectorMove::NextField);
        assert_eq!(state.selected_index(), 2);
        assert_eq!(state.offset(), 2);
        state.move_selection(InspectorMove::PageForward);
        assert_eq!(state.selected_index(), 4);
        assert_eq!(state.offset(), 7);
        state.move_selection(InspectorMove::NextField);
        assert_eq!(state.selected_index(), 4);
        state.move_selection(InspectorMove::PageBackward);
        assert_eq!(state.selected_index(), 2);
        state.move_selection(InspectorMove::First);
        assert_eq!(state.selected_index(), 0);
        assert_eq!(state.offset(), 0);
        state.move_selection(InspectorMove::PreviousField);
        assert_eq!(state.selected_index(), 0);
    }

    #[test]
    fn returning_to_the_first_field_restores_the_document_heading() {
        for movement in [InspectorMove::PreviousField, InspectorMove::PageBackward] {
            let mut state = InspectorState::default();
            state.resize_viewport(4, 12, vec![1..2, 2..4, 5..6, 8..10, 10..11], Vec::new());
            state.move_selection(InspectorMove::Last);

            while state.selected_index() > 0 {
                state.move_selection(movement);
            }

            assert_eq!(state.offset(), 0, "{movement:?}");
        }
    }

    #[test]
    fn group_movement_targets_adjacent_sections_even_from_the_middle() {
        let mut state = InspectorState::default();
        state.resize_viewport(
            3,
            13,
            vec![1..2, 2..4, 6..7, 7..9, 11..13],
            vec![
                InspectorGroup {
                    first_field: 0,
                    heading_line: 0,
                },
                InspectorGroup {
                    first_field: 2,
                    heading_line: 5,
                },
                InspectorGroup {
                    first_field: 4,
                    heading_line: 10,
                },
            ],
        );
        state.move_selection(InspectorMove::NextField);
        let before = state.clone();
        state.move_selection(InspectorMove::PreviousGroup);
        assert_eq!(state, before);

        state.move_selection(InspectorMove::NextGroup);
        assert_eq!(state.selected_index(), 2);
        assert_eq!(state.visible_range(), 5..8);
        state.move_selection(InspectorMove::NextField);
        state.move_selection(InspectorMove::PreviousGroup);
        assert_eq!(state.selected_index(), 0);
        assert_eq!(state.visible_range(), 0..3);

        state.move_selection(InspectorMove::NextGroup);
        state.move_selection(InspectorMove::NextGroup);
        assert_eq!(state.selected_index(), 4);
        assert_eq!(state.visible_range(), 10..13);
        let before = state.clone();
        state.move_selection(InspectorMove::NextGroup);
        assert_eq!(state, before);
    }

    #[test]
    fn group_jump_preserves_the_heading_when_the_first_field_exceeds_the_viewport() {
        let mut state = InspectorState::default();
        let fields = vec![1..2, 4..12, 12..13];
        let groups = vec![
            InspectorGroup {
                first_field: 0,
                heading_line: 0,
            },
            InspectorGroup {
                first_field: 1,
                heading_line: 3,
            },
        ];
        state.resize_viewport(3, 13, fields.clone(), groups.clone());
        state.move_selection(InspectorMove::NextGroup);
        assert_eq!(state.selected_index(), 1);
        assert_eq!(state.visible_range(), 3..6);
        state.resize_viewport(2, 13, fields.clone(), groups.clone());
        assert_eq!(state.visible_range(), 3..5);
        state.resize_viewport(1, 13, fields.clone(), groups.clone());
        assert_eq!(state.visible_range(), 4..5);
        state.resize_viewport(0, 13, fields.clone(), groups.clone());
        assert_eq!(state.visible_range(), 0..0);
        state.resize_viewport(3, 13, fields, groups);
        assert_eq!(state.visible_range(), 3..6);

        state.move_selection(InspectorMove::NextField);
        assert_eq!(state.selected_index(), 2);
        assert_eq!(state.visible_range(), 10..13);
    }

    #[test]
    fn resizing_keeps_the_complete_selection_visible() {
        let mut state = InspectorState::default();
        let ranges = vec![1..2, 10..11, 32..35];
        state.resize_viewport(10, 35, ranges.clone(), Vec::new());
        state.move_selection(InspectorMove::Last);
        assert_eq!(state.visible_range(), 25..35);

        state.resize_viewport(5, 35, ranges.clone(), Vec::new());
        assert_eq!(state.visible_range(), 30..35);
        state.resize_viewport(20, 35, ranges.clone(), Vec::new());
        assert_eq!(state.visible_range(), 15..35);
        state.resize_viewport(40, 35, ranges, Vec::new());
        assert_eq!(state.visible_range(), 0..35);
    }

    #[test]
    fn zero_height_and_empty_documents_have_empty_ranges() {
        let mut state = InspectorState::default();

        state.resize_viewport(0, 20, vec![1..2, 2..3], Vec::new());
        state.move_selection(InspectorMove::Last);
        assert_eq!(state.visible_range(), 0..0);

        state.resize_viewport(10, 0, Vec::new(), Vec::new());
        assert_eq!(state.visible_range(), 0..0);
    }

    #[test]
    fn presents_basic_details_for_an_assigned_character() {
        let state = fixtures::startup();
        let details = InspectorDetails::for_code_point(state.selected());
        let category = details.general_category();

        assert_eq!(details.character().as_str(), "A");
        assert_eq!(details.code_point().to_string(), "U+0041");
        assert_eq!(details.primary_name(), "LATIN CAPITAL LETTER A");
        assert_eq!(details.plane().number(), 0);
        assert_eq!(details.plane().name(), Some("Basic Multilingual Plane"));
        assert_eq!(details.block(), "Basic Latin");
        assert_eq!(category.abbreviation(), "Lu");
        assert_eq!(category.name(), "Uppercase Letter");
        assert_eq!(details.script(), "Latin");
        assert_eq!(details.age(), "1.1");
        assert!(!details.is_default_ignorable());
        assert_eq!(details.unicode_version(), "17.0.0");
    }

    #[test]
    fn presents_explicit_defaults_for_an_unassigned_code_point() {
        let state = fixtures::unassigned();
        let details = InspectorDetails::for_code_point(state.selected());

        assert_eq!(details.character().as_str(), "<UNASSIGNED>");
        assert_eq!(details.primary_name(), "Unassigned");
        assert_eq!(details.block(), "Greek and Coptic");
        assert_eq!(details.general_category(), GeneralCategory::Unassigned);
        assert_eq!(details.script(), "Unknown");
        assert_eq!(details.age(), "Unassigned");
        assert!(!details.is_default_ignorable());
    }

    #[test]
    fn preserves_default_ignorable_safe_display_and_property() {
        let state = fixtures::default_ignorable();
        let details = InspectorDetails::for_code_point(state.selected());

        assert_eq!(details.character().as_str(), "<DEFAULT IGNORABLE>");
        assert_eq!(details.primary_name(), "HANGUL CHOSEONG FILLER");
        assert!(details.is_default_ignorable());
    }

    #[test]
    fn presents_a_surrogate_without_converting_it_to_char() {
        let state = fixtures::surrogate();
        let details = InspectorDetails::for_code_point(state.selected());

        assert_eq!(details.character().as_str(), "<SURROGATE>");
        assert_eq!(details.primary_name(), "No Primary Name");
        assert_eq!(details.block(), "High Surrogates");
        assert_eq!(details.general_category(), GeneralCategory::Surrogate);
        assert_eq!(details.script(), "Unknown");
        assert_eq!(details.age(), "2.0");
    }

    #[test]
    fn identifies_code_points_outside_unicode_blocks() {
        let details = InspectorDetails::for_code_point(CodePoint::new(0x2fe0).unwrap());

        assert_eq!(details.block(), "No Block");
    }

    #[test]
    fn builds_stable_fields_with_unwrapped_copy_values() {
        let fields =
            InspectorField::for_code_point(CodePoint::new(0).unwrap(), InspectorDisplayMode::All);

        assert_eq!(fields.len(), 22);
        assert_eq!(fields[0].id(), InspectorFieldId::Character);
        assert_eq!(fields[0].section(), InspectorSection::Identity);
        assert_eq!(fields[0].label(), "Character");
        assert_eq!(fields[0].values(), ["<CONTROL>"]);
        assert_eq!(fields[0].copy_value(), Some("\0"));
        assert_eq!(fields[1].copy_value(), Some("U+0000"));
        assert_eq!(
            fields[3].copy_value(),
            Some("NULL — control\nNUL — abbreviation")
        );
    }

    #[test]
    fn relevant_properties_preserve_basic_values_for_all_scalar_categories() {
        use InspectorFieldId::*;

        for value in [0x0041, 0x0378, 0xe000, 0x2fe0, 0x10ffff] {
            let code_point = crate::unicode::CodePoint::new(value).unwrap();
            let fields = InspectorField::for_code_point(code_point, InspectorDisplayMode::Relevant);
            assert_eq!(
                fields.iter().map(InspectorField::id).collect::<Vec<_>>(),
                [
                    Character,
                    CodePoint,
                    PrimaryName,
                    Plane,
                    Block,
                    GeneralCategory,
                    Script,
                    Age,
                    EastAsianWidth,
                    BidiClass,
                    Utf8,
                    Utf16,
                    HtmlDecimal,
                    HtmlHex,
                    RustChar,
                    UnicodeEscape,
                    UnicodeVersion,
                ],
                "{code_point}",
            );
        }
    }

    #[test]
    fn relevant_properties_include_applicable_supplemental_data() {
        use InspectorFieldId::*;

        let supplemental = [
            Aliases,
            CanonicalCombiningClass,
            DefaultIgnorable,
            DecompositionType,
            Decomposition,
        ];
        let cases: &[(u32, &[InspectorFieldId])] = &[
            (0x0000, &[Aliases]),
            (0x00e9, &[DecompositionType, Decomposition]),
            (0xfb01, &[DecompositionType, Decomposition]),
            (0xac00, &[DecompositionType, Decomposition]),
            (0x0301, &[CanonicalCombiningClass]),
            (
                0x034f,
                &[Aliases, CanonicalCombiningClass, DefaultIgnorable],
            ),
            (0x0903, &[CanonicalCombiningClass]),
            (0x20dd, &[CanonicalCombiningClass]),
            (0x115f, &[DefaultIgnorable]),
        ];
        for &(value, expected) in cases {
            let code_point = crate::unicode::CodePoint::new(value).unwrap();
            let fields = InspectorField::for_code_point(code_point, InspectorDisplayMode::Relevant);
            assert_eq!(
                fields
                    .iter()
                    .map(InspectorField::id)
                    .filter(|id| supplemental.contains(id))
                    .collect::<Vec<_>>(),
                expected,
                "{code_point}",
            );
        }
    }

    #[test]
    fn marks_absent_and_non_scalar_field_values_as_unavailable_to_copy() {
        let ordinary =
            InspectorField::for_code_point(CodePoint::from('A'), InspectorDisplayMode::All);
        let surrogate = InspectorField::for_code_point(
            CodePoint::new(0xd800).unwrap(),
            InspectorDisplayMode::All,
        );

        assert_eq!(ordinary[3].values(), ["None"]);
        assert_eq!(ordinary[3].copy_value(), None);
        assert_eq!(surrogate[0].copy_value(), None);
        assert_eq!(surrogate[13].id(), InspectorFieldId::Utf8);
        assert_eq!(surrogate[13].copy_value(), None);

        let relevant = InspectorField::for_code_point(
            CodePoint::new(0xd800).unwrap(),
            InspectorDisplayMode::Relevant,
        );
        assert_eq!(relevant.len(), 11);
        assert!(
            relevant
                .iter()
                .all(|field| field.section() != InspectorSection::Encoding)
        );
        assert!(
            relevant
                .iter()
                .any(|field| field.id() == InspectorFieldId::GeneralCategory)
        );
    }

    #[test]
    fn keeps_decomposition_entries_separate_for_display_and_copy() {
        let fields = InspectorField::for_code_point(
            CodePoint::new(0x00e9).unwrap(),
            InspectorDisplayMode::All,
        );
        let decomposition = fields
            .iter()
            .find(|field| field.id() == InspectorFieldId::Decomposition)
            .unwrap();

        assert_eq!(
            decomposition.values(),
            [
                "U+0065 LATIN SMALL LETTER E",
                "U+0301 COMBINING ACUTE ACCENT"
            ]
        );
        assert_eq!(
            decomposition.copy_value(),
            Some("U+0065 LATIN SMALL LETTER E\nU+0301 COMBINING ACUTE ACCENT")
        );
    }
}
