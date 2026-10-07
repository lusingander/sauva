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
    PreviousGroup,
    NextGroup,
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
            !analysis.code_points().is_empty(),
            "a sequence contains at least one code point"
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
        let grapheme_index = self.code_points()[self.selected_index].grapheme_index();
        let next = match movement {
            SequenceMove::Previous => self.selected_index.saturating_sub(1),
            SequenceMove::Next => (self.selected_index + 1).min(last),
            SequenceMove::PreviousGroup => grapheme_index
                .checked_sub(1)
                .and_then(|index| self.analysis.graphemes().get(index))
                .map_or(self.selected_index, |group| group.code_point_range().start),
            SequenceMove::NextGroup => self
                .analysis
                .graphemes()
                .get(grapheme_index + 1)
                .map_or(self.selected_index, |group| group.code_point_range().start),
            SequenceMove::First => 0,
            SequenceMove::Last => last,
        };
        if next == self.selected_index {
            return false;
        }

        self.selected_index = next;
        if matches!(
            movement,
            SequenceMove::PreviousGroup | SequenceMove::NextGroup
        ) {
            self.viewport.align_start(next, self.code_points().len());
        } else {
            self.viewport.ensure_visible(next, self.code_points().len());
        }
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

    #[test]
    #[should_panic(expected = "a sequence contains at least one code point")]
    fn rejects_empty_inputs() {
        SequenceState::new(String::new());
    }

    #[rstest]
    #[case("A")]
    #[case("Á")]
    #[case("👩")]
    fn supports_single_code_point_normalization_results(#[case] source: &str) {
        let mut state = SequenceState::new(source.to_owned());
        state.resize_viewport(1);
        for movement in [
            SequenceMove::Previous,
            SequenceMove::Next,
            SequenceMove::PreviousGroup,
            SequenceMove::NextGroup,
            SequenceMove::First,
            SequenceMove::Last,
        ] {
            assert!(!state.move_selection(movement));
        }
        assert_eq!(state.visible_range(), 0..1);
        assert_eq!(state.selected().to_char().unwrap().to_string(), source);
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

    #[test]
    fn group_jumps_follow_grapheme_boundaries_and_align_their_first_members() {
        let mut state = SequenceState::new("A\u{0301}\r\n👩🏽‍💻🇯🇵BB".to_owned());
        state.resize_viewport(2);
        state.move_selection(SequenceMove::Next);
        assert!(!state.move_selection(SequenceMove::PreviousGroup));
        assert_eq!(state.selected_index(), 1);
        for index in [2, 4, 8, 10, 11] {
            assert!(state.move_selection(SequenceMove::NextGroup));
            assert_eq!(state.selected_index(), index);
            assert_eq!(state.code_points()[index].index_in_grapheme(), 0);
            assert_eq!(state.visible_range().start, index.min(10));
        }
        assert!(!state.move_selection(SequenceMove::NextGroup));
        assert_eq!(state.selected_index(), 11);
        for index in [10, 8] {
            assert!(state.move_selection(SequenceMove::PreviousGroup));
            assert_eq!(state.selected_index(), index);
        }
        state.move_selection(SequenceMove::Next);
        assert_eq!(state.selected_index(), 9);
        assert!(state.move_selection(SequenceMove::PreviousGroup));
        assert_eq!(state.selected_index(), 4);
        assert_eq!(state.visible_range(), 4..6);
        state.resize_viewport(3);
        assert_eq!(state.visible_range(), 4..7);
    }

    #[rstest]
    #[case("A\u{0301}")]
    #[case("👩🏽‍💻")]
    #[case("\r\n")]
    #[case("🇯🇵")]
    fn single_grapheme_group_moves_preserve_the_selected_member(#[case] source: &str) {
        let mut state = SequenceState::new(source.to_owned());
        state.resize_viewport(1);
        state.move_selection(SequenceMove::Last);
        let selected = state.selected_index();
        let visible = state.visible_range();
        for movement in [SequenceMove::PreviousGroup, SequenceMove::NextGroup] {
            assert!(!state.move_selection(movement));
            assert_eq!(state.selected_index(), selected);
            assert_eq!(state.visible_range(), visible);
        }
        assert_eq!(state.analysis().source(), source);
    }
}
