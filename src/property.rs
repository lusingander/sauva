use crate::inspector::InspectorFieldId;

#[derive(Debug, Clone, Copy)]
pub struct PropertyReference {
    pub overview: &'static str,
    pub values_heading: &'static str,
    pub values: &'static [(&'static str, &'static str)],
    pub notes: &'static str,
    pub source: &'static str,
    pub url: &'static str,
}

pub fn reference(id: InspectorFieldId) -> PropertyReference {
    match id {
        InspectorFieldId::Character => PropertyReference {
            overview: "A display representation of the selected code point.",
            values_heading: "Display",
            values: &[
                ("Visible character", "The character itself."),
                (
                    "Display aid",
                    "A label or dotted circle for a character that is invisible or difficult to see alone.",
                ),
                (
                    "Not available",
                    "Surrogate code points are not Unicode scalar values.",
                ),
            ],
            notes: "Display aids are not part of the character. Copying Character in the Inspector copies the actual scalar value, including invisible characters and combining marks.",
            source: "Unicode 17.0 · Display guidelines",
            url: "https://www.unicode.org/versions/Unicode17.0.0/core-spec/chapter-5/",
        },
        InspectorFieldId::CodePoint => PropertyReference {
            overview: "The numeric position of this code point in the Unicode codespace.",
            values_heading: "Format",
            values: &[
                (
                    "U+0000..U+10FFFF",
                    "U+ followed by four to six hexadecimal digits.",
                ),
                (
                    "U+D800..U+DFFF",
                    "Surrogate code points; excluded from Unicode scalar values.",
                ),
            ],
            notes: "A code point is distinct from a glyph, an encoded byte sequence, and a grapheme cluster. A displayed character can contain multiple code points.",
            source: "Unicode 17.0 · Definitions",
            url: "https://www.unicode.org/versions/Unicode17.0.0/core-spec/chapter-3/",
        },
        InspectorFieldId::PrimaryName => PropertyReference {
            overview: "The Unicode name of this character, or a fallback label supplied by Sauva when no Unicode name exists.",
            values_heading: "Values",
            values: &[
                (
                    "Unicode name",
                    "An assigned name, including algorithmically derived names for characters such as Hangul syllables and ideographs.",
                ),
                (
                    "Fallback label",
                    "Identifies a control, private-use, surrogate, noncharacter, or unassigned code point.",
                ),
            ],
            notes: "A fallback label is not a formal Unicode character name. Formal alternate names are listed in Aliases.",
            source: "UAX #44 · Name",
            url: "https://www.unicode.org/reports/tr44/tr44-36.html#Name",
        },
        InspectorFieldId::Aliases => PropertyReference {
            overview: "Formal Unicode name aliases and their types.",
            values_heading: "Alias types",
            values: &[
                ("correction", "Corrects a problem in a character name."),
                ("control", "Identifies a control character."),
                ("alternate", "An alternate name for a format character."),
                (
                    "figment",
                    "A documented name for a control that was never formally standardized.",
                ),
                ("abbreviation", "An abbreviated name."),
                ("None", "No formal name aliases are recorded."),
            ],
            notes: "Aliases are not a list of all common names or synonyms. Sauva can search these aliases.",
            source: "UAX #44 · Name_Alias",
            url: "https://www.unicode.org/reports/tr44/tr44-36.html#Name_Alias",
        },
        InspectorFieldId::Plane => PropertyReference {
            overview: "One of the 17 consecutive regions of the Unicode codespace, each containing 65,536 code points.",
            values_heading: "Values",
            values: &[
                ("0", "Basic Multilingual Plane."),
                ("1", "Supplementary Multilingual Plane."),
                ("2", "Supplementary Ideographic Plane."),
                ("3", "Tertiary Ideographic Plane."),
                ("4..13", "Other supplementary planes."),
                ("14", "Supplementary Special-purpose Plane."),
                ("15..16", "Supplementary Private Use Areas."),
            ],
            notes: "A plane can contain multiple blocks and unassigned code points.",
            source: "Unicode 17.0 · Unicode codespace",
            url: "https://www.unicode.org/versions/Unicode17.0.0/core-spec/chapter-3/",
        },
        InspectorFieldId::Block => PropertyReference {
            overview: "The named Unicode block containing this code point.",
            values_heading: "Values",
            values: &[
                ("Block name", "For example, Basic Latin or Hiragana."),
                ("No Block", "This code point is outside the defined blocks."),
            ],
            notes: "Blocks are ranges of code points, not writing systems. A block can contain multiple scripts, and a script can span multiple blocks. Unassigned code points can still belong to a block.",
            source: "UAX #44 · Block",
            url: "https://www.unicode.org/reports/tr44/tr44-36.html#Block",
        },
        InspectorFieldId::GeneralCategory => PropertyReference {
            overview: "A broad classification of a code point by its primary use. Each code point has one two-letter category.",
            values_heading: "Values · all 30 categories",
            values: &[
                ("L · Letter", ""),
                ("Lu", "Uppercase Letter"),
                ("Ll", "Lowercase Letter"),
                ("Lt", "Titlecase Letter"),
                ("Lm", "Modifier Letter"),
                ("Lo", "Other Letter"),
                ("M · Mark", ""),
                ("Mn", "Nonspacing Mark"),
                ("Mc", "Spacing Mark"),
                ("Me", "Enclosing Mark"),
                ("N · Number", ""),
                ("Nd", "Decimal Number"),
                ("Nl", "Letter Number"),
                ("No", "Other Number"),
                ("P · Punctuation", ""),
                ("Pc", "Connector Punctuation"),
                ("Pd", "Dash Punctuation"),
                ("Ps", "Open Punctuation"),
                ("Pe", "Close Punctuation"),
                ("Pi", "Initial Punctuation"),
                ("Pf", "Final Punctuation"),
                ("Po", "Other Punctuation"),
                ("S · Symbol", ""),
                ("Sm", "Math Symbol"),
                ("Sc", "Currency Symbol"),
                ("Sk", "Modifier Symbol"),
                ("So", "Other Symbol"),
                ("Z · Separator", ""),
                ("Zs", "Space Separator"),
                ("Zl", "Line Separator"),
                ("Zp", "Paragraph Separator"),
                ("C · Other", ""),
                ("Cc", "Control"),
                ("Cf", "Format"),
                ("Cs", "Surrogate"),
                ("Co", "Private Use"),
                ("Cn", "Unassigned"),
            ],
            notes: "One-letter group labels are shown here for organization; the Inspector displays a two-letter value.",
            source: "Unicode 17.0 · §4.5 General Category",
            url: "https://www.unicode.org/versions/Unicode17.0.0/core-spec/chapter-4/#G124142",
        },
        InspectorFieldId::Script => PropertyReference {
            overview: "The writing system associated with this code point.",
            values_heading: "Values",
            values: &[
                ("Script name", "For example, Latin, Hiragana, or Han."),
                (
                    "Common",
                    "Shared across scripts or not specific to one script.",
                ),
                (
                    "Inherited",
                    "Typically takes its script from surrounding text.",
                ),
                ("Unknown", "No script has been assigned."),
            ],
            notes: "Script does not identify a language. This is the Script property; Sauva does not show the separate Script_Extensions property here.",
            source: "UAX #44 · Script",
            url: "https://www.unicode.org/reports/tr44/tr44-36.html#Script",
        },
        InspectorFieldId::Age => PropertyReference {
            overview: "The first Unicode version in which this code point was assigned or designated.",
            values_heading: "Values",
            values: &[
                (
                    "major.minor",
                    "A Unicode version, such as 1.1, 2.1, or 17.0.",
                ),
                (
                    "Unassigned",
                    "Not yet assigned or designated in the bundled Unicode data.",
                ),
            ],
            notes: "Unicode Version elsewhere in the Inspector identifies Sauva's bundled data. It does not describe when this code point was introduced.",
            source: "UAX #44 · §5.14 Character Age",
            url: "https://www.unicode.org/reports/tr44/tr44-36.html#Character_Age",
        },
        InspectorFieldId::EastAsianWidth => PropertyReference {
            overview: "A width classification used in East Asian typography and legacy encoding interoperability.",
            values_heading: "Values",
            values: &[
                (
                    "A",
                    "Ambiguous: Context-dependent narrow or wide treatment.",
                ),
                ("F", "Fullwidth: A fullwidth compatibility form."),
                ("H", "Halfwidth: A halfwidth compatibility form."),
                ("Na", "Narrow: An implicitly narrow character."),
                ("W", "Wide: An implicitly wide character."),
                (
                    "N",
                    "Neutral: Outside the East Asian wide/narrow distinction.",
                ),
            ],
            notes: "This property alone does not determine terminal cell width. Terminal rendering rules and the surrounding text also matter.",
            source: "UAX #11 · §4 Definitions",
            url: "https://www.unicode.org/reports/tr11/tr11-44.html#Definitions",
        },
        InspectorFieldId::CanonicalCombiningClass => PropertyReference {
            overview: "A number used to order combining marks during Unicode normalization.",
            values_heading: "Range and examples",
            values: &[
                (
                    "0..254",
                    "The defined numeric range; many values are unused.",
                ),
                ("0", "Not Reordered: A starter for canonical ordering."),
                ("220", "Below: A common class for marks below a base."),
                ("230", "Above: A common class for marks above a base."),
            ],
            notes: "Class 0 does not mean 'not a combining mark'. The number is not an exact drawing position.",
            source: "Unicode 17.0 · §3.11.2 Combining Classes",
            url: "https://www.unicode.org/versions/Unicode17.0.0/core-spec/chapter-3/#G50313",
        },
        InspectorFieldId::BidiClass => PropertyReference {
            overview: "The directional class used by the Unicode Bidirectional Algorithm.",
            values_heading: "Values",
            values: &[
                ("Strong types", ""),
                ("L", "Left To Right"),
                ("R", "Right To Left"),
                ("AL", "Arabic Letter"),
                ("Weak types", ""),
                ("EN", "European Number"),
                ("ES", "European Separator"),
                ("ET", "European Terminator"),
                ("AN", "Arabic Number"),
                ("CS", "Common Separator"),
                ("NSM", "Nonspacing Mark"),
                ("BN", "Boundary Neutral"),
                ("Neutral types", ""),
                ("B", "Paragraph Separator"),
                ("S", "Segment Separator"),
                ("WS", "White Space"),
                ("ON", "Other Neutral"),
                ("Explicit formatting", ""),
                ("LRE", "Left To Right Embedding"),
                ("RLE", "Right To Left Embedding"),
                ("LRO", "Left To Right Override"),
                ("RLO", "Right To Left Override"),
                ("PDF", "Pop Directional Format"),
                ("LRI", "Left To Right Isolate"),
                ("RLI", "Right To Left Isolate"),
                ("FSI", "First Strong Isolate"),
                ("PDI", "Pop Directional Isolate"),
            ],
            notes: "This class is input to an algorithm that also considers surrounding text. It does not by itself determine the final visual position of a character.",
            source: "UAX #9 · Bidirectional character types",
            url: "https://www.unicode.org/reports/tr9/tr9-51.html#Bidirectional_Character_Types",
        },
        InspectorFieldId::DefaultIgnorable => PropertyReference {
            overview: "Identifies code points normally omitted from fallback display when their intended behavior is unsupported.",
            values_heading: "Values",
            values: &[
                ("Yes", "Included in Default_Ignorable_Code_Point."),
                (
                    "No",
                    "Not included; this does not guarantee a visible glyph.",
                ),
            ],
            notes: "These characters can affect surrounding text. This property is not a recommendation to delete them. Sauva may show a label to make them identifiable.",
            source: "Unicode 17.0 · §5.21 Ignoring Characters in Processing",
            url: "https://www.unicode.org/versions/Unicode17.0.0/core-spec/chapter-5/#G40025",
        },
        InspectorFieldId::UppercaseMapping => PropertyReference {
            overview: "The default full Unicode uppercase mapping of this code point.",
            values_heading: "Format",
            values: &[
                (
                    "Code point sequence",
                    "Read the code points and names in order. A mapping can contain multiple characters.",
                ),
                (
                    "Unchanged",
                    "The sequence contains only the selected code point.",
                ),
                ("Example", "U+00DF (ß) maps to U+0053 U+0053 (SS)."),
            ],
            notes: "Language-specific and context-dependent rules are not applied. This is a mapping for one code point, not language-aware case conversion of a whole string.",
            source: "Unicode 17.0 · §3.13 Default Case Algorithms",
            url: "https://www.unicode.org/versions/Unicode17.0.0/core-spec/chapter-3/",
        },
        InspectorFieldId::LowercaseMapping => PropertyReference {
            overview: "The default full Unicode lowercase mapping of this code point.",
            values_heading: "Format",
            values: &[
                (
                    "Code point sequence",
                    "Read the code points and names in order. A mapping can contain multiple characters.",
                ),
                (
                    "Unchanged",
                    "The sequence contains only the selected code point.",
                ),
                ("Example", "U+0130 maps to U+0069 U+0307."),
            ],
            notes: "Language-specific and context-dependent rules are not applied. This is a mapping for one code point, not language-aware case conversion of a whole string.",
            source: "Unicode 17.0 · §3.13 Default Case Algorithms",
            url: "https://www.unicode.org/versions/Unicode17.0.0/core-spec/chapter-3/",
        },
        InspectorFieldId::TitlecaseMapping => PropertyReference {
            overview: "The default full Unicode titlecase mapping of this code point.",
            values_heading: "Format",
            values: &[
                (
                    "Code point sequence",
                    "Read the code points and names in order. A mapping can contain multiple characters.",
                ),
                (
                    "Unchanged",
                    "The sequence contains only the selected code point.",
                ),
                ("Example", "U+00DF (ß) maps to U+0053 U+0073 (Ss)."),
            ],
            notes: "Language-specific and context-dependent rules are not applied. This is a mapping for one code point, not language-aware case conversion of a whole string.",
            source: "Unicode 17.0 · §3.13 Default Case Algorithms",
            url: "https://www.unicode.org/versions/Unicode17.0.0/core-spec/chapter-3/",
        },
        InspectorFieldId::CaseFolding => PropertyReference {
            overview: "The default full Unicode case-folded mapping of this code point.",
            values_heading: "Format",
            values: &[
                (
                    "Code point sequence",
                    "Read the code points and names in order. A mapping can contain multiple characters.",
                ),
                (
                    "Unchanged",
                    "The sequence contains only the selected code point.",
                ),
                ("Example", "U+00DF (ß) maps to U+0073 U+0073 (ss)."),
            ],
            notes: "Language-specific and context-dependent rules are not applied. Case folding supports caseless matching; it is not the same as lowercasing.",
            source: "Unicode 17.0 · §3.13 Default Case Algorithms",
            url: "https://www.unicode.org/versions/Unicode17.0.0/core-spec/chapter-3/",
        },
        InspectorFieldId::NumericType => PropertyReference {
            overview: "The kind of numeric value associated with this code point.",
            values_heading: "Values",
            values: &[
                (
                    "Decimal",
                    "A decimal digit in a positional decimal digit set.",
                ),
                ("Digit", "Another digit, such as a superscript digit."),
                (
                    "Numeric",
                    "Another numeric character, such as a fraction or numeric ideograph.",
                ),
                (
                    "None",
                    "No numeric value is associated with this code point.",
                ),
            ],
            notes: "These values are distinct from General Category. Check Numeric Value for the exact number.",
            source: "UAX #44 · Numeric_Type",
            url: "https://www.unicode.org/reports/tr44/tr44-36.html#Numeric_Type",
        },
        InspectorFieldId::NumericValue => PropertyReference {
            overview: "The exact numeric value associated with this code point.",
            values_heading: "Format",
            values: &[
                ("Integer", "For example, 0, 4, or 10000."),
                (
                    "Fraction",
                    "An exact rational value, such as 1/2; negative values are possible.",
                ),
                (
                    "NaN",
                    "No numeric value is associated with this code point.",
                ),
            ],
            notes: "Numeric Value is not the hexadecimal code point or an encoding. Sauva preserves exact numbers and fractions instead of rounding them to floating point.",
            source: "UAX #44 · Numeric_Value",
            url: "https://www.unicode.org/reports/tr44/tr44-36.html#Numeric_Value",
        },
        InspectorFieldId::Utf8 => PropertyReference {
            overview: "The UTF-8 encoding of this Unicode scalar value, displayed as hexadecimal bytes.",
            values_heading: "Format",
            values: &[
                (
                    "1–4 bytes",
                    "Two hexadecimal digits per byte, separated by spaces.",
                ),
                ("Examples", "A → 41; é → C3 A9; 😀 → F0 9F 98 80."),
                (
                    "Not available",
                    "Surrogate code points U+D800..U+DFFF have no valid UTF-8 encoding.",
                ),
            ],
            notes: "These are encoded bytes, not the hexadecimal code point value.",
            source: "RFC 3629 · §3 UTF-8 definition",
            url: "https://www.rfc-editor.org/rfc/rfc3629.html#section-3",
        },
        InspectorFieldId::Utf16 => PropertyReference {
            overview: "The UTF-16 encoding of this Unicode scalar value, displayed as hexadecimal code units.",
            values_heading: "Format",
            values: &[
                ("BMP scalar", "One four-digit code unit."),
                (
                    "Supplementary scalar",
                    "A pair of four-digit code units, separated by a space.",
                ),
                (
                    "Not available",
                    "An isolated surrogate code point cannot be encoded as a Unicode scalar value.",
                ),
            ],
            notes: "The displayed units do not select a byte order. UTF-16 byte order matters when those units are serialized as bytes.",
            source: "Unicode 17.0 · §3.9.2 UTF-16",
            url: "https://www.unicode.org/versions/Unicode17.0.0/core-spec/chapter-3/",
        },
        InspectorFieldId::HtmlDecimal => PropertyReference {
            overview: "A decimal numeric character reference for this Unicode scalar value.",
            values_heading: "Format",
            values: &[
                (
                    "&#number;",
                    "The code point in decimal, enclosed by &# and a semicolon.",
                ),
                ("Example", "A is &#65;."),
                (
                    "Not available",
                    "Surrogate code points are not Unicode scalar values.",
                ),
            ],
            notes: "This is a generated representation, not the HTML parser result. HTML parsing has special handling for some numeric references, including U+0000 and certain control codes.",
            source: "HTML · Character references",
            url: "https://html.spec.whatwg.org/multipage/syntax.html#character-references",
        },
        InspectorFieldId::HtmlHex => PropertyReference {
            overview: "A hexadecimal numeric character reference for this Unicode scalar value.",
            values_heading: "Format",
            values: &[
                (
                    "&#xHEX;",
                    "The code point in hexadecimal, enclosed by &#x and a semicolon.",
                ),
                ("Example", "A is &#x41;."),
                (
                    "Not available",
                    "Surrogate code points are not Unicode scalar values.",
                ),
            ],
            notes: "This is a generated representation, not the HTML parser result. HTML parsing has special handling for some numeric references, including U+0000 and certain control codes.",
            source: "HTML · Character references",
            url: "https://html.spec.whatwg.org/multipage/syntax.html#character-references",
        },
        InspectorFieldId::RustChar => PropertyReference {
            overview: "A Rust character literal using a Unicode escape for this scalar value.",
            values_heading: "Format",
            values: &[
                (
                    "'\\u{HEX}'",
                    "A hexadecimal Unicode scalar value inside braces and single quotes.",
                ),
                ("Example", "A is '\\u{41}'."),
                (
                    "Not available",
                    "Rust char values exclude surrogate code points.",
                ),
            ],
            notes: "A Rust char contains one Unicode scalar value. A grapheme cluster can require multiple char values.",
            source: "Rust Reference · Character literals",
            url: "https://doc.rust-lang.org/reference/tokens.html#character-literals",
        },
        InspectorFieldId::UnicodeEscape => PropertyReference {
            overview: "An escaped code point representation displayed by Sauva.",
            values_heading: "Format",
            values: &[
                ("\\uHHHH", "Four hexadecimal digits for a BMP scalar."),
                (
                    "\\UHHHHHHHH",
                    "Eight hexadecimal digits for a supplementary scalar.",
                ),
                (
                    "Not available",
                    "Surrogate code points are not Unicode scalar values.",
                ),
            ],
            notes: "This notation is not accepted unchanged by every programming language. Rust char shows Rust syntax, and HTML fields show numeric character references.",
            source: "Unicode 17.0 · Encoding guidelines",
            url: "https://www.unicode.org/versions/Unicode17.0.0/core-spec/chapter-5/",
        },
        InspectorFieldId::DecompositionType => PropertyReference {
            overview: "Whether the decomposition mapping is canonical or compatibility-based.",
            values_heading: "Values",
            values: &[
                ("Canonical", "A mapping used in canonical decomposition."),
                (
                    "Compatibility",
                    "A mapping that may remove distinctions such as formatting or width; Sauva also shows its tag.",
                ),
                ("None", "No decomposition mapping is shown."),
            ],
            notes: "NFC and NFD use canonical decomposition. NFKC and NFKD also use compatibility mappings. Decomposition shows the mapped code points, not a complete normalization result.",
            source: "UAX #15 · Normalization forms",
            url: "https://www.unicode.org/reports/tr15/tr15-57.html#Normalization_Forms",
        },
        InspectorFieldId::Decomposition => PropertyReference {
            overview: "The decomposition mapping shown by Sauva for this code point.",
            values_heading: "Reading the value",
            values: &[
                (
                    "Code point sequence",
                    "Read in order. Each line identifies a code point and its name.",
                ),
                (
                    "None",
                    "No decomposition mapping is shown for this code point.",
                ),
            ],
            notes: "A mapping can contain characters that decompose further. Normalization recursively decomposes and reorders; it may also compose. See Decomposition Type for canonical versus compatibility mappings, and Sequence's Normalization view for full results.",
            source: "UAX #15 · §1.3 Normalization Process",
            url: "https://www.unicode.org/reports/tr15/tr15-57.html#Description_Norm",
        },
        InspectorFieldId::UnicodeVersion => PropertyReference {
            overview: "The version of the Unicode data bundled with this Sauva build.",
            values_heading: "Format",
            values: &[("major.minor.update", "For example, 17.0.0.")],
            notes: "This describes the bundled data, not the terminal, font, or system Unicode support. Age is the version in which the selected code point was first assigned or designated.",
            source: "Unicode 17.0.0",
            url: "https://www.unicode.org/versions/Unicode17.0.0/",
        },
    }
}
