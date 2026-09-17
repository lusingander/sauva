use std::{error::Error, fmt};

use crate::unicode::{CodePoint, InvalidCodePoint};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodePointNotationError {
    Empty,
    InvalidFormat,
    OutOfRange { value: u32 },
}

impl fmt::Display for CodePointNotationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("enter a code point"),
            Self::InvalidFormat => formatter.write_str("use U+ followed by 1-6 hexadecimal digits"),
            Self::OutOfRange { .. } => {
                write!(
                    formatter,
                    "code point must be between U+{:04X} and U+{:X}",
                    CodePoint::MIN_VALUE,
                    CodePoint::MAX_VALUE
                )
            }
        }
    }
}

impl Error for CodePointNotationError {}

impl From<InvalidCodePoint> for CodePointNotationError {
    fn from(error: InvalidCodePoint) -> Self {
        Self::OutOfRange {
            value: error.value(),
        }
    }
}

pub fn parse_code_point_notation(input: &str) -> Result<CodePoint, CodePointNotationError> {
    let notation = input.trim_matches(|character: char| character.is_ascii_whitespace());
    if notation.is_empty() {
        return Err(CodePointNotationError::Empty);
    }

    let digits = notation
        .strip_prefix("U+")
        .or_else(|| notation.strip_prefix("u+"))
        .unwrap_or(notation);
    if digits.is_empty() || digits.len() > 6 || !digits.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(CodePointNotationError::InvalidFormat);
    }

    let value =
        u32::from_str_radix(digits, 16).map_err(|_| CodePointNotationError::InvalidFormat)?;
    CodePoint::new(value).map_err(CodePointNotationError::from)
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("U+0041", 0x0041)]
    #[case("u+1f600", 0x1f600)]
    #[case("0041", 0x0041)]
    #[case("a", 0x000a)]
    #[case("10FFFF", 0x10_ffff)]
    fn accepts_prefixed_and_bare_hexadecimal_notation(#[case] input: &str, #[case] expected: u32) {
        assert_eq!(parse_code_point_notation(input).unwrap().value(), expected);
    }

    #[test]
    fn trims_only_surrounding_ascii_whitespace() {
        let code_point = parse_code_point_notation(" \tU+2192\r\n").unwrap();

        assert_eq!(code_point.value(), 0x2192);
    }

    #[rstest]
    #[case("a", "U+000A")]
    #[case("u+1f600", "U+1F600")]
    fn returns_normalized_notation_through_code_point_display(
        #[case] input: &str,
        #[case] expected: &str,
    ) {
        assert_eq!(
            parse_code_point_notation(input).unwrap().to_string(),
            expected
        );
    }

    #[rstest]
    #[case("")]
    #[case(" ")]
    #[case("\t\r\n")]
    fn rejects_empty_input(#[case] input: &str) {
        assert_eq!(
            parse_code_point_notation(input),
            Err(CodePointNotationError::Empty)
        );
    }

    #[rstest]
    #[case("U+")]
    #[case("U+G")]
    #[case("+0041")]
    #[case("-1")]
    #[case("0x41")]
    #[case("U+00_41")]
    #[case("U+00 41")]
    #[case("0000041")]
    #[case("Ｕ＋0041")]
    #[case("\u{a0}U+0041")]
    fn rejects_invalid_notation(#[case] input: &str) {
        assert_eq!(
            parse_code_point_notation(input),
            Err(CodePointNotationError::InvalidFormat)
        );
    }

    #[rstest]
    #[case("U+110000", 0x11_0000)]
    #[case("FFFFFF", 0xff_ffff)]
    fn distinguishes_values_outside_the_unicode_code_space(
        #[case] input: &str,
        #[case] value: u32,
    ) {
        assert_eq!(
            parse_code_point_notation(input),
            Err(CodePointNotationError::OutOfRange { value })
        );
    }

    #[test]
    fn provides_stable_user_facing_error_messages() {
        assert_eq!(
            CodePointNotationError::Empty.to_string(),
            "enter a code point"
        );
        assert_eq!(
            CodePointNotationError::InvalidFormat.to_string(),
            "use U+ followed by 1-6 hexadecimal digits"
        );
        assert_eq!(
            CodePointNotationError::OutOfRange { value: 0x11_0000 }.to_string(),
            "code point must be between U+0000 and U+10FFFF"
        );
    }
}
