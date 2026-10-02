use std::ops::Range;

use crate::{
    unicode::{
        CodePoint,
        text::{AnalyzedCodePoint, TextAnalysis},
    },
    viewport::ListViewport,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SequenceMove {
    Previous,
    Next,
    First,
    Last,
}

#[derive(Debug, Clone)]
pub struct SequenceState {
    analysis: TextAnalysis,
    selected_index: usize,
    viewport: ListViewport,
}

impl SequenceState {
    pub fn new(source: String) -> Self {
        let analysis = TextAnalysis::new(source);
        assert!(
            analysis.code_points().len() > 1,
            "a sequence contains at least two code points"
        );
        Self {
            analysis,
            selected_index: 0,
            viewport: ListViewport::new(),
        }
    }

    pub const fn analysis(&self) -> &TextAnalysis {
        &self.analysis
    }

    pub fn code_points(&self) -> &[AnalyzedCodePoint] {
        self.analysis().code_points()
    }

    pub const fn selected_index(&self) -> usize {
        self.selected_index
    }

    pub fn selected(&self) -> CodePoint {
        self.code_points()[self.selected_index].code_point()
    }

    pub fn visible_range(&self) -> Range<usize> {
        self.viewport.visible_range(self.code_points().len())
    }

    pub fn move_selection(&mut self, movement: SequenceMove) -> bool {
        let last = self.code_points().len() - 1;
        let next = match movement {
            SequenceMove::Previous => self.selected_index.saturating_sub(1),
            SequenceMove::Next => (self.selected_index + 1).min(last),
            SequenceMove::First => 0,
            SequenceMove::Last => last,
        };
        if next == self.selected_index {
            return false;
        }

        self.selected_index = next;
        self.viewport.ensure_visible(next, self.code_points().len());
        true
    }

    pub fn resize_viewport(&mut self, height: usize) {
        self.viewport
            .resize(height, self.selected_index, self.code_points().len());
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::unicode::text::NormalizationForm;

    fn state() -> SequenceState {
        SequenceState::new("A→A".to_owned())
    }

    #[test]
    fn preserves_input_order_and_duplicates() {
        let state = state();

        assert_eq!(
            state
                .code_points()
                .iter()
                .map(|code_point| code_point.code_point().value())
                .collect::<Vec<_>>(),
            [0x0041, 0x2192, 0x0041]
        );
        assert_eq!(state.selected_index(), 0);
        assert_eq!(state.selected().value(), 0x0041);
        assert_eq!(state.analysis().source(), "A→A");
    }

    #[rstest]
    #[case("")]
    #[case("A")]
    #[case("é")]
    #[case("👩")]
    #[should_panic(expected = "a sequence contains at least two code points")]
    fn rejects_inputs_without_multiple_code_points(#[case] source: &str) {
        SequenceState::new(source.to_owned());
    }

    #[test]
    fn selects_each_code_point_without_changing_the_analysis() {
        let source = "A\u{0301} 👩‍💻";
        let mut state = SequenceState::new(source.to_owned());
        let analysis = state.analysis().clone();
        state.resize_viewport(2);

        assert!(state.move_selection(SequenceMove::Next));
        assert_eq!(state.selected().value(), 0x0301);
        assert_eq!(
            state.code_points()[state.selected_index()].grapheme_index(),
            0
        );
        assert!(state.move_selection(SequenceMove::Last));
        assert_eq!(state.selected().value(), 0x1f4bb);
        assert_eq!(state.visible_range(), 4..6);
        assert!(state.move_selection(SequenceMove::Previous));
        assert_eq!(state.selected().value(), 0x200d);
        assert_eq!(
            state.code_points()[state.selected_index()].grapheme_index(),
            2
        );
        state.resize_viewport(1);
        assert_eq!(state.visible_range(), 4..5);
        assert_eq!(state.analysis(), &analysis);
        assert_eq!(state.analysis().source(), source);
        assert_eq!(
            state.analysis().normalized_text(NormalizationForm::Nfc),
            "Á 👩‍💻"
        );
    }

    #[test]
    fn moves_within_the_sequence_and_keeps_the_selection_visible() {
        let mut state = state();
        state.resize_viewport(2);

        assert!(state.move_selection(SequenceMove::Last));
        assert_eq!(state.selected_index(), 2);
        assert_eq!(state.visible_range(), 1..3);
        assert!(state.move_selection(SequenceMove::Previous));
        assert_eq!(state.selected_index(), 1);
        assert_eq!(state.visible_range(), 1..3);
        assert!(state.move_selection(SequenceMove::First));
        assert_eq!(state.selected_index(), 0);
        assert_eq!(state.visible_range(), 0..2);
    }

    #[test]
    fn stops_at_the_sequence_boundaries() {
        let mut state = state();

        assert!(!state.move_selection(SequenceMove::Previous));
        assert!(state.move_selection(SequenceMove::Next));
        assert!(state.move_selection(SequenceMove::Next));
        assert!(!state.move_selection(SequenceMove::Next));
    }
}
