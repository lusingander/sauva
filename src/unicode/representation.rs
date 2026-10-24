use crate::unicode::{
    CodePoint,
    encoding::{format_hex_bytes, format_hex_code_units},
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TextFormat {
    #[default]
    Text,
    JsonString,
    RustString,
    CodePointList,
    Utf8Hex,
    Utf16Units,
}

impl TextFormat {
    pub const ALL: [Self; 6] = [
        Self::Text,
        Self::JsonString,
        Self::RustString,
        Self::CodePointList,
        Self::Utf8Hex,
        Self::Utf16Units,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Text => "Text",
            Self::JsonString => "JSON String",
            Self::RustString => "Rust String",
            Self::CodePointList => "Code Point List",
            Self::Utf8Hex => "UTF-8 Hex",
            Self::Utf16Units => "UTF-16 Units",
        }
    }

    pub fn format(self, text: &str) -> String {
        self.atoms(text).collect()
    }

    /// Yield complete scalar representations, keeping escapes and surrogate
    /// pairs together so a preview can stop without formatting the whole input.
    pub fn atoms(self, text: &str) -> impl Iterator<Item = String> + '_ {
        let quote = matches!(self, Self::JsonString | Self::RustString).then(|| "\"".to_owned());
        quote
            .clone()
            .into_iter()
            .chain(text.chars().enumerate().map(move |(index, character)| {
                let value = match self {
                    Self::Text => character.to_string(),
                    Self::JsonString => escape_scalar(character, false),
                    Self::RustString => escape_scalar(character, true),
                    Self::CodePointList => CodePoint::from(character).to_string(),
                    Self::Utf8Hex => {
                        let mut buffer = [0; 4];
                        format_hex_bytes(character.encode_utf8(&mut buffer).as_bytes())
                    }
                    Self::Utf16Units => {
                        let mut buffer = [0; 2];
                        format_hex_code_units(character.encode_utf16(&mut buffer))
                    }
                };
                if index > 0
                    && matches!(self, Self::CodePointList | Self::Utf8Hex | Self::Utf16Units)
                {
                    format!(" {value}")
                } else {
                    value
                }
            }))
            .chain(quote)
    }
}

fn escape_scalar(character: char, rust: bool) -> String {
    match character {
        '"' => r#"\""#.to_owned(),
        '\\' => r"\\".to_owned(),
        '\n' => r"\n".to_owned(),
        '\r' => r"\r".to_owned(),
        '\t' => r"\t".to_owned(),
        '\0' if rust => r"\0".to_owned(),
        '\u{8}' if !rust => r"\b".to_owned(),
        '\u{c}' if !rust => r"\f".to_owned(),
        ' '..='~' => character.to_string(),
        _ if rust => format!(r"\u{{{:X}}}", character as u32),
        _ => {
            let mut buffer = [0; 2];
            character
                .encode_utf16(&mut buffer)
                .iter()
                .map(|unit| format!(r"\u{unit:04X}"))
                .collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(TextFormat::Text, "A\u{301}😀")]
    #[case(TextFormat::JsonString, r#""A\u0301\uD83D\uDE00""#)]
    #[case(TextFormat::RustString, r#""A\u{301}\u{1F600}""#)]
    #[case(TextFormat::CodePointList, "U+0041 U+0301 U+1F600")]
    #[case(TextFormat::Utf8Hex, "41 CC 81 F0 9F 98 80")]
    #[case(TextFormat::Utf16Units, "0041 0301 D83D DE00")]
    fn formats_exact_text_without_normalizing(#[case] format: TextFormat, #[case] expected: &str) {
        assert_eq!(format.format("A\u{301}😀"), expected);
    }

    #[test]
    fn escapes_controls_quotes_and_literal_backslashes_in_each_language() {
        let text = "\0\n\r\t\u{8}\u{c}\u{1b}\u{7f}\"\\/\u{a0}\u{202e}\u{feff}";
        assert_eq!(
            TextFormat::JsonString.format(text),
            r#""\u0000\n\r\t\b\f\u001B\u007F\"\\/\u00A0\u202E\uFEFF""#
        );
        assert_eq!(
            TextFormat::RustString.format(text),
            r#""\0\n\r\t\u{8}\u{C}\u{1B}\u{7F}\"\\/\u{A0}\u{202E}\u{FEFF}""#
        );
        for format in [TextFormat::JsonString, TextFormat::RustString] {
            assert_eq!(format.format(r"\u0041"), r#""\\u0041""#);
            assert_eq!(format.format("' / A"), r#""' / A""#);
            assert!(format.format(text).is_ascii());
        }
    }

    #[test]
    fn distinguishes_empty_text_from_quoted_empty_strings() {
        for format in TextFormat::ALL {
            let expected = if matches!(format, TextFormat::JsonString | TextFormat::RustString) {
                r#""""#
            } else {
                ""
            };
            assert_eq!(format.format(""), expected);
        }
    }

    #[test]
    fn encoded_sequences_round_trip_without_losing_line_endings_or_special_scalars() {
        let text = "A\u{301}\r\n👩‍💻\0\u{378}\u{e000}\u{ffff}\u{10ffff}";
        let bytes = TextFormat::Utf8Hex
            .format(text)
            .split(' ')
            .map(|value| u8::from_str_radix(value, 16).unwrap())
            .collect();
        assert_eq!(String::from_utf8(bytes).unwrap(), text);
        let units: Vec<_> = TextFormat::Utf16Units
            .format(text)
            .split(' ')
            .map(|value| u16::from_str_radix(value, 16).unwrap())
            .collect();
        assert_eq!(String::from_utf16(&units).unwrap(), text);
    }

    #[test]
    fn literal_atoms_preserve_quotes_and_keep_surrogate_pairs_together() {
        assert_eq!(
            TextFormat::JsonString.atoms("😀A").collect::<Vec<_>>(),
            [r#"""#, r"\uD83D\uDE00", "A", r#"""#]
        );
        assert_eq!(
            TextFormat::RustString.atoms("😀A").collect::<Vec<_>>(),
            [r#"""#, r"\u{1F600}", "A", r#"""#]
        );
    }
}
