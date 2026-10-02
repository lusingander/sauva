use std::ops::Range;

use unicode_normalization::UnicodeNormalization;

use super::text::{NormalizationForm, TextAnalysis};

const MAX_DIFF_CELLS: usize = 262_144;

/// Paired, half-open grapheme ranges, not a trace of normalization operations.
/// A region can contain different numbers of source and result graphemes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeRegion {
    pub original: Range<usize>,
    pub result: Range<usize>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NormalizationDiff {
    regions: Vec<ChangeRegion>,
    grouped: bool,
}

impl NormalizationDiff {
    pub fn new(original: &TextAnalysis, result: &TextAnalysis, form: NormalizationForm) -> Self {
        Self::with_budget(original, result, form, MAX_DIFF_CELLS)
    }

    pub fn regions(&self) -> &[ChangeRegion] {
        &self.regions
    }

    /// Very large contextual replacements are grouped rather than allocating
    /// an unbounded quadratic alignment table. The UI must label this case.
    pub const fn is_grouped(&self) -> bool {
        self.grouped
    }

    fn with_budget(
        original: &TextAnalysis,
        result: &TextAnalysis,
        form: NormalizationForm,
        budget: usize,
    ) -> Self {
        if original.source() == result.source() {
            return Self::default();
        }
        // Most normalization changes can be aligned in linear time. This fast
        // path is accepted ONLY when every independently normalized piece
        // matches the whole-string result and ends on a result boundary.
        if let Some(regions) = piece_alignment(original, result, form) {
            return Self {
                regions,
                grouped: false,
            };
        }
        let before = graphemes(original);
        let after = graphemes(result);
        let prefix = before
            .iter()
            .zip(&after)
            .take_while(|(a, b)| a == b)
            .count();
        let suffix = before[prefix..]
            .iter()
            .rev()
            .zip(after[prefix..].iter().rev())
            .take_while(|(a, b)| a == b)
            .count();
        let a = &before[prefix..before.len() - suffix];
        let b = &after[prefix..after.len() - suffix];
        let columns = b.len() + 1;
        let Some(cells) = (a.len() + 1)
            .checked_mul(columns)
            .filter(|cells| *cells <= budget)
        else {
            return Self {
                regions: vec![ChangeRegion {
                    original: prefix..before.len() - suffix,
                    result: prefix..after.len() - suffix,
                }],
                grouped: true,
            };
        };
        let mut lengths = vec![0usize; cells];
        for i in (0..a.len()).rev() {
            for j in (0..b.len()).rev() {
                lengths[i * columns + j] = if a[i] == b[j] {
                    lengths[(i + 1) * columns + j + 1] + 1
                } else {
                    lengths[(i + 1) * columns + j].max(lengths[i * columns + j + 1])
                };
            }
        }
        let mut regions = Vec::new();
        let (mut i, mut j) = (0, 0);
        while i < a.len() || j < b.len() {
            if i < a.len() && j < b.len() && a[i] == b[j] {
                i += 1;
                j += 1;
                continue;
            }
            let (start_i, start_j) = (i, j);
            while i < a.len() || j < b.len() {
                if i < a.len() && j < b.len() && a[i] == b[j] {
                    break;
                }
                if i < a.len()
                    && (j == b.len()
                        || lengths[(i + 1) * columns + j] >= lengths[i * columns + j + 1])
                {
                    i += 1;
                } else {
                    j += 1;
                }
            }
            regions.push(ChangeRegion {
                original: prefix + start_i..prefix + i,
                result: prefix + start_j..prefix + j,
            });
        }
        Self {
            regions,
            grouped: false,
        }
    }
}

fn graphemes(analysis: &TextAnalysis) -> Vec<&str> {
    analysis
        .graphemes()
        .iter()
        .map(|cluster| &analysis.source()[cluster.byte_range()])
        .collect()
}

fn piece_alignment(
    original: &TextAnalysis,
    result: &TextAnalysis,
    form: NormalizationForm,
) -> Option<Vec<ChangeRegion>> {
    let mut regions: Vec<ChangeRegion> = Vec::new();
    let mut piece = String::new();
    let (mut byte, mut result_index) = (0, 0);
    for (index, cluster) in original.graphemes().iter().enumerate() {
        let source = &original.source()[cluster.byte_range()];
        piece.clear();
        match form {
            NormalizationForm::Nfc => piece.extend(source.nfc()),
            NormalizationForm::Nfd => piece.extend(source.nfd()),
            NormalizationForm::Nfkc => piece.extend(source.nfkc()),
            NormalizationForm::Nfkd => piece.extend(source.nfkd()),
        }
        let end = byte + piece.len();
        if result.source().get(byte..end) != Some(piece.as_str()) {
            return None;
        }
        let start_index = result_index;
        while result_index < result.graphemes().len()
            && result.graphemes()[result_index].byte_range().end <= end
        {
            result_index += 1;
        }
        let boundary = if result_index == 0 {
            0
        } else {
            result.graphemes()[result_index - 1].byte_range().end
        };
        if boundary != end {
            return None;
        }
        if source != piece {
            if let Some(previous) = regions
                .last_mut()
                .filter(|region| region.original.end == index && region.result.end == start_index)
            {
                previous.original.end = index + 1;
                previous.result.end = result_index;
            } else {
                regions.push(ChangeRegion {
                    original: index..index + 1,
                    result: start_index..result_index,
                });
            }
        }
        byte = end;
    }
    (byte == result.source().len()).then_some(regions)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn diff(source: &str, form: NormalizationForm) -> NormalizationDiff {
        let original = TextAnalysis::new(source.to_owned());
        let result = TextAnalysis::new(original.normalized_text(form).to_owned());
        NormalizationDiff::new(&original, &result, form)
    }

    #[test]
    fn marks_whole_clusters_and_preserves_unchanged_context() {
        let changes = diff("A\u{0301} ①👩‍💻", NormalizationForm::Nfkc);
        assert_eq!(
            changes.regions(),
            [
                ChangeRegion {
                    original: 0..1,
                    result: 0..1
                },
                ChangeRegion {
                    original: 2..3,
                    result: 2..3
                }
            ]
        );
        assert!(!changes.is_grouped());
    }

    #[test]
    fn supports_composition_decomposition_and_reordering() {
        for (source, form) in [
            ("A\u{0301}", NormalizationForm::Nfc),
            ("é", NormalizationForm::Nfd),
            ("q\u{0307}\u{0323}", NormalizationForm::Nfd),
        ] {
            assert_eq!(
                diff(source, form).regions(),
                [ChangeRegion {
                    original: 0..1,
                    result: 0..1
                }]
            );
        }
    }

    #[test]
    fn pairs_one_cluster_with_multiple_result_clusters() {
        assert_eq!(
            diff("oﬃce", NormalizationForm::Nfkc).regions(),
            [ChangeRegion {
                original: 1..2,
                result: 1..4
            }]
        );
    }

    #[test]
    fn normalizes_the_whole_string_before_aligning() {
        // Compatibility jamo compose across ORIGINAL grapheme boundaries.
        assert_eq!(
            diff("ᄀㅏ", NormalizationForm::Nfkc).regions(),
            [ChangeRegion {
                original: 0..2,
                result: 0..1
            }]
        );
    }

    #[test]
    fn leaves_unchanged_and_empty_text_unmarked() {
        for form in NormalizationForm::ALL {
            assert!(diff("Rust → Unicode", form).regions().is_empty());
            assert!(diff("", form).regions().is_empty());
        }
    }

    #[test]
    fn aligns_large_regular_inputs_without_quadratic_work() {
        let source = "① ".repeat(10_000);
        let changes = diff(&source, NormalizationForm::Nfkc);
        assert_eq!(changes.regions().len(), 10_000);
        assert!(!changes.is_grouped());
    }

    #[test]
    fn bounds_contextual_alignment_and_labels_grouped_ranges() {
        let original = TextAnalysis::new("prefix ᄀㅏxᄀㅏ suffix".to_owned());
        let result =
            TextAnalysis::new(original.normalized_text(NormalizationForm::Nfkc).to_owned());
        let changes =
            NormalizationDiff::with_budget(&original, &result, NormalizationForm::Nfkc, 1);
        assert!(changes.is_grouped());
        assert_eq!(
            changes.regions(),
            [ChangeRegion {
                original: 7..12,
                result: 7..10
            }]
        );
    }
}
