use std::{
    io::{self, Write},
    ops::Range,
};

use serde::Serialize;
use serde_json::ser::{CharEscape, Formatter, PrettyFormatter, Serializer};

use crate::{
    cli::LaunchTarget,
    unicode::{
        CaseMapping, CodePoint, DecompositionType, UnicodeDatabase,
        representation::escape_json_scalar,
        text::{NormalizationForm, TextAnalysis, TextMetrics},
    },
};

pub fn write(target: &LaunchTarget, writer: impl Write) -> io::Result<()> {
    let source = match target {
        LaunchTarget::CodePoint(point) => match point.to_char() {
            Some(character) => character.to_string(),
            None => return Report::for_surrogate(*point).write(writer),
        },
        LaunchTarget::Sequence(text) => text.clone(),
    };
    let analysis = TextAnalysis::new(source);
    Report::for_text(&analysis).write(writer)
}

#[derive(Serialize)]
struct Report<'a> {
    schema_version: u8,
    sauva_version: &'static str,
    unicode_version: &'static str,
    text: Option<&'a str>,
    metrics: Option<TextMetrics>,
    code_points: Vec<ReportedCodePoint>,
    graphemes: Option<Vec<ReportedGrapheme>>,
    normalization: Option<Normalization<'a>>,
}

impl<'a> Report<'a> {
    fn for_text(analysis: &'a TextAnalysis) -> Self {
        let code_points = analysis
            .code_points()
            .iter()
            .enumerate()
            .map(|(index, point)| {
                let mut entry = ReportedCodePoint::new(index, point.code_point());
                entry.utf8_range = Some(range(point.byte_range()));
                entry.utf16_range = Some(range(point.utf16_range()));
                entry.grapheme_index = Some(point.grapheme_index());
                entry
            })
            .collect();
        let graphemes = analysis
            .graphemes()
            .iter()
            .enumerate()
            .map(|(index, cluster)| ReportedGrapheme {
                index,
                code_point_range: range(cluster.code_point_range()),
                utf8_range: range(cluster.byte_range()),
                utf16_range: range(cluster.utf16_range()),
            })
            .collect();
        let [nfc, nfd, nfkc, nfkd] = NormalizationForm::ALL.map(|form| {
            let result = analysis.normalization().get(form);
            NormalizationResult {
                changed: result.is_changed(),
                text: analysis.normalized_text(form),
                metrics: result.metrics(),
            }
        });
        Self {
            schema_version: 1,
            sauva_version: env!("CARGO_PKG_VERSION"),
            unicode_version: UnicodeDatabase::version(),
            text: Some(analysis.source()),
            metrics: Some(analysis.metrics()),
            code_points,
            graphemes: Some(graphemes),
            normalization: Some(Normalization {
                nfc,
                nfd,
                nfkc,
                nfkd,
            }),
        }
    }

    fn for_surrogate(point: CodePoint) -> Self {
        Self {
            schema_version: 1,
            sauva_version: env!("CARGO_PKG_VERSION"),
            unicode_version: UnicodeDatabase::version(),
            text: None,
            metrics: None,
            code_points: vec![ReportedCodePoint::new(0, point)],
            graphemes: None,
            normalization: None,
        }
    }

    fn write(&self, writer: impl Write) -> io::Result<()> {
        self.serialize(&mut Serializer::with_formatter(
            writer,
            AsciiFormatter::default(),
        ))
        .map_err(io::Error::from)
    }
}

fn range(value: Range<usize>) -> [usize; 2] {
    [value.start, value.end]
}

#[derive(Serialize)]
struct ReportedCodePoint {
    index: usize,
    value: u32,
    code_point: String,
    character: Option<char>,
    utf8_range: Option<[usize; 2]>,
    utf16_range: Option<[usize; 2]>,
    grapheme_index: Option<usize>,
    properties: Properties,
}

impl ReportedCodePoint {
    fn new(index: usize, point: CodePoint) -> Self {
        Self {
            index,
            value: point.value(),
            code_point: point.to_string(),
            character: point.to_char(),
            utf8_range: None,
            utf16_range: None,
            grapheme_index: None,
            properties: Properties::new(point),
        }
    }
}

#[derive(Serialize)]
struct ReportedGrapheme {
    index: usize,
    code_point_range: [usize; 2],
    utf8_range: [usize; 2],
    utf16_range: [usize; 2],
}

#[derive(Serialize)]
struct Properties {
    name: Option<&'static str>,
    aliases: Vec<Alias>,
    plane: u8,
    block: Option<&'static str>,
    general_category: &'static str,
    script: &'static str,
    age: Option<&'static str>,
    east_asian_width: &'static str,
    canonical_combining_class: u8,
    bidi_class: &'static str,
    default_ignorable: bool,
    case_mapping: ReportedCaseMapping,
    numeric_type: &'static str,
    numeric_value: Option<&'static str>,
    decomposition: Option<ReportedDecomposition>,
    encoding: Option<Encoding>,
}

impl Properties {
    fn new(point: CodePoint) -> Self {
        let record = UnicodeDatabase::lookup(point);
        let aliases = record
            .name_aliases()
            .iter()
            .map(|alias| Alias {
                name: alias.name(),
                kind: alias.kind().label(),
            })
            .collect();
        let decomposition = record
            .decomposition()
            .map(|decomposition| ReportedDecomposition {
                kind: match decomposition.decomposition_type() {
                    DecompositionType::Canonical => "canonical",
                    DecompositionType::Compatibility(tag) => tag,
                },
                mapping: decomposition.mapping().map(CodePoint::value).collect(),
            });
        let encoding = point.to_char().map(|character| Encoding {
            utf8: character.encode_utf8(&mut [0; 4]).as_bytes().to_vec(),
            utf16: character.encode_utf16(&mut [0; 2]).to_vec(),
        });
        Self {
            name: record.primary_name(),
            aliases,
            plane: point.plane(),
            block: record.block(),
            general_category: record.general_category().abbreviation(),
            script: record.script(),
            age: record.age(),
            east_asian_width: record.east_asian_width().abbreviation(),
            canonical_combining_class: record.canonical_combining_class().value(),
            bidi_class: record.bidi_class().abbreviation(),
            default_ignorable: record.is_default_ignorable(),
            case_mapping: ReportedCaseMapping {
                uppercase: mapping(record.uppercase_mapping()),
                lowercase: mapping(record.lowercase_mapping()),
                titlecase: mapping(record.titlecase_mapping()),
                case_folding: mapping(record.case_folding()),
            },
            numeric_type: record.numeric_type().name(),
            numeric_value: record.numeric_value(),
            decomposition,
            encoding,
        }
    }
}

fn mapping(value: CaseMapping) -> Vec<u32> {
    value.mapping().map(CodePoint::value).collect()
}

#[derive(Serialize)]
struct Alias {
    name: &'static str,
    #[serde(rename = "type")]
    kind: &'static str,
}

#[derive(Serialize)]
struct ReportedCaseMapping {
    uppercase: Vec<u32>,
    lowercase: Vec<u32>,
    titlecase: Vec<u32>,
    case_folding: Vec<u32>,
}

#[derive(Serialize)]
struct ReportedDecomposition {
    #[serde(rename = "type")]
    kind: &'static str,
    mapping: Vec<u32>,
}

#[derive(Serialize)]
struct Encoding {
    utf8: Vec<u8>,
    utf16: Vec<u16>,
}

#[derive(Serialize)]
struct Normalization<'a> {
    nfc: NormalizationResult<'a>,
    nfd: NormalizationResult<'a>,
    nfkc: NormalizationResult<'a>,
    nfkd: NormalizationResult<'a>,
}

#[derive(Serialize)]
struct NormalizationResult<'a> {
    changed: bool,
    text: &'a str,
    metrics: TextMetrics,
}

// Delegate layout to serde_json while sharing the literal format's escaping for
// every string, including keys and Unicode property values.
#[derive(Default)]
struct AsciiFormatter {
    pretty: PrettyFormatter<'static>,
}

impl Formatter for AsciiFormatter {
    fn write_string_fragment<W: ?Sized + Write>(
        &mut self,
        writer: &mut W,
        fragment: &str,
    ) -> io::Result<()> {
        if fragment.bytes().all(|byte| (b' '..=b'~').contains(&byte)) {
            return writer.write_all(fragment.as_bytes());
        }
        for character in fragment.chars() {
            writer.write_all(escape_json_scalar(character).as_bytes())?;
        }
        Ok(())
    }

    fn write_char_escape<W: ?Sized + Write>(
        &mut self,
        writer: &mut W,
        escape: CharEscape,
    ) -> io::Result<()> {
        let character = match escape {
            CharEscape::Quote => '"',
            CharEscape::ReverseSolidus => '\\',
            CharEscape::Solidus => '/',
            CharEscape::Backspace => '\u{8}',
            CharEscape::FormFeed => '\u{c}',
            CharEscape::LineFeed => '\n',
            CharEscape::CarriageReturn => '\r',
            CharEscape::Tab => '\t',
            CharEscape::AsciiControl(byte) => char::from(byte),
        };
        writer.write_all(escape_json_scalar(character).as_bytes())
    }

    fn begin_array<W: ?Sized + Write>(&mut self, writer: &mut W) -> io::Result<()> {
        self.pretty.begin_array(writer)
    }
    fn end_array<W: ?Sized + Write>(&mut self, writer: &mut W) -> io::Result<()> {
        self.pretty.end_array(writer)
    }
    fn begin_array_value<W: ?Sized + Write>(
        &mut self,
        writer: &mut W,
        first: bool,
    ) -> io::Result<()> {
        self.pretty.begin_array_value(writer, first)
    }
    fn end_array_value<W: ?Sized + Write>(&mut self, writer: &mut W) -> io::Result<()> {
        self.pretty.end_array_value(writer)
    }
    fn begin_object<W: ?Sized + Write>(&mut self, writer: &mut W) -> io::Result<()> {
        self.pretty.begin_object(writer)
    }
    fn end_object<W: ?Sized + Write>(&mut self, writer: &mut W) -> io::Result<()> {
        self.pretty.end_object(writer)
    }
    fn begin_object_key<W: ?Sized + Write>(
        &mut self,
        writer: &mut W,
        first: bool,
    ) -> io::Result<()> {
        self.pretty.begin_object_key(writer, first)
    }
    fn begin_object_value<W: ?Sized + Write>(&mut self, writer: &mut W) -> io::Result<()> {
        self.pretty.begin_object_value(writer)
    }
    fn end_object_value<W: ?Sized + Write>(&mut self, writer: &mut W) -> io::Result<()> {
        self.pretty.end_object_value(writer)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;
    use crate::unicode::representation::TextFormat;

    fn analyze(target: LaunchTarget) -> Value {
        let mut bytes = Vec::new();
        write(&target, &mut bytes).unwrap();
        assert!(bytes.is_ascii());
        serde_json::from_slice(&bytes).unwrap()
    }

    #[test]
    fn returns_full_case_mappings_and_direct_decomposition_as_integers() {
        let report = analyze(LaunchTarget::Sequence("ßİǺ각".to_owned()));
        let points = &report["code_points"];
        assert_eq!(
            points[0]["properties"]["case_mapping"],
            json!({
                "uppercase": [83, 83], "lowercase": [223], "titlecase": [83, 115], "case_folding": [115, 115]
            })
        );
        assert_eq!(
            points[1]["properties"]["case_mapping"]["lowercase"],
            json!([105, 775])
        );
        assert_eq!(
            points[1]["properties"]["case_mapping"]["case_folding"],
            json!([105, 775])
        );
        assert_eq!(
            points[2]["properties"]["decomposition"],
            json!({"type": "canonical", "mapping": [197, 769]})
        );
        assert_eq!(
            points[3]["properties"]["decomposition"],
            json!({"type": "canonical", "mapping": [44032, 4520]})
        );
        assert_eq!(
            report["normalization"]["nfd"]["text"],
            "ßI\u{307}A\u{30a}\u{301}각"
        );
    }

    #[test]
    fn represents_empty_analysis_without_nulls() {
        let report = analyze(LaunchTarget::Sequence(String::new()));
        let metrics =
            json!({ "code_points": 0, "utf8_bytes": 0, "utf16_units": 0, "graphemes": 0 });
        assert_eq!(report["text"], "");
        assert_eq!(report["metrics"], metrics);
        assert_eq!(report["code_points"], json!([]));
        assert_eq!(report["graphemes"], json!([]));
        for form in ["nfc", "nfd", "nfkc", "nfkd"] {
            assert_eq!(
                report["normalization"][form],
                json!({ "changed": false, "text": "", "metrics": metrics })
            );
        }
    }

    #[test]
    fn serializes_all_strings_with_the_existing_literal_escape_rules() {
        let text = "\0\n\r\t\u{8}\u{c}\u{1b}\u{7f}\"\\/\u{a0}\u{202e}\u{feff}😀";
        let value = json!({text: [text, {text: text}]});
        let mut bytes = Vec::new();
        value
            .serialize(&mut Serializer::with_formatter(
                &mut bytes,
                AsciiFormatter::default(),
            ))
            .unwrap();
        let output = String::from_utf8(bytes).unwrap();
        assert!(output.is_ascii());
        assert_eq!(serde_json::from_str::<Value>(&output).unwrap(), value);
        assert_eq!(
            output.matches(&TextFormat::JsonString.format(text)).count(),
            4
        );
    }
}
