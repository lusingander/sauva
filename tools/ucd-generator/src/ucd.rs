use std::{error::Error, fmt};

const MAX_CODE_POINT: u32 = 0x10_ffff;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodePointRange {
    start: u32,
    end: u32,
}

impl CodePointRange {
    pub const fn start(self) -> u32 {
        self.start
    }

    pub const fn end(self) -> u32 {
        self.end
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EntryKind {
    Data,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UcdEntry {
    range: CodePointRange,
    fields: Vec<String>,
    kind: EntryKind,
}

impl UcdEntry {
    pub const fn range(&self) -> CodePointRange {
        self.range
    }

    pub fn fields(&self) -> &[String] {
        &self.fields
    }

    pub const fn is_missing(&self) -> bool {
        matches!(self.kind, EntryKind::Missing)
    }
}

pub fn parse(source: &str) -> Result<Vec<UcdEntry>, ParseError> {
    source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| match parse_line(index + 1, line) {
            Ok(Some(entry)) => Some(Ok(entry)),
            Ok(None) => None,
            Err(error) => Some(Err(error)),
        })
        .collect()
}

fn parse_line(line_number: usize, line: &str) -> Result<Option<UcdEntry>, ParseError> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    let (kind, content) = if let Some(comment) = trimmed.strip_prefix('#') {
        let comment = comment.trim_start();
        let Some(missing) = comment.strip_prefix("@missing:") else {
            return Ok(None);
        };
        (EntryKind::Missing, missing.trim())
    } else {
        (
            EntryKind::Data,
            trimmed
                .split_once('#')
                .map_or(trimmed, |item| item.0)
                .trim(),
        )
    };

    if content.is_empty() {
        return Err(ParseError::new(line_number, "entry is empty"));
    }

    let mut columns = content.split(';');
    let range_text = columns.next().expect("split always yields one item").trim();
    let fields = columns
        .enumerate()
        .map(|(index, field)| {
            let field = field.trim();
            if field.is_empty() {
                Err(ParseError::new(
                    line_number,
                    format!("field {} is empty", index + 1),
                ))
            } else {
                Ok(field.to_owned())
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    if fields.is_empty() {
        return Err(ParseError::new(
            line_number,
            "expected a semicolon and at least one field",
        ));
    }

    Ok(Some(UcdEntry {
        range: parse_range(line_number, range_text)?,
        fields,
        kind,
    }))
}

fn parse_range(line_number: usize, text: &str) -> Result<CodePointRange, ParseError> {
    let mut bounds = text.split("..");
    let start_text = bounds.next().expect("split always yields one item").trim();
    let end_text = bounds.next().map(str::trim);

    if bounds.next().is_some() || start_text.is_empty() || end_text == Some("") {
        return Err(ParseError::new(
            line_number,
            format!("invalid code point range `{text}`"),
        ));
    }

    let start = parse_code_point(line_number, start_text)?;
    let end = match end_text {
        Some(value) => parse_code_point(line_number, value)?,
        None => start,
    };

    if start > end {
        return Err(ParseError::new(
            line_number,
            format!("code point range starts after it ends: `{text}`"),
        ));
    }

    Ok(CodePointRange { start, end })
}

fn parse_code_point(line_number: usize, text: &str) -> Result<u32, ParseError> {
    let value = u32::from_str_radix(text, 16).map_err(|_| {
        ParseError::new(
            line_number,
            format!("invalid hexadecimal code point `{text}`"),
        )
    })?;

    if value > MAX_CODE_POINT {
        return Err(ParseError::new(
            line_number,
            format!("code point `{text}` is above U+10FFFF"),
        ));
    }

    Ok(value)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    line: usize,
    message: String,
}

impl ParseError {
    fn new(line: usize, message: impl Into<String>) -> Self {
        Self {
            line,
            message: message.into(),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "line {}: {}", self.line, self.message)
    }
}

impl Error for ParseError {}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[test]
    fn parses_comments_single_values_ranges_and_missing_directives() {
        let entries = parse(include_str!("../tests/fixtures/valid.txt")).unwrap();

        assert_eq!(entries.len(), 3);
        assert_eq!(
            entries[0],
            UcdEntry {
                range: CodePointRange {
                    start: 0x0041,
                    end: 0x0041,
                },
                fields: vec!["Lu".to_owned()],
                kind: EntryKind::Data,
            }
        );
        assert_eq!(entries[1].range.start, 0x3400);
        assert_eq!(entries[1].range.end, 0x4dbf);
        assert_eq!(entries[1].fields(), ["Lo"]);
        assert!(!entries[1].is_missing());
        assert_eq!(entries[2].range.start, 0x0000);
        assert_eq!(entries[2].range.end, 0x10_ffff);
        assert_eq!(entries[2].fields(), ["InCB", "None"]);
        assert!(entries[2].is_missing());
    }

    #[rstest]
    #[case(
        include_str!("../tests/fixtures/invalid-hex.txt"),
        2,
        "line 2: invalid hexadecimal code point `XYZ`"
    )]
    #[case(
        include_str!("../tests/fixtures/reversed-range.txt"),
        1,
        "line 1: code point range starts after it ends: `0042..0041`"
    )]
    #[case(
        include_str!("../tests/fixtures/out-of-range.txt"),
        1,
        "line 1: code point `110000` is above U+10FFFF"
    )]
    #[case(
        include_str!("../tests/fixtures/missing-field.txt"),
        1,
        "line 1: expected a semicolon and at least one field"
    )]
    #[case(
        include_str!("../tests/fixtures/empty-field.txt"),
        1,
        "line 1: field 1 is empty"
    )]
    fn reports_fixture_errors_with_line_numbers(
        #[case] fixture: &str,
        #[case] expected_line: usize,
        #[case] expected_message: &str,
    ) {
        let error = parse(fixture).unwrap_err();

        assert_eq!(error.line, expected_line);
        assert_eq!(error.to_string(), expected_message);
    }
}
