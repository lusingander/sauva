use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;

use crate::{
    sequence::SequenceState,
    unicode::{
        diff::NormalizationDiff,
        text::{NormalizationForm, TextAnalysis},
    },
    viewport::ListViewport,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NormalizationMove {
    Previous,
    Next,
    First,
    Last,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextMetrics {
    pub code_points: usize,
    pub bytes: usize,
    pub graphemes: usize,
}

impl TextMetrics {
    pub fn original(analysis: &TextAnalysis) -> Self {
        Self {
            code_points: analysis.code_points().len(),
            bytes: analysis.byte_count(),
            graphemes: analysis.graphemes().len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NormalizationComparison {
    pub result: SequenceState,
    pub diff: NormalizationDiff,
    original_viewport: ListViewport,
}

impl NormalizationComparison {
    pub fn original_selection(&self, original: &TextAnalysis) -> Range<usize> {
        let index = self.result.code_points()[self.result.selected_index()].grapheme_index();
        let Some(range) = self
            .diff
            .original_graphemes(index)
            .filter(|range| !range.is_empty())
        else {
            return 0..0;
        };
        original.graphemes()[range.start].code_point_range().start
            ..original.graphemes()[range.end - 1].code_point_range().end
    }

    pub fn original_visible_range(&self, original: &TextAnalysis) -> Range<usize> {
        self.original_viewport
            .visible_range(original.code_points().len())
    }

    pub fn follow_selection(&mut self, original: &TextAnalysis) {
        let selected = self.original_selection(original);
        if !selected.is_empty() {
            self.original_viewport
                .ensure_visible(selected.end - 1, original.code_points().len());
            self.original_viewport
                .ensure_visible(selected.start, original.code_points().len());
        }
    }

    pub fn resize_original_viewport(&mut self, original: &TextAnalysis, height: usize) {
        let selected = self.original_selection(original);
        self.original_viewport
            .resize(height, selected.start, original.code_points().len());
        self.follow_selection(original);
    }
}

#[derive(Debug, Clone)]
pub struct NormalizationState {
    selected: usize,
    metrics: [TextMetrics; 4],
    comparisons: [Option<NormalizationComparison>; 4],
}

impl NormalizationState {
    pub fn new(original: &TextAnalysis) -> Self {
        let metrics = NormalizationForm::ALL.map(|form| {
            let text = original.normalized_text(form);
            TextMetrics {
                code_points: original.normalization().get(form).code_point_count(),
                bytes: text.len(),
                graphemes: text.graphemes(true).count(),
            }
        });
        let mut state = Self {
            selected: 0,
            metrics,
            comparisons: std::array::from_fn(|_| None),
        };
        state.prepare_result(original);
        state
    }

    pub fn form(&self) -> NormalizationForm {
        NormalizationForm::ALL[self.selected]
    }

    pub fn metrics(&self, form: NormalizationForm) -> TextMetrics {
        self.metrics[form as usize]
    }

    pub fn comparison(&self) -> &NormalizationComparison {
        self.comparisons[self.selected]
            .as_ref()
            .expect("the selected comparison is prepared")
    }

    pub fn comparison_mut(&mut self) -> &mut NormalizationComparison {
        self.comparisons[self.selected]
            .as_mut()
            .expect("the selected comparison is prepared")
    }

    pub fn move_selection(&mut self, movement: NormalizationMove, original: &TextAnalysis) {
        self.selected = match movement {
            NormalizationMove::Previous => self.selected.saturating_sub(1),
            NormalizationMove::Next => (self.selected + 1).min(3),
            NormalizationMove::First => 0,
            NormalizationMove::Last => 3,
        };
        self.prepare_result(original);
    }

    fn prepare_result(&mut self, original: &TextAnalysis) {
        let form = self.form();
        self.comparisons[self.selected].get_or_insert_with(|| {
            let result = SequenceState::new(original.normalized_text(form).to_owned());
            let diff = NormalizationDiff::new(original, result.analysis(), form);
            NormalizationComparison {
                result,
                diff,
                original_viewport: ListViewport::new(),
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_tracks_expanded_scalars_and_adjacent_changes_separately() {
        let original = TextAnalysis::new("ﬃ①".to_owned());
        let mut state = NormalizationState::new(&original);
        state.move_selection(NormalizationMove::Next, &original);
        state.move_selection(NormalizationMove::Next, &original);
        let comparison = state.comparison_mut();
        for expected in [0..1, 0..1, 0..1, 1..2] {
            assert_eq!(comparison.original_selection(&original), expected);
            comparison
                .result
                .move_selection(crate::sequence::SequenceMove::Next);
        }
    }

    #[test]
    fn lazily_prepares_forms_and_preserves_the_original() {
        let original = TextAnalysis::new("A\u{0301} ①👩‍💻".to_owned());
        let saved = original.clone();
        let mut state = NormalizationState::new(&original);
        assert_eq!(state.form(), NormalizationForm::Nfc);
        assert_eq!(state.comparison().result.analysis().source(), "Á ①👩‍💻");
        assert_eq!(
            state
                .comparisons
                .iter()
                .filter(|value| value.is_some())
                .count(),
            1
        );
        state.move_selection(NormalizationMove::Last, &original);
        assert_eq!(state.form(), NormalizationForm::Nfkd);
        assert_eq!(
            state.metrics(NormalizationForm::Nfkc),
            TextMetrics {
                code_points: 6,
                bytes: 15,
                graphemes: 4
            }
        );
        assert_eq!(
            state
                .comparisons
                .iter()
                .filter(|value| value.is_some())
                .count(),
            2
        );
        assert_eq!(original, saved);
        state.move_selection(NormalizationMove::Next, &original);
        assert_eq!(state.form(), NormalizationForm::Nfkd);
        state.move_selection(NormalizationMove::First, &original);
        state.move_selection(NormalizationMove::Previous, &original);
        assert_eq!(state.form(), NormalizationForm::Nfc);
    }
}
