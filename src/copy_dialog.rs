use std::borrow::Cow;

use crate::unicode::{
    CodePoint,
    representation::TextFormat,
    text::{NormalizationForm, TextAnalysis},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopyTarget {
    CodePoint,
    Grapheme,
    WholeText,
}

impl CopyTarget {
    pub const ALL: [Self; 3] = [Self::CodePoint, Self::Grapheme, Self::WholeText];

    pub const fn label(self) -> &'static str {
        match self {
            Self::CodePoint => "Code Point",
            Self::Grapheme => "Grapheme Cluster",
            Self::WholeText => "Whole Text",
        }
    }
}

/// The originating screen supplies exact data, independently of dialog state.
#[derive(Debug, Clone, Copy)]
pub enum CopySource<'a> {
    Inspector(CodePoint),
    Sequence {
        analysis: &'a TextAnalysis,
        selected_index: usize,
        form: Option<NormalizationForm>,
    },
    Normalization {
        analysis: &'a TextAnalysis,
        form: NormalizationForm,
    },
}

impl<'a> CopySource<'a> {
    pub fn targets(self) -> &'static [CopyTarget] {
        match self {
            Self::Inspector(_) => &[CopyTarget::CodePoint],
            Self::Sequence { .. } => &CopyTarget::ALL,
            Self::Normalization { .. } => &[CopyTarget::WholeText],
        }
    }

    pub fn formats(self) -> &'static [TextFormat] {
        match self {
            Self::Inspector(point) if point.to_char().is_none() => &[TextFormat::CodePointList],
            _ => &TextFormat::ALL,
        }
    }

    pub fn context_label(self) -> Option<String> {
        match self {
            Self::Inspector(point) => Some(point.to_string()),
            Self::Sequence {
                form: Some(form), ..
            } => Some(form.label().to_owned()),
            Self::Sequence { form: None, .. } | Self::Normalization { .. } => None,
        }
    }

    pub fn origin_label(self) -> String {
        match self {
            Self::Inspector(point) => point.to_string(),
            Self::Sequence { form: None, .. } => "Input".to_owned(),
            Self::Sequence {
                form: Some(form), ..
            }
            | Self::Normalization { form, .. } => {
                format!("{} Result", form.label())
            }
        }
    }

    pub fn target_label(self, target: CopyTarget) -> String {
        if target == CopyTarget::WholeText {
            format!("Whole {}", self.origin_label())
        } else {
            target.label().to_owned()
        }
    }

    pub fn value(self, target: CopyTarget) -> CopyValue<'a> {
        match self {
            Self::Inspector(point) => CopyValue::CodePoint(point),
            Self::Normalization { analysis, .. } => CopyValue::Text {
                text: analysis.source(),
                code_point_count: analysis.code_points().len(),
            },
            Self::Sequence {
                analysis,
                selected_index,
                ..
            } => {
                let (range, count) = match target {
                    CopyTarget::WholeText => {
                        (0..analysis.byte_count(), analysis.code_points().len())
                    }
                    CopyTarget::CodePoint => {
                        (analysis.code_points()[selected_index].byte_range(), 1)
                    }
                    CopyTarget::Grapheme => {
                        let point = &analysis.code_points()[selected_index];
                        let cluster = &analysis.graphemes()[point.grapheme_index()];
                        (cluster.byte_range(), cluster.code_point_range().len())
                    }
                };
                CopyValue::Text {
                    text: &analysis.source()[range],
                    code_point_count: count,
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CopyValue<'a> {
    CodePoint(CodePoint),
    Text {
        text: &'a str,
        code_point_count: usize,
    },
}

impl<'a> CopyValue<'a> {
    pub fn text(self) -> Option<Cow<'a, str>> {
        match self {
            Self::CodePoint(point) => point
                .to_char()
                .map(|character| Cow::Owned(character.to_string())),
            Self::Text { text, .. } => Some(Cow::Borrowed(text)),
        }
    }

    pub fn code_point_count(self) -> usize {
        match self {
            Self::CodePoint(_) => 1,
            Self::Text {
                code_point_count, ..
            } => code_point_count,
        }
    }

    pub fn format(self, format: TextFormat) -> Option<String> {
        if let Self::CodePoint(point) = self
            && format == TextFormat::CodePointList
        {
            return Some(point.to_string());
        }
        self.text().map(|text| format.format(&text))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopyMove {
    Previous,
    Next,
    First,
    Last,
    PreviousFormat,
    NextFormat,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CopyDialogState {
    selected_index: usize,
    format_index: usize,
}

impl CopyDialogState {
    pub const fn selected_index(&self) -> usize {
        self.selected_index
    }

    pub fn selected_target(&self, source: CopySource<'_>) -> CopyTarget {
        source.targets()[self.selected_index]
    }

    pub fn selected_format(&self, source: CopySource<'_>) -> TextFormat {
        source.formats()[self.format_index]
    }

    pub fn move_selection(&mut self, movement: CopyMove, source: CopySource<'_>) {
        let last = source.targets().len() - 1;
        let formats = source.formats().len();
        match movement {
            CopyMove::Previous => self.selected_index = self.selected_index.saturating_sub(1),
            CopyMove::Next => self.selected_index = (self.selected_index + 1).min(last),
            CopyMove::First => self.selected_index = 0,
            CopyMove::Last => self.selected_index = last,
            CopyMove::PreviousFormat => {
                self.format_index = (self.format_index + formats - 1) % formats
            }
            CopyMove::NextFormat => self.format_index = (self.format_index + 1) % formats,
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::sequence::{SequenceMove, SequenceState};

    fn source(sequence: &SequenceState) -> CopySource<'_> {
        CopySource::Sequence {
            analysis: sequence.analysis(),
            selected_index: sequence.selected_index(),
            form: None,
        }
    }

    #[rstest]
    #[case("A\u{0301}")]
    #[case("\u{0301}\u{0300}")]
    #[case("👩🏽‍💻")]
    #[case("🇯🇵")]
    #[case("❤️")]
    #[case("\r\n")]
    fn copies_the_whole_cluster_from_each_member_even_outside_the_viewport(#[case] cluster: &str) {
        let text = format!("\n{cluster}Y");
        let mut sequence = SequenceState::new(text.clone());
        sequence.resize_viewport(1);
        for member in cluster.chars() {
            sequence.move_selection(SequenceMove::Next);
            assert_eq!(
                source(&sequence)
                    .value(CopyTarget::CodePoint)
                    .text()
                    .unwrap(),
                member.to_string()
            );
            assert_eq!(
                source(&sequence)
                    .value(CopyTarget::Grapheme)
                    .text()
                    .unwrap(),
                cluster
            );
            assert_eq!(
                source(&sequence)
                    .value(CopyTarget::WholeText)
                    .text()
                    .unwrap(),
                text
            );
        }
    }

    #[test]
    fn distinguishes_repeated_clusters_and_preserves_all_controls() {
        let text = "A\u{0301}\t\r\n\u{1b}\u{202e}A\u{0301}";
        let mut sequence = SequenceState::new(text.to_owned());
        sequence.move_selection(SequenceMove::Last);
        assert_eq!(
            source(&sequence)
                .value(CopyTarget::CodePoint)
                .text()
                .unwrap(),
            "\u{0301}"
        );
        assert_eq!(
            source(&sequence)
                .value(CopyTarget::Grapheme)
                .text()
                .unwrap(),
            "A\u{0301}"
        );
        assert_eq!(
            source(&sequence)
                .value(CopyTarget::WholeText)
                .text()
                .unwrap(),
            text
        );
    }

    #[test]
    fn scopes_and_formats_move_independently_and_only_formats_wrap() {
        let sequence = SequenceState::new("A\u{301}😀".to_owned());
        let source = source(&sequence);
        let mut dialog = CopyDialogState::default();
        dialog.move_selection(CopyMove::Previous, source);
        dialog.move_selection(CopyMove::PreviousFormat, source);
        assert_eq!(dialog.selected_target(source), CopyTarget::CodePoint);
        assert_eq!(dialog.selected_format(source), TextFormat::Utf16Units);
        dialog.move_selection(CopyMove::Last, source);
        dialog.move_selection(CopyMove::Next, source);
        assert_eq!(dialog.selected_target(source), CopyTarget::WholeText);
        dialog.move_selection(CopyMove::NextFormat, source);
        assert_eq!(dialog.selected_format(source), TextFormat::Text);
        dialog.move_selection(CopyMove::First, source);
        assert_eq!(dialog.selected_target(source), CopyTarget::CodePoint);
    }

    #[test]
    fn surrogates_have_one_target_and_one_available_format() {
        let source = CopySource::Inspector(CodePoint::new(0xd800).unwrap());
        let mut dialog = CopyDialogState::default();
        for movement in [
            CopyMove::Last,
            CopyMove::Next,
            CopyMove::PreviousFormat,
            CopyMove::NextFormat,
        ] {
            dialog.move_selection(movement, source);
        }
        assert_eq!(source.targets(), &[CopyTarget::CodePoint]);
        assert_eq!(source.formats(), &[TextFormat::CodePointList]);
        assert_eq!(dialog.selected_format(source), TextFormat::CodePointList);
        let value = source.value(dialog.selected_target(source));
        assert_eq!(
            value.format(dialog.selected_format(source)).as_deref(),
            Some("U+D800")
        );
        assert!(value.format(TextFormat::Text).is_none());
    }

    #[test]
    fn whole_text_does_not_require_a_selected_code_point_or_nonempty_input() {
        let analysis = TextAnalysis::new(String::new());
        let source = CopySource::Sequence {
            analysis: &analysis,
            selected_index: 0,
            form: None,
        };
        let value = source.value(CopyTarget::WholeText);
        assert_eq!(value.code_point_count(), 0);
        assert_eq!(value.format(TextFormat::Text).as_deref(), Some(""));
        assert_eq!(
            value.format(TextFormat::JsonString).as_deref(),
            Some(r#""""#)
        );
    }
}
