use std::ops::Range;

use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

use crate::unicode::CodePoint;

/// Unicode analysis of the original UTF-8 text, independent of UI state.
///
/// All byte ranges are half-open offsets into `source`. Code point and grapheme
/// indices are zero-based. Graphemes use the extended cluster rules from UAX #29.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextAnalysis {
    source: Box<str>,
    code_points: Vec<AnalyzedCodePoint>,
    graphemes: Vec<GraphemeCluster>,
    normalization: NormalizationAnalysis,
}

impl TextAnalysis {
    /// Empty and single-code-point text are valid analysis inputs.
    pub fn new(source: String) -> Self {
        let source = source.into_boxed_str();
        let mut code_points = Vec::new();
        let mut graphemes = Vec::new();

        for (grapheme_index, (byte_start, text)) in source.grapheme_indices(true).enumerate() {
            let code_point_start = code_points.len();
            for (index_in_grapheme, (offset, character)) in text.char_indices().enumerate() {
                let start = byte_start + offset;
                code_points.push(AnalyzedCodePoint {
                    code_point: CodePoint::from(character),
                    byte_range: start..start + character.len_utf8(),
                    grapheme_index,
                    index_in_grapheme,
                });
            }
            graphemes.push(GraphemeCluster {
                byte_range: byte_start..byte_start + text.len(),
                code_point_range: code_point_start..code_points.len(),
            });
        }

        let normalization = NormalizationAnalysis::new(&source);
        Self {
            source,
            code_points,
            graphemes,
            normalization,
        }
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn byte_count(&self) -> usize {
        self.source.len()
    }

    pub fn code_points(&self) -> &[AnalyzedCodePoint] {
        &self.code_points
    }

    pub fn graphemes(&self) -> &[GraphemeCluster] {
        &self.graphemes
    }

    pub const fn normalization(&self) -> &NormalizationAnalysis {
        &self.normalization
    }

    /// Returns the exact result, borrowing the source when normalization leaves
    /// it unchanged. Display escaping belongs to the UI, not the analysis model.
    pub fn normalized_text(&self, form: NormalizationForm) -> &str {
        self.normalization
            .get(form)
            .changed_text()
            .unwrap_or(&self.source)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalyzedCodePoint {
    code_point: CodePoint,
    byte_range: Range<usize>,
    grapheme_index: usize,
    index_in_grapheme: usize,
}

impl AnalyzedCodePoint {
    pub const fn code_point(&self) -> CodePoint {
        self.code_point
    }

    pub fn byte_range(&self) -> Range<usize> {
        self.byte_range.clone()
    }

    pub const fn grapheme_index(&self) -> usize {
        self.grapheme_index
    }

    pub const fn index_in_grapheme(&self) -> usize {
        self.index_in_grapheme
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphemeCluster {
    byte_range: Range<usize>,
    code_point_range: Range<usize>,
}

impl GraphemeCluster {
    pub fn byte_range(&self) -> Range<usize> {
        self.byte_range.clone()
    }

    pub fn code_point_range(&self) -> Range<usize> {
        self.code_point_range.clone()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NormalizationForm {
    Nfc,
    Nfd,
    Nfkc,
    Nfkd,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizationAnalysis {
    nfc: NormalizationResult,
    nfd: NormalizationResult,
    nfkc: NormalizationResult,
    nfkd: NormalizationResult,
}

impl NormalizationAnalysis {
    fn new(source: &str) -> Self {
        Self {
            nfc: NormalizationResult::new(source, source.nfc()),
            nfd: NormalizationResult::new(source, source.nfd()),
            nfkc: NormalizationResult::new(source, source.nfkc()),
            nfkd: NormalizationResult::new(source, source.nfkd()),
        }
    }

    pub const fn get(&self, form: NormalizationForm) -> &NormalizationResult {
        match form {
            NormalizationForm::Nfc => &self.nfc,
            NormalizationForm::Nfd => &self.nfd,
            NormalizationForm::Nfkc => &self.nfkc,
            NormalizationForm::Nfkd => &self.nfkd,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizationResult {
    /// `None` means byte-for-byte equality with the source, avoiding retention
    /// of a duplicate string for unchanged results.
    output: Option<Box<str>>,
    code_point_count: usize,
}

impl NormalizationResult {
    fn new(source: &str, characters: impl Iterator<Item = char>) -> Self {
        let output = characters.collect::<String>();
        let code_point_count = output.chars().count();
        Self {
            output: (output != source).then(|| output.into_boxed_str()),
            code_point_count,
        }
    }

    pub const fn is_changed(&self) -> bool {
        self.output.is_some()
    }

    pub fn changed_text(&self) -> Option<&str> {
        self.output.as_deref()
    }

    pub const fn code_point_count(&self) -> usize {
        self.code_point_count
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::unicode::UnicodeDatabase;

    const FORMS: [NormalizationForm; 4] = [
        NormalizationForm::Nfc,
        NormalizationForm::Nfd,
        NormalizationForm::Nfkc,
        NormalizationForm::Nfkd,
    ];

    #[test]
    fn algorithm_versions_match_the_bundled_ucd() {
        let normalization = unicode_normalization::UNICODE_VERSION;
        for (major, minor, patch) in [
            unicode_segmentation::UNICODE_VERSION,
            (
                u64::from(normalization.0),
                u64::from(normalization.1),
                u64::from(normalization.2),
            ),
        ] {
            assert_eq!(
                format!("{major}.{minor}.{patch}"),
                UnicodeDatabase::version()
            );
        }
    }

    #[test]
    fn tracks_byte_ranges_and_membership_for_combining_text_and_zwj_emoji() {
        let source = "A\u{0301} 👩‍💻";
        let analysis = TextAnalysis::new(source.to_owned());

        assert_eq!(analysis.source(), source);
        assert_eq!(analysis.byte_count(), 15);
        assert_eq!(analysis.code_points().len(), 6);
        assert_eq!(analysis.graphemes().len(), 3);
        let expected = [
            (0x0041, 0..1, 0, 0),
            (0x0301, 1..3, 0, 1),
            (0x0020, 3..4, 1, 0),
            (0x1f469, 4..8, 2, 0),
            (0x200d, 8..11, 2, 1),
            (0x1f4bb, 11..15, 2, 2),
        ];
        for (point, (value, bytes, grapheme, member)) in analysis.code_points().iter().zip(expected)
        {
            assert_eq!(point.code_point().value(), value);
            assert_eq!(point.byte_range(), bytes);
            assert_eq!(point.grapheme_index(), grapheme);
            assert_eq!(point.index_in_grapheme(), member);
            assert_eq!(
                &source[point.byte_range()],
                point.code_point().to_char().unwrap().to_string()
            );
        }
        for (cluster, (bytes, points)) in
            analysis
                .graphemes()
                .iter()
                .zip([(0..3, 0..2), (3..4, 2..3), (4..15, 3..6)])
        {
            assert_eq!(cluster.byte_range(), bytes);
            assert_eq!(cluster.code_point_range(), points);
        }

        let nfc = analysis.normalization().get(NormalizationForm::Nfc);
        assert!(nfc.is_changed());
        assert_eq!(nfc.code_point_count(), 5);
        assert_eq!(analysis.normalized_text(NormalizationForm::Nfc), "Á 👩‍💻");
        let nfd = analysis.normalization().get(NormalizationForm::Nfd);
        assert!(!nfd.is_changed());
        assert_eq!(nfd.changed_text(), None);
        assert_eq!(nfd.code_point_count(), 6);
        assert_eq!(analysis.normalized_text(NormalizationForm::Nfd), source);
    }

    #[rstest]
    #[case("", &[])]
    #[case("é", &[(0..2, 0..1)])]
    #[case("🇯🇵", &[(0..8, 0..2)])]
    #[case("🇯🇵🇺", &[(0..8, 0..2), (8..12, 2..3)])]
    #[case("\r\n", &[(0..2, 0..2)])]
    #[case("\u{0301}", &[(0..2, 0..1)])]
    #[case("\u{0301}\u{0300}A", &[(0..4, 0..2), (4..5, 2..3)])]
    #[case("❤️", &[(0..6, 0..2)])]
    #[case("👍🏽", &[(0..8, 0..2)])]
    #[case("\u{1100}\u{1161}\u{11a8}", &[(0..9, 0..3)])]
    #[case("\u{0915}\u{094d}\u{0937}", &[(0..9, 0..3)])]
    #[case("あAあ", &[(0..3, 0..1), (3..4, 1..2), (4..7, 2..3)])]
    fn segments_extended_graphemes_and_keeps_contiguous_ranges(
        #[case] source: &str,
        #[case] expected: &[(Range<usize>, Range<usize>)],
    ) {
        let analysis = TextAnalysis::new(source.to_owned());

        assert_eq!(analysis.byte_count(), source.len());
        assert_eq!(analysis.code_points().len(), source.chars().count());
        assert_eq!(
            analysis
                .graphemes()
                .iter()
                .map(|cluster| (cluster.byte_range(), cluster.code_point_range()))
                .collect::<Vec<_>>(),
            expected
        );
        let mut byte_end = 0;
        for (cluster_index, cluster) in analysis.graphemes().iter().enumerate() {
            let cluster_text = &source[cluster.byte_range()];
            let members = &analysis.code_points()[cluster.code_point_range()];
            assert_eq!(cluster.byte_range().start, byte_end);
            assert_eq!(cluster_text.chars().count(), members.len());
            for (member_index, point) in members.iter().enumerate() {
                assert_eq!(point.grapheme_index(), cluster_index);
                assert_eq!(point.index_in_grapheme(), member_index);
                assert_eq!(point.byte_range().start, byte_end);
                byte_end = point.byte_range().end;
            }
            assert_eq!(cluster.byte_range().end, byte_end);
        }
        assert_eq!(byte_end, source.len());
    }

    #[rstest]
    #[case("", ["", "", "", ""])]
    #[case("ASCII", ["ASCII", "ASCII", "ASCII", "ASCII"])]
    #[case("é", ["é", "e\u{0301}", "é", "e\u{0301}"])]
    #[case("e\u{0301}", ["é", "e\u{0301}", "é", "e\u{0301}"])]
    #[case("①", ["①", "①", "1", "1"])]
    #[case("ﬃ", ["ﬃ", "ﬃ", "ffi", "ffi"])]
    #[case("각", ["각", "\u{1100}\u{1161}\u{11a8}", "각", "\u{1100}\u{1161}\u{11a8}"])]
    #[case("q\u{0301}\u{0323}", ["q\u{0323}\u{0301}"; 4])]
    #[case("\0\r\né\u{200d}", ["\0\r\né\u{200d}", "\0\r\ne\u{0301}\u{200d}", "\0\r\né\u{200d}", "\0\r\ne\u{0301}\u{200d}"])]
    fn retains_exact_normalized_outputs_without_changing_the_source(
        #[case] source: &str,
        #[case] expected: [&str; 4],
    ) {
        let analysis = TextAnalysis::new(source.to_owned());

        for (form, text) in FORMS.into_iter().zip(expected) {
            let result = analysis.normalization().get(form);
            assert_eq!(analysis.normalized_text(form), text);
            assert_eq!(result.code_point_count(), text.chars().count());
            assert_eq!(result.is_changed(), text != source);
            assert_eq!(result.changed_text(), (text != source).then_some(text));
            let normalized = TextAnalysis::new(text.to_owned());
            assert_eq!(normalized.normalized_text(form), text);
        }
        assert_eq!(analysis.source(), source);
    }
}
