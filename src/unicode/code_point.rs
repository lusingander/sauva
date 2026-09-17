use std::{error::Error, fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CodePoint(u32);

impl CodePoint {
    pub const MIN_VALUE: u32 = 0;
    pub const MAX_VALUE: u32 = 0x10_ffff;

    pub const fn new(value: u32) -> Result<Self, InvalidCodePoint> {
        if value <= Self::MAX_VALUE {
            Ok(Self(value))
        } else {
            Err(InvalidCodePoint { value })
        }
    }

    pub const fn value(self) -> u32 {
        self.0
    }

    pub const fn previous(self) -> Option<Self> {
        match self.0.checked_sub(1) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    pub const fn next(self) -> Option<Self> {
        if self.0 < Self::MAX_VALUE {
            Some(Self(self.0 + 1))
        } else {
            None
        }
    }

    pub const fn plane(self) -> u8 {
        (self.0 >> 16) as u8
    }

    pub const fn to_char(self) -> Option<char> {
        char::from_u32(self.0)
    }

    pub const fn structure(self) -> CodePointStructure {
        if matches!(self.0, 0xd800..=0xdfff) {
            CodePointStructure::Surrogate
        } else if matches!(
            self.0,
            0xe000..=0xf8ff | 0xf_0000..=0xf_fffd | 0x10_0000..=0x10_fffd
        ) {
            CodePointStructure::PrivateUse
        } else if matches!(self.0, 0xfdd0..=0xfdef) || self.0 & 0xffff >= 0xfffe {
            CodePointStructure::Noncharacter
        } else {
            CodePointStructure::Other
        }
    }
}

impl TryFrom<u32> for CodePoint {
    type Error = InvalidCodePoint;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<char> for CodePoint {
    fn from(value: char) -> Self {
        Self(value as u32)
    }
}

impl fmt::Display for CodePoint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "U+{:04X}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodePointStructure {
    Other,
    Surrogate,
    PrivateUse,
    Noncharacter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidCodePoint {
    value: u32,
}

impl InvalidCodePoint {
    pub const fn value(self) -> u32 {
        self.value
    }
}

impl fmt::Display for InvalidCodePoint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "code point value U+{:X} is outside U+0000..=U+10FFFF",
            self.value
        )
    }
}

impl Error for InvalidCodePoint {}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[test]
    fn accepts_the_unicode_code_space_boundaries() {
        assert_eq!(CodePoint::new(CodePoint::MIN_VALUE).unwrap().value(), 0);
        assert_eq!(
            CodePoint::new(CodePoint::MAX_VALUE).unwrap().value(),
            0x10_ffff
        );
    }

    #[test]
    fn rejects_values_above_the_unicode_code_space() {
        let error = CodePoint::new(0x11_0000).unwrap_err();

        assert_eq!(error.value(), 0x11_0000);
        assert_eq!(
            error.to_string(),
            "code point value U+110000 is outside U+0000..=U+10FFFF"
        );
    }

    #[test]
    fn moves_to_adjacent_code_points_within_the_unicode_code_space() {
        let code_point = CodePoint::new(0xd7ff).unwrap();

        assert_eq!(code_point.previous().unwrap().value(), 0xd7fe);
        assert_eq!(code_point.next().unwrap().value(), 0xd800);
    }

    #[test]
    fn has_no_adjacent_code_point_beyond_the_unicode_code_space_boundaries() {
        let first = CodePoint::new(CodePoint::MIN_VALUE).unwrap();
        let last = CodePoint::new(CodePoint::MAX_VALUE).unwrap();

        assert_eq!(first.previous(), None);
        assert_eq!(last.next(), None);
    }

    #[rstest]
    #[case(0x0000, 0)]
    #[case(0xffff, 0)]
    #[case(0x1_0000, 1)]
    #[case(0x1_ffff, 1)]
    #[case(0x10_0000, 16)]
    #[case(0x10_ffff, 16)]
    fn calculates_plane_numbers_at_their_boundaries(
        #[case] value: u32,
        #[case] expected_plane: u8,
    ) {
        assert_eq!(CodePoint::new(value).unwrap().plane(), expected_plane);
    }

    #[rstest]
    #[case(0x0000, "U+0000")]
    #[case(0x0041, "U+0041")]
    #[case(0xffff, "U+FFFF")]
    #[case(0x1_0000, "U+10000")]
    #[case(0x10_ffff, "U+10FFFF")]
    fn formats_code_points_with_uppercase_hex_and_at_least_four_digits(
        #[case] value: u32,
        #[case] expected: &str,
    ) {
        assert_eq!(CodePoint::new(value).unwrap().to_string(), expected);
    }

    #[rstest]
    #[case(0xd7ff, Some('\u{d7ff}'))]
    #[case(0xd800, None)]
    #[case(0xdfff, None)]
    #[case(0xe000, Some('\u{e000}'))]
    #[case(0x10_ffff, Some('\u{10ffff}'))]
    fn converts_only_unicode_scalar_values_to_char(
        #[case] value: u32,
        #[case] expected: Option<char>,
    ) {
        assert_eq!(CodePoint::new(value).unwrap().to_char(), expected);
    }

    #[test]
    fn converts_from_scalar_values_without_validation() {
        assert_eq!(CodePoint::from('A'), CodePoint::new(0x0041).unwrap());
        assert_eq!(CodePoint::try_from(0x0041), Ok(CodePoint::from('A')));
    }

    #[rstest]
    #[case(0xd7ff, CodePointStructure::Other)]
    #[case(0xd800, CodePointStructure::Surrogate)]
    #[case(0xdfff, CodePointStructure::Surrogate)]
    #[case(0xe000, CodePointStructure::PrivateUse)]
    fn classifies_surrogate_boundaries(#[case] value: u32, #[case] expected: CodePointStructure) {
        assert_eq!(CodePoint::new(value).unwrap().structure(), expected);
    }

    #[rstest]
    #[case(0xdfff, CodePointStructure::Surrogate)]
    #[case(0xe000, CodePointStructure::PrivateUse)]
    #[case(0xf8ff, CodePointStructure::PrivateUse)]
    #[case(0xf900, CodePointStructure::Other)]
    #[case(0xeffff, CodePointStructure::Noncharacter)]
    #[case(0xf_0000, CodePointStructure::PrivateUse)]
    #[case(0xf_fffd, CodePointStructure::PrivateUse)]
    #[case(0xf_fffe, CodePointStructure::Noncharacter)]
    #[case(0x10_0000, CodePointStructure::PrivateUse)]
    #[case(0x10_fffd, CodePointStructure::PrivateUse)]
    #[case(0x10_fffe, CodePointStructure::Noncharacter)]
    fn classifies_private_use_boundaries(#[case] value: u32, #[case] expected: CodePointStructure) {
        assert_eq!(CodePoint::new(value).unwrap().structure(), expected);
    }

    #[rstest]
    #[case(0xfdcf, CodePointStructure::Other)]
    #[case(0xfdd0, CodePointStructure::Noncharacter)]
    #[case(0xfdef, CodePointStructure::Noncharacter)]
    #[case(0xfdf0, CodePointStructure::Other)]
    fn classifies_noncharacter_boundaries(
        #[case] value: u32,
        #[case] expected: CodePointStructure,
    ) {
        assert_eq!(CodePoint::new(value).unwrap().structure(), expected);
    }

    #[test]
    fn classifies_the_last_two_code_points_of_every_plane_as_noncharacters() {
        for plane in 0..=16 {
            let plane_start = plane << 16;
            assert_eq!(
                CodePoint::new(plane_start | 0xfffe).unwrap().structure(),
                CodePointStructure::Noncharacter
            );
            assert_eq!(
                CodePoint::new(plane_start | 0xffff).unwrap().structure(),
                CodePointStructure::Noncharacter
            );
        }
    }
}
