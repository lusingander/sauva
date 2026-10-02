use unicode_segmentation::UnicodeSegmentation;

use crate::{
    sequence::SequenceState,
    unicode::{
        diff::NormalizationDiff,
        text::{NormalizationForm, TextAnalysis},
    },
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
            NormalizationComparison { result, diff }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
