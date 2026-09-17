use std::fmt::Write;

use crate::unicode::CodePoint;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnicodeScalarEncoding {
    scalar: char,
}

impl UnicodeScalarEncoding {
    pub const fn for_code_point(code_point: CodePoint) -> Option<Self> {
        match code_point.to_char() {
            Some(scalar) => Some(Self { scalar }),
            None => None,
        }
    }

    pub fn utf8(self) -> String {
        let mut buffer = [0; 4];
        let encoded = self.scalar.encode_utf8(&mut buffer);
        format_hex_bytes(encoded.as_bytes())
    }

    pub fn utf16(self) -> String {
        let mut buffer = [0; 2];
        let encoded = self.scalar.encode_utf16(&mut buffer);
        format_hex_code_units(encoded)
    }

    pub fn html_decimal(self) -> String {
        format!("&#{};", self.scalar as u32)
    }

    pub fn html_hex(self) -> String {
        format!("&#x{:X};", self.scalar as u32)
    }

    pub fn rust_char(self) -> String {
        format!("'\\u{{{:X}}}'", self.scalar as u32)
    }

    pub fn unicode_escape(self) -> String {
        let value = self.scalar as u32;
        if value <= 0xffff {
            format!("\\u{value:04X}")
        } else {
            format!("\\U{value:08X}")
        }
    }
}

fn format_hex_bytes(values: &[u8]) -> String {
    let mut output = String::with_capacity(values.len() * 3 - 1);
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            output.push(' ');
        }
        write!(&mut output, "{value:02X}").expect("writing to a string cannot fail");
    }
    output
}

fn format_hex_code_units(values: &[u16]) -> String {
    let mut output = String::with_capacity(values.len() * 5 - 1);
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            output.push(' ');
        }
        write!(&mut output, "{value:04X}").expect("writing to a string cannot fail");
    }
    output
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::unicode::{CodePoint, UnicodeScalarEncoding};

    #[rustfmt::skip]
    #[rstest]
    #[case(
        0x0041,
        "41",
        "0041",
        "&#65;",
        "&#x41;",
        "'\\u{41}'",
        "\\u0041"
    )]
    #[case(
        0x2192,
        "E2 86 92",
        "2192",
        "&#8594;",
        "&#x2192;",
        "'\\u{2192}'",
        "\\u2192"
    )]
    #[case(
        0x1f600,
        "F0 9F 98 80",
        "D83D DE00",
        "&#128512;",
        "&#x1F600;",
        "'\\u{1F600}'",
        "\\U0001F600"
    )]
    #[case(
        0x10ffff,
        "F4 8F BF BF",
        "DBFF DFFF",
        "&#1114111;",
        "&#x10FFFF;",
        "'\\u{10FFFF}'",
        "\\U0010FFFF"
    )]
    fn formats_scalar_value_representations(
        #[case] value: u32,
        #[case] utf8: &str,
        #[case] utf16: &str,
        #[case] html_decimal: &str,
        #[case] html_hex: &str,
        #[case] rust_char: &str,
        #[case] unicode_escape: &str,
    ) {
        let encoding =
            UnicodeScalarEncoding::for_code_point(CodePoint::new(value).unwrap()).unwrap();

        assert_eq!(encoding.utf8(), utf8);
        assert_eq!(encoding.utf16(), utf16);
        assert_eq!(encoding.html_decimal(), html_decimal);
        assert_eq!(encoding.html_hex(), html_hex);
        assert_eq!(encoding.rust_char(), rust_char);
        assert_eq!(encoding.unicode_escape(), unicode_escape);
    }

    #[rstest]
    #[case(0xd800)]
    #[case(0xdfff)]
    fn rejects_surrogates(#[case] value: u32) {
        assert!(UnicodeScalarEncoding::for_code_point(CodePoint::new(value).unwrap()).is_none());
    }
}
