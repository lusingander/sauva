#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NameAlias {
    name: &'static str,
    kind: NameAliasType,
}

impl NameAlias {
    pub const fn new(name: &'static str, kind: NameAliasType) -> Self {
        Self { name, kind }
    }

    pub const fn name(self) -> &'static str {
        self.name
    }

    pub const fn kind(self) -> NameAliasType {
        self.kind
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameAliasType {
    Correction,
    Control,
    Alternate,
    Figment,
    Abbreviation,
}

impl NameAliasType {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Correction => "correction",
            Self::Control => "control",
            Self::Alternate => "alternate",
            Self::Figment => "figment",
            Self::Abbreviation => "abbreviation",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EastAsianWidth {
    Ambiguous,
    Fullwidth,
    Halfwidth,
    Neutral,
    Narrow,
    Wide,
}

impl EastAsianWidth {
    pub const fn abbreviation(self) -> &'static str {
        match self {
            Self::Ambiguous => "A",
            Self::Fullwidth => "F",
            Self::Halfwidth => "H",
            Self::Neutral => "N",
            Self::Narrow => "Na",
            Self::Wide => "W",
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Ambiguous => "Ambiguous",
            Self::Fullwidth => "Fullwidth",
            Self::Halfwidth => "Halfwidth",
            Self::Neutral => "Neutral",
            Self::Narrow => "Narrow",
            Self::Wide => "Wide",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanonicalCombiningClass {
    value: u8,
    name: &'static str,
}

impl CanonicalCombiningClass {
    pub const fn new(value: u8, name: &'static str) -> Self {
        Self { value, name }
    }

    pub const fn value(self) -> u8 {
        self.value
    }

    pub const fn name(self) -> &'static str {
        self.name
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BidiClass {
    ArabicLetter,
    ArabicNumber,
    ParagraphSeparator,
    BoundaryNeutral,
    CommonSeparator,
    EuropeanNumber,
    EuropeanSeparator,
    EuropeanTerminator,
    FirstStrongIsolate,
    LeftToRight,
    LeftToRightEmbedding,
    LeftToRightIsolate,
    LeftToRightOverride,
    NonspacingMark,
    OtherNeutral,
    PopDirectionalFormat,
    PopDirectionalIsolate,
    RightToLeft,
    RightToLeftEmbedding,
    RightToLeftIsolate,
    RightToLeftOverride,
    SegmentSeparator,
    WhiteSpace,
}

impl BidiClass {
    pub const fn abbreviation(self) -> &'static str {
        match self {
            Self::ArabicLetter => "AL",
            Self::ArabicNumber => "AN",
            Self::ParagraphSeparator => "B",
            Self::BoundaryNeutral => "BN",
            Self::CommonSeparator => "CS",
            Self::EuropeanNumber => "EN",
            Self::EuropeanSeparator => "ES",
            Self::EuropeanTerminator => "ET",
            Self::FirstStrongIsolate => "FSI",
            Self::LeftToRight => "L",
            Self::LeftToRightEmbedding => "LRE",
            Self::LeftToRightIsolate => "LRI",
            Self::LeftToRightOverride => "LRO",
            Self::NonspacingMark => "NSM",
            Self::OtherNeutral => "ON",
            Self::PopDirectionalFormat => "PDF",
            Self::PopDirectionalIsolate => "PDI",
            Self::RightToLeft => "R",
            Self::RightToLeftEmbedding => "RLE",
            Self::RightToLeftIsolate => "RLI",
            Self::RightToLeftOverride => "RLO",
            Self::SegmentSeparator => "S",
            Self::WhiteSpace => "WS",
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::ArabicLetter => "Arabic Letter",
            Self::ArabicNumber => "Arabic Number",
            Self::ParagraphSeparator => "Paragraph Separator",
            Self::BoundaryNeutral => "Boundary Neutral",
            Self::CommonSeparator => "Common Separator",
            Self::EuropeanNumber => "European Number",
            Self::EuropeanSeparator => "European Separator",
            Self::EuropeanTerminator => "European Terminator",
            Self::FirstStrongIsolate => "First Strong Isolate",
            Self::LeftToRight => "Left To Right",
            Self::LeftToRightEmbedding => "Left To Right Embedding",
            Self::LeftToRightIsolate => "Left To Right Isolate",
            Self::LeftToRightOverride => "Left To Right Override",
            Self::NonspacingMark => "Nonspacing Mark",
            Self::OtherNeutral => "Other Neutral",
            Self::PopDirectionalFormat => "Pop Directional Format",
            Self::PopDirectionalIsolate => "Pop Directional Isolate",
            Self::RightToLeft => "Right To Left",
            Self::RightToLeftEmbedding => "Right To Left Embedding",
            Self::RightToLeftIsolate => "Right To Left Isolate",
            Self::RightToLeftOverride => "Right To Left Override",
            Self::SegmentSeparator => "Segment Separator",
            Self::WhiteSpace => "White Space",
        }
    }
}
