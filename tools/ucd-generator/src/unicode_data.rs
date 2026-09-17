use std::{error::Error, fmt};

const FIELD_COUNT: usize = 15;
const MAX_CODE_POINT: u32 = 0x10_ffff;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecompositionEntry {
    code_point: u32,
    kind: DecompositionKind,
    mapping: Vec<u32>,
}

impl DecompositionEntry {
    pub const fn code_point(&self) -> u32 {
        self.code_point
    }

    pub const fn kind(&self) -> &DecompositionKind {
        &self.kind
    }

    pub fn mapping(&self) -> &[u32] {
        &self.mapping
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecompositionKind {
    Canonical,
    Compatibility(String),
}

pub fn parse_decomposition_entries(source: &str) -> Result<Vec<DecompositionEntry>, ParseError> {
    let mut entries = Vec::new();
    let mut previous = None;

    for (index, line) in source.lines().enumerate() {
        let line_number = index + 1;
        if line.trim().is_empty() {
            continue;
        }

        let fields = line.split(';').collect::<Vec<_>>();
        if fields.len() != FIELD_COUNT {
            return Err(ParseError::new(
                line_number,
                format!("expected {FIELD_COUNT} fields, found {}", fields.len()),
            ));
        }

        let code_point = parse_code_point(line_number, fields[0].trim())?;
        if previous.is_some_and(|value| code_point <= value) {
            return Err(ParseError::new(
                line_number,
                format!(
                    "code points must be strictly increasing after U+{:04X}",
                    previous.expect("the previous code point exists")
                ),
            ));
        }
        previous = Some(code_point);

        let decomposition = fields[5].trim();
        if decomposition.is_empty() {
            continue;
        }
        entries.push(parse_decomposition(line_number, code_point, decomposition)?);
    }

    Ok(entries)
}

fn parse_decomposition(
    line_number: usize,
    code_point: u32,
    source: &str,
) -> Result<DecompositionEntry, ParseError> {
    let mut values = source.split_whitespace();
    let first = values
        .next()
        .expect("a non-empty decomposition has at least one value");
    let (kind, first_mapping) = if first.starts_with('<') {
        let Some(tag) = first
            .strip_prefix('<')
            .and_then(|value| value.strip_suffix('>'))
        else {
            return Err(ParseError::new(
                line_number,
                format!("invalid decomposition tag `{first}`"),
            ));
        };
        if tag.is_empty() {
            return Err(ParseError::new(
                line_number,
                "decomposition tag must not be empty",
            ));
        }
        (DecompositionKind::Compatibility(tag.to_owned()), None)
    } else {
        (DecompositionKind::Canonical, Some(first))
    };

    let mapping = first_mapping
        .into_iter()
        .chain(values)
        .map(|value| parse_code_point(line_number, value))
        .collect::<Result<Vec<_>, _>>()?;
    if mapping.is_empty() {
        return Err(ParseError::new(
            line_number,
            "decomposition mapping must contain at least one code point",
        ));
    }

    Ok(DecompositionEntry {
        code_point,
        kind,
        mapping,
    })
}

fn parse_code_point(line_number: usize, source: &str) -> Result<u32, ParseError> {
    let value = u32::from_str_radix(source, 16).map_err(|_| {
        ParseError::new(
            line_number,
            format!("invalid hexadecimal code point `{source}`"),
        )
    })?;
    if value > MAX_CODE_POINT {
        return Err(ParseError::new(
            line_number,
            format!("code point `{source}` is above U+10FFFF"),
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

    use crate::unicode_data::{DecompositionEntry, DecompositionKind, parse_decomposition_entries};

    #[test]
    fn parses_only_canonical_and_compatibility_decompositions() {
        let entries =
            parse_decomposition_entries(include_str!("../tests/fixtures/unicode-data-valid.txt"))
                .unwrap();

        assert_eq!(
            entries,
            [
                DecompositionEntry {
                    code_point: 0x00a0,
                    kind: DecompositionKind::Compatibility("noBreak".to_owned()),
                    mapping: vec![0x0020],
                },
                DecompositionEntry {
                    code_point: 0x00e9,
                    kind: DecompositionKind::Canonical,
                    mapping: vec![0x0065, 0x0301],
                },
            ]
        );
    }

    #[rstest]
    #[case(
        "unicode-data-wrong-field-count.txt",
        include_str!("../tests/fixtures/unicode-data-wrong-field-count.txt"),
        "line 1: expected 15 fields, found 14"
    )]
    #[case(
        "unicode-data-invalid-tag.txt",
        include_str!("../tests/fixtures/unicode-data-invalid-tag.txt"),
        "line 1: invalid decomposition tag `<compat`"
    )]
    #[case(
        "unicode-data-empty-mapping.txt",
        include_str!("../tests/fixtures/unicode-data-empty-mapping.txt"),
        "line 1: decomposition mapping must contain at least one code point"
    )]
    #[case(
        "unicode-data-out-of-order.txt",
        include_str!("../tests/fixtures/unicode-data-out-of-order.txt"),
        "line 2: code points must be strictly increasing after U+00E9"
    )]
    fn reports_invalid_unicode_data_with_physical_line_numbers(
        #[case] _fixture_name: &str,
        #[case] fixture: &str,
        #[case] expected: &str,
    ) {
        assert_eq!(
            parse_decomposition_entries(fixture)
                .unwrap_err()
                .to_string(),
            expected
        );
    }
}
