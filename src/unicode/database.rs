use crate::unicode::generated::{
    AGES, BIDI_CLASSES, BLOCKS, CANONICAL_COMBINING_CLASS_VALUES, CANONICAL_COMBINING_CLASSES,
    DECOMPOSITIONS, DEFAULT_IGNORABLES, EAST_ASIAN_WIDTHS, GENERAL_CATEGORIES, NAME_ALIASES,
    PRIMARY_NAMES, SCRIPTS, UNICODE_VERSION,
};
use crate::unicode::{
    BidiClass, CanonicalCombiningClass, CodePoint, Decomposition, DisplayKind,
    DisplayRepresentation, EastAsianWidth, NameAlias, UnicodeScalarEncoding,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnicodeDatabase;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnicodeBlock {
    index: usize,
}

impl UnicodeBlock {
    pub const fn index(self) -> usize {
        self.index
    }

    pub fn start(self) -> CodePoint {
        CodePoint::new(BLOCKS[self.index].0).expect("generated block starts are valid code points")
    }

    pub fn end(self) -> CodePoint {
        CodePoint::new(BLOCKS[self.index].1).expect("generated block ends are valid code points")
    }

    pub const fn name(self) -> &'static str {
        BLOCKS[self.index].2
    }

    pub fn contains(self, code_point: CodePoint) -> bool {
        self.start() <= code_point && code_point <= self.end()
    }
}

impl UnicodeDatabase {
    pub const fn version() -> &'static str {
        UNICODE_VERSION
    }

    pub fn lookup(code_point: CodePoint) -> UnicodeRecord {
        UnicodeRecord {
            code_point,
            primary_name: primary_name(code_point.value()),
            general_category: general_category(code_point.value()),
            block: Self::block_containing(code_point).map(UnicodeBlock::name),
            script: range_value(SCRIPTS, code_point.value()).unwrap_or("Unknown"),
            age: range_value(AGES, code_point.value()),
            default_ignorable: range_contains(DEFAULT_IGNORABLES, code_point.value()),
            name_aliases: name_aliases(code_point.value()),
            east_asian_width: complete_range_value(EAST_ASIAN_WIDTHS, code_point.value()),
            canonical_combining_class: canonical_combining_class(code_point.value()),
            bidi_class: complete_range_value(BIDI_CLASSES, code_point.value()),
            decomposition: decomposition(code_point),
        }
    }

    pub fn general_category(code_point: CodePoint) -> GeneralCategory {
        general_category(code_point.value())
    }

    pub fn primary_name_or_fallback(code_point: CodePoint) -> &'static str {
        primary_name_or_fallback(
            primary_name(code_point.value()),
            general_category(code_point.value()),
        )
    }

    pub fn display_representation(code_point: CodePoint) -> DisplayRepresentation {
        display_representation(
            code_point,
            general_category(code_point.value()),
            range_contains(DEFAULT_IGNORABLES, code_point.value()),
        )
    }

    pub fn primary_names() -> impl Iterator<Item = (CodePoint, &'static str)> {
        PRIMARY_NAMES.iter().map(|&(value, name)| {
            let code_point =
                CodePoint::new(value).expect("generated primary names contain valid code points");
            (code_point, name)
        })
    }

    pub fn name_aliases() -> impl Iterator<Item = (CodePoint, NameAlias)> {
        NAME_ALIASES.iter().flat_map(|&(value, aliases)| {
            let code_point =
                CodePoint::new(value).expect("generated name aliases contain valid code points");
            aliases
                .iter()
                .copied()
                .map(move |alias| (code_point, alias))
        })
    }

    pub fn blocks() -> impl ExactSizeIterator<Item = UnicodeBlock> {
        (0..BLOCKS.len()).map(|index| UnicodeBlock { index })
    }

    pub fn block(index: usize) -> Option<UnicodeBlock> {
        (index < BLOCKS.len()).then_some(UnicodeBlock { index })
    }

    pub fn block_at_or_after(code_point: CodePoint) -> Option<UnicodeBlock> {
        let index = BLOCKS.partition_point(|&(_, end, _)| end < code_point.value());
        Self::block(index)
    }

    pub fn block_containing(code_point: CodePoint) -> Option<UnicodeBlock> {
        Self::block_at_or_after(code_point).filter(|block| block.contains(code_point))
    }

    pub fn block_names_in_range(
        start: CodePoint,
        end: CodePoint,
    ) -> impl Iterator<Item = &'static str> {
        assert!(start <= end, "a Unicode range starts at or before its end");

        let first = BLOCKS.partition_point(|&(_, block_end, _)| block_end < start.value());
        BLOCKS[first..]
            .iter()
            .take_while(move |&&(block_start, _, _)| block_start <= end.value())
            .map(|&(_, _, name)| name)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnicodeRecord {
    code_point: CodePoint,
    primary_name: Option<&'static str>,
    general_category: GeneralCategory,
    block: Option<&'static str>,
    script: &'static str,
    age: Option<&'static str>,
    default_ignorable: bool,
    name_aliases: &'static [NameAlias],
    east_asian_width: EastAsianWidth,
    canonical_combining_class: CanonicalCombiningClass,
    bidi_class: BidiClass,
    decomposition: Option<Decomposition>,
}

impl UnicodeRecord {
    pub const fn code_point(self) -> CodePoint {
        self.code_point
    }

    pub fn primary_name_or_fallback(self) -> &'static str {
        primary_name_or_fallback(self.primary_name, self.general_category)
    }

    pub const fn general_category(self) -> GeneralCategory {
        self.general_category
    }

    pub const fn block(self) -> Option<&'static str> {
        self.block
    }

    pub const fn script(self) -> &'static str {
        self.script
    }

    pub const fn age(self) -> Option<&'static str> {
        self.age
    }

    pub const fn is_default_ignorable(self) -> bool {
        self.default_ignorable
    }

    pub const fn name_aliases(self) -> &'static [NameAlias] {
        self.name_aliases
    }

    pub const fn east_asian_width(self) -> EastAsianWidth {
        self.east_asian_width
    }

    pub const fn canonical_combining_class(self) -> CanonicalCombiningClass {
        self.canonical_combining_class
    }

    pub const fn bidi_class(self) -> BidiClass {
        self.bidi_class
    }

    pub const fn decomposition(self) -> Option<Decomposition> {
        self.decomposition
    }

    pub const fn encoding(self) -> Option<UnicodeScalarEncoding> {
        UnicodeScalarEncoding::for_code_point(self.code_point)
    }

    pub fn display_representation(self) -> DisplayRepresentation {
        display_representation(
            self.code_point,
            self.general_category,
            self.default_ignorable,
        )
    }
}

fn primary_name_or_fallback(
    primary_name: Option<&'static str>,
    general_category: GeneralCategory,
) -> &'static str {
    primary_name.unwrap_or_else(|| {
        if general_category == GeneralCategory::Unassigned {
            "Unassigned"
        } else {
            "No Primary Name"
        }
    })
}

fn display_representation(
    code_point: CodePoint,
    general_category: GeneralCategory,
    default_ignorable: bool,
) -> DisplayRepresentation {
    let display_kind = if default_ignorable {
        DisplayKind::DefaultIgnorable
    } else {
        general_category.display_kind()
    };
    code_point.display_representation(display_kind)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeneralCategory {
    UppercaseLetter,
    LowercaseLetter,
    TitlecaseLetter,
    ModifierLetter,
    OtherLetter,
    NonspacingMark,
    SpacingMark,
    EnclosingMark,
    DecimalNumber,
    LetterNumber,
    OtherNumber,
    ConnectorPunctuation,
    DashPunctuation,
    OpenPunctuation,
    ClosePunctuation,
    InitialPunctuation,
    FinalPunctuation,
    OtherPunctuation,
    MathSymbol,
    CurrencySymbol,
    ModifierSymbol,
    OtherSymbol,
    SpaceSeparator,
    LineSeparator,
    ParagraphSeparator,
    Control,
    Format,
    Surrogate,
    PrivateUse,
    Unassigned,
}

impl GeneralCategory {
    pub const fn abbreviation(self) -> &'static str {
        match self {
            Self::UppercaseLetter => "Lu",
            Self::LowercaseLetter => "Ll",
            Self::TitlecaseLetter => "Lt",
            Self::ModifierLetter => "Lm",
            Self::OtherLetter => "Lo",
            Self::NonspacingMark => "Mn",
            Self::SpacingMark => "Mc",
            Self::EnclosingMark => "Me",
            Self::DecimalNumber => "Nd",
            Self::LetterNumber => "Nl",
            Self::OtherNumber => "No",
            Self::ConnectorPunctuation => "Pc",
            Self::DashPunctuation => "Pd",
            Self::OpenPunctuation => "Ps",
            Self::ClosePunctuation => "Pe",
            Self::InitialPunctuation => "Pi",
            Self::FinalPunctuation => "Pf",
            Self::OtherPunctuation => "Po",
            Self::MathSymbol => "Sm",
            Self::CurrencySymbol => "Sc",
            Self::ModifierSymbol => "Sk",
            Self::OtherSymbol => "So",
            Self::SpaceSeparator => "Zs",
            Self::LineSeparator => "Zl",
            Self::ParagraphSeparator => "Zp",
            Self::Control => "Cc",
            Self::Format => "Cf",
            Self::Surrogate => "Cs",
            Self::PrivateUse => "Co",
            Self::Unassigned => "Cn",
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::UppercaseLetter => "Uppercase Letter",
            Self::LowercaseLetter => "Lowercase Letter",
            Self::TitlecaseLetter => "Titlecase Letter",
            Self::ModifierLetter => "Modifier Letter",
            Self::OtherLetter => "Other Letter",
            Self::NonspacingMark => "Nonspacing Mark",
            Self::SpacingMark => "Spacing Mark",
            Self::EnclosingMark => "Enclosing Mark",
            Self::DecimalNumber => "Decimal Number",
            Self::LetterNumber => "Letter Number",
            Self::OtherNumber => "Other Number",
            Self::ConnectorPunctuation => "Connector Punctuation",
            Self::DashPunctuation => "Dash Punctuation",
            Self::OpenPunctuation => "Open Punctuation",
            Self::ClosePunctuation => "Close Punctuation",
            Self::InitialPunctuation => "Initial Punctuation",
            Self::FinalPunctuation => "Final Punctuation",
            Self::OtherPunctuation => "Other Punctuation",
            Self::MathSymbol => "Math Symbol",
            Self::CurrencySymbol => "Currency Symbol",
            Self::ModifierSymbol => "Modifier Symbol",
            Self::OtherSymbol => "Other Symbol",
            Self::SpaceSeparator => "Space Separator",
            Self::LineSeparator => "Line Separator",
            Self::ParagraphSeparator => "Paragraph Separator",
            Self::Control => "Control",
            Self::Format => "Format",
            Self::Surrogate => "Surrogate",
            Self::PrivateUse => "Private Use",
            Self::Unassigned => "Unassigned",
        }
    }

    const fn display_kind(self) -> DisplayKind {
        match self {
            Self::NonspacingMark | Self::SpacingMark | Self::EnclosingMark => {
                DisplayKind::CombiningMark
            }
            Self::SpaceSeparator => DisplayKind::Space,
            Self::LineSeparator => DisplayKind::LineSeparator,
            Self::ParagraphSeparator => DisplayKind::ParagraphSeparator,
            Self::Control => DisplayKind::Control,
            Self::Format => DisplayKind::Format,
            Self::Unassigned => DisplayKind::Unassigned,
            Self::UppercaseLetter
            | Self::LowercaseLetter
            | Self::TitlecaseLetter
            | Self::ModifierLetter
            | Self::OtherLetter
            | Self::DecimalNumber
            | Self::LetterNumber
            | Self::OtherNumber
            | Self::ConnectorPunctuation
            | Self::DashPunctuation
            | Self::OpenPunctuation
            | Self::ClosePunctuation
            | Self::InitialPunctuation
            | Self::FinalPunctuation
            | Self::OtherPunctuation
            | Self::MathSymbol
            | Self::CurrencySymbol
            | Self::ModifierSymbol
            | Self::OtherSymbol
            | Self::Surrogate
            | Self::PrivateUse => DisplayKind::Graphic,
        }
    }
}

fn primary_name(value: u32) -> Option<&'static str> {
    PRIMARY_NAMES
        .binary_search_by_key(&value, |(code_point, _)| *code_point)
        .ok()
        .map(|index| PRIMARY_NAMES[index].1)
}

fn general_category(value: u32) -> GeneralCategory {
    let insertion_index = GENERAL_CATEGORIES.partition_point(|range| range.0 <= value);
    let range = GENERAL_CATEGORIES[insertion_index.saturating_sub(1)];
    debug_assert!(range.0 <= value && value <= range.1);
    range.2
}

fn range_value(ranges: &'static [(u32, u32, &'static str)], value: u32) -> Option<&'static str> {
    let insertion_index = ranges.partition_point(|range| range.0 <= value);
    insertion_index
        .checked_sub(1)
        .map(|index| ranges[index])
        .filter(|range| value <= range.1)
        .map(|range| range.2)
}

fn range_contains(ranges: &[(u32, u32)], value: u32) -> bool {
    let insertion_index = ranges.partition_point(|range| range.0 <= value);
    insertion_index
        .checked_sub(1)
        .map(|index| value <= ranges[index].1)
        .unwrap_or(false)
}

fn name_aliases(value: u32) -> &'static [NameAlias] {
    NAME_ALIASES
        .binary_search_by_key(&value, |(code_point, _)| *code_point)
        .ok()
        .map_or(&[], |index| NAME_ALIASES[index].1)
}

fn complete_range_value<T: Copy>(ranges: &[(u32, u32, T)], value: u32) -> T {
    let insertion_index = ranges.partition_point(|range| range.0 <= value);
    let range = ranges[insertion_index.saturating_sub(1)];
    debug_assert!(range.0 <= value && value <= range.1);
    range.2
}

fn canonical_combining_class(value: u32) -> CanonicalCombiningClass {
    let numeric_value = complete_range_value(CANONICAL_COMBINING_CLASSES, value);
    let index = CANONICAL_COMBINING_CLASS_VALUES
        .binary_search_by_key(&numeric_value, |(value, _)| *value)
        .expect("generated combining class values contain every range value");
    let (_, name) = CANONICAL_COMBINING_CLASS_VALUES[index];
    CanonicalCombiningClass::new(numeric_value, name)
}

fn decomposition(code_point: CodePoint) -> Option<Decomposition> {
    Decomposition::hangul(code_point).or_else(|| {
        DECOMPOSITIONS
            .binary_search_by_key(&code_point.value(), |(value, _)| *value)
            .ok()
            .map(|index| DECOMPOSITIONS[index].1)
    })
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::unicode::{DecompositionType, NameAliasType};

    #[test]
    fn exposes_the_unicode_version() {
        assert_eq!(UnicodeDatabase::version(), "17.0.0");
    }

    #[rustfmt::skip]
    #[rstest]
    #[case(
        0x0041,
        Some("LATIN CAPITAL LETTER A"),
        GeneralCategory::UppercaseLetter,
        "A"
    )]
    #[case(
        0x0301,
        Some("COMBINING ACUTE ACCENT"),
        GeneralCategory::NonspacingMark,
        "◌́"
    )]
    #[case(
        0x000a,
        None,
        GeneralCategory::Control,
        "<CONTROL>"
    )]
    #[case(
        0x200d,
        Some("ZERO WIDTH JOINER"),
        GeneralCategory::Format,
        "<DEFAULT IGNORABLE>"
    )]
    #[case(
        0x202e,
        Some("RIGHT-TO-LEFT OVERRIDE"),
        GeneralCategory::Format,
        "<DEFAULT IGNORABLE>"
    )]
    #[case(
        0x2028,
        Some("LINE SEPARATOR"),
        GeneralCategory::LineSeparator,
        "<LINE SEPARATOR>"
    )]
    #[case(
        0x2029,
        Some("PARAGRAPH SEPARATOR"),
        GeneralCategory::ParagraphSeparator,
        "<PARAGRAPH SEPARATOR>"
    )]
    #[case(
        0x0378,
        None,
        GeneralCategory::Unassigned,
        "<UNASSIGNED>"
    )]
    #[case(
        0xd800,
        None,
        GeneralCategory::Surrogate,
        "<SURROGATE>"
    )]
    #[case(
        0xe000,
        None,
        GeneralCategory::PrivateUse,
        "<PRIVATE USE>"
    )]
    #[case(
        0xfdd0,
        None,
        GeneralCategory::Unassigned,
        "<NONCHARACTER>"
    )]
    fn looks_up_representative_records_and_safe_display(
        #[case] value: u32,
        #[case] expected_name: Option<&str>,
        #[case] expected_category: GeneralCategory,
        #[case] expected_display: &str,
    ) {
        let record = UnicodeDatabase::lookup(CodePoint::new(value).unwrap());

        assert_eq!(record.code_point().value(), value);
        assert_eq!(record.primary_name, expected_name);
        assert_eq!(record.general_category(), expected_category);
        assert_eq!(record.display_representation().as_str(), expected_display);
    }

    #[rstest]
    #[case(0x4e00, "CJK UNIFIED IDEOGRAPH-4E00")]
    #[case(0xac00, "HANGUL SYLLABLE GA")]
    #[case(0x18cff, "KHITAN SMALL SCRIPT CHARACTER-18CFF")]
    #[case(0x1f600, "GRINNING FACE")]
    #[case(0x20000, "CJK UNIFIED IDEOGRAPH-20000")]
    fn expands_algorithmic_primary_names(#[case] value: u32, #[case] expected_name: &str) {
        let record = UnicodeDatabase::lookup(CodePoint::new(value).unwrap());

        assert_eq!(record.primary_name, Some(expected_name));
    }

    #[test]
    fn exposes_general_category_names() {
        let category = UnicodeDatabase::lookup(CodePoint::from('A')).general_category();

        assert_eq!(category.abbreviation(), "Lu");
        assert_eq!(category.name(), "Uppercase Letter");
    }

    #[test]
    fn exposes_all_formal_name_aliases_in_source_order() {
        let aliases = UnicodeDatabase::lookup(CodePoint::new(0x0000).unwrap()).name_aliases();

        assert_eq!(aliases.len(), 2);
        assert_eq!(aliases[0].name(), "NULL");
        assert_eq!(aliases[0].kind(), NameAliasType::Control);
        assert_eq!(aliases[0].kind().label(), "control");
        assert_eq!(aliases[1].name(), "NUL");
        assert_eq!(aliases[1].kind(), NameAliasType::Abbreviation);
        assert!(
            UnicodeDatabase::lookup(CodePoint::from('A'))
                .name_aliases()
                .is_empty()
        );
    }

    #[rstest]
    #[case(
        0x0041,
        EastAsianWidth::Narrow,
        0,
        "Not Reordered",
        BidiClass::LeftToRight
    )]
    #[case(
        0x0301,
        EastAsianWidth::Ambiguous,
        230,
        "Above",
        BidiClass::NonspacingMark
    )]
    #[case(
        0x05d0,
        EastAsianWidth::Neutral,
        0,
        "Not Reordered",
        BidiClass::RightToLeft
    )]
    #[case(
        0x1f600,
        EastAsianWidth::Wide,
        0,
        "Not Reordered",
        BidiClass::OtherNeutral
    )]
    #[case(
        0xd800,
        EastAsianWidth::Neutral,
        0,
        "Not Reordered",
        BidiClass::LeftToRight
    )]
    fn looks_up_typed_classification_properties(
        #[case] value: u32,
        #[case] east_asian_width: EastAsianWidth,
        #[case] combining_value: u8,
        #[case] combining_name: &str,
        #[case] bidi_class: BidiClass,
    ) {
        let record = UnicodeDatabase::lookup(CodePoint::new(value).unwrap());
        let combining_class = record.canonical_combining_class();

        assert_eq!(record.east_asian_width(), east_asian_width);
        assert!(!record.east_asian_width().abbreviation().is_empty());
        assert_eq!(combining_class.value(), combining_value);
        assert_eq!(combining_class.name(), combining_name);
        assert_eq!(record.bidi_class(), bidi_class);
        assert!(!record.bidi_class().abbreviation().is_empty());
    }

    #[rstest]
    #[case(0x0590, BidiClass::RightToLeft)]
    #[case(0x070e, BidiClass::ArabicLetter)]
    #[case(0x20c2, BidiClass::EuropeanTerminator)]
    #[case(0x10ffff, BidiClass::BoundaryNeutral)]
    fn applies_bidi_defaults_to_unassigned_and_special_code_points(
        #[case] value: u32,
        #[case] expected: BidiClass,
    ) {
        let record = UnicodeDatabase::lookup(CodePoint::new(value).unwrap());

        assert_eq!(record.bidi_class(), expected);
    }

    #[rustfmt::skip]
    #[rstest]
    #[case(
        0x00e9,
        DecompositionType::Canonical,
        &[0x0065, 0x0301]
    )]
    #[case(
        0xfb01,
        DecompositionType::Compatibility("compat"),
        &[0x0066, 0x0069]
    )]
    #[case(
        0xac00,
        DecompositionType::Canonical,
        &[0x1100, 0x1161]
    )]
    #[case(
        0xac01,
        DecompositionType::Canonical,
        &[0x1100, 0x1161, 0x11a8]
    )]
    fn looks_up_direct_decompositions(
        #[case] value: u32,
        #[case] decomposition_type: DecompositionType,
        #[case] expected_mapping: &[u32],
    ) {
        let decomposition = UnicodeDatabase::lookup(CodePoint::new(value).unwrap())
            .decomposition()
            .unwrap();

        assert_eq!(decomposition.decomposition_type(), decomposition_type);
        assert_eq!(
            decomposition
                .mapping()
                .map(CodePoint::value)
                .collect::<Vec<_>>(),
            expected_mapping
        );
    }

    #[rstest]
    #[case(0x0041)]
    #[case(0x0378)]
    #[case(0xd800)]
    fn reports_the_absence_of_a_decomposition(#[case] value: u32) {
        assert!(
            UnicodeDatabase::lookup(CodePoint::new(value).unwrap())
                .decomposition()
                .is_none()
        );
    }

    #[rstest]
    #[case(0x0041, Some("41"), Some("0041"))]
    #[case(0x0378, Some("CD B8"), Some("0378"))]
    #[case(0x1f600, Some("F0 9F 98 80"), Some("D83D DE00"))]
    #[case(0xd800, None, None)]
    fn exposes_encodings_only_for_unicode_scalar_values(
        #[case] value: u32,
        #[case] expected_utf8: Option<&str>,
        #[case] expected_utf16: Option<&str>,
    ) {
        let encoding = UnicodeDatabase::lookup(CodePoint::new(value).unwrap()).encoding();

        assert_eq!(
            encoding.map(UnicodeScalarEncoding::utf8).as_deref(),
            expected_utf8
        );
        assert_eq!(
            encoding.map(UnicodeScalarEncoding::utf16).as_deref(),
            expected_utf16
        );
    }

    #[rstest]
    #[case(0x0041, Some("Basic Latin"), "Latin", Some("1.1"), false)]
    #[case(0x0378, Some("Greek and Coptic"), "Unknown", None, false)]
    #[case(0x2fe0, None, "Unknown", None, false)]
    #[case(0x115f, Some("Hangul Jamo"), "Hangul", Some("1.1"), true)]
    #[case(0xd800, Some("High Surrogates"), "Unknown", Some("2.0"), false)]
    #[case(0x1f600, Some("Emoticons"), "Common", Some("6.1"), false)]
    #[case(
        0xe0100,
        Some("Variation Selectors Supplement"),
        "Inherited",
        Some("4.0"),
        true
    )]
    fn looks_up_blocks_scripts_ages_and_default_ignorables(
        #[case] value: u32,
        #[case] block: Option<&str>,
        #[case] script: &str,
        #[case] age: Option<&str>,
        #[case] default_ignorable: bool,
    ) {
        let record = UnicodeDatabase::lookup(CodePoint::new(value).unwrap());

        assert_eq!(record.block(), block);
        assert_eq!(record.script(), script);
        assert_eq!(record.age(), age);
        assert_eq!(record.is_default_ignorable(), default_ignorable);
    }

    #[rstest]
    #[case(0x00ac, false)]
    #[case(0x00ad, true)]
    #[case(0x00ae, false)]
    #[case(0x034e, false)]
    #[case(0x034f, true)]
    #[case(0x0350, false)]
    #[case(0xe0000, true)]
    #[case(0xe0fff, true)]
    #[case(0xe1000, false)]
    fn checks_default_ignorable_boundaries(#[case] value: u32, #[case] expected: bool) {
        let record = UnicodeDatabase::lookup(CodePoint::new(value).unwrap());

        assert_eq!(record.is_default_ignorable(), expected);
    }

    #[test]
    fn gives_default_ignorables_a_safe_display() {
        let filler = UnicodeDatabase::lookup(CodePoint::new(0x115f).unwrap());
        assert_eq!(
            filler.display_representation().as_str(),
            "<DEFAULT IGNORABLE>"
        );
    }

    #[rstest]
    #[case(0x0040, Some("Basic Latin"), "Common")]
    #[case(0x0041, Some("Basic Latin"), "Latin")]
    #[case(0x005a, Some("Basic Latin"), "Latin")]
    #[case(0x005b, Some("Basic Latin"), "Common")]
    #[case(0x007f, Some("Basic Latin"), "Common")]
    #[case(0x0080, Some("Latin-1 Supplement"), "Common")]
    #[case(0x2fd5, Some("Kangxi Radicals"), "Han")]
    #[case(0x2fdf, Some("Kangxi Radicals"), "Unknown")]
    #[case(0x2fe0, None, "Unknown")]
    #[case(0x2ff0, Some("Ideographic Description Characters"), "Common")]
    fn looks_up_block_and_script_boundaries(
        #[case] value: u32,
        #[case] block: Option<&str>,
        #[case] script: &str,
    ) {
        let record = UnicodeDatabase::lookup(CodePoint::new(value).unwrap());

        assert_eq!(record.block(), block);
        assert_eq!(record.script(), script);
    }

    #[test]
    fn exposes_named_blocks_in_code_point_order() {
        let mut blocks = UnicodeDatabase::blocks();
        assert_eq!(blocks.len(), 346);
        let first = blocks.next().unwrap();
        assert_eq!(first.index(), 0);
        assert_eq!(first.name(), "Basic Latin");
        assert_eq!(first.start().value(), 0x0000);
        assert_eq!(first.end().value(), 0x007f);

        let last = blocks.last().unwrap();
        assert_eq!(last.name(), "Supplementary Private Use Area-B");
        assert_eq!(last.end().value(), CodePoint::MAX_VALUE);
        assert_eq!(UnicodeDatabase::block(346), None);
    }

    #[test]
    fn exposes_name_aliases_in_code_point_and_source_order() {
        let aliases = UnicodeDatabase::name_aliases().collect::<Vec<_>>();

        assert_eq!(aliases.len(), 481);
        assert_eq!(aliases[0].0.value(), 0x0000);
        assert_eq!(aliases[0].1.name(), "NULL");
        assert_eq!(aliases[1].0.value(), 0x0000);
        assert_eq!(aliases[1].1.name(), "NUL");
        assert!(aliases.windows(2).all(|pair| pair[0].0 <= pair[1].0));
    }

    #[test]
    fn distinguishes_block_membership_from_the_next_named_block() {
        let member = CodePoint::new(0x2fdf).unwrap();
        let gap = CodePoint::new(0x2fe0).unwrap();
        let next = CodePoint::new(0x2ff0).unwrap();
        let current_block = UnicodeDatabase::block_containing(member).unwrap();

        assert_eq!(current_block.name(), "Kangxi Radicals");
        assert!(current_block.contains(member));
        assert!(!current_block.contains(gap));
        assert_eq!(UnicodeDatabase::block_containing(gap), None);
        assert_eq!(
            UnicodeDatabase::block_at_or_after(gap).unwrap().name(),
            "Ideographic Description Characters"
        );
        assert_eq!(
            UnicodeDatabase::block_containing(next),
            UnicodeDatabase::block_at_or_after(gap)
        );
    }

    #[test]
    fn lists_block_names_that_overlap_a_code_point_range() {
        let names = UnicodeDatabase::block_names_in_range(
            CodePoint::new(0x0100).unwrap(),
            CodePoint::new(0x01ff).unwrap(),
        )
        .collect::<Vec<_>>();

        assert_eq!(names, ["Latin Extended-A", "Latin Extended-B"]);
    }

    #[test]
    fn lists_all_blocks_in_a_densely_partitioned_code_point_range() {
        let names = UnicodeDatabase::block_names_in_range(
            CodePoint::new(0x1c00).unwrap(),
            CodePoint::new(0x1cff).unwrap(),
        )
        .collect::<Vec<_>>();

        assert_eq!(
            names,
            [
                "Lepcha",
                "Ol Chiki",
                "Cyrillic Extended-C",
                "Georgian Extended",
                "Sundanese Supplement",
                "Vedic Extensions",
            ]
        );
    }

    #[test]
    fn lists_no_blocks_for_a_range_outside_named_blocks() {
        assert!(
            UnicodeDatabase::block_names_in_range(
                CodePoint::new(0x40000).unwrap(),
                CodePoint::new(0x400ff).unwrap(),
            )
            .next()
            .is_none()
        );
    }

    #[test]
    fn looks_up_every_code_point() {
        for value in CodePoint::MIN_VALUE..=CodePoint::MAX_VALUE {
            let code_point = CodePoint::new(value).unwrap();
            let record = UnicodeDatabase::lookup(code_point);

            assert_eq!(record.code_point(), code_point);
            assert!(!record.script().is_empty());
            assert!(!record.east_asian_width().name().is_empty());
            assert!(!record.canonical_combining_class().name().is_empty());
            assert!(!record.bidi_class().name().is_empty());
        }
    }
}
