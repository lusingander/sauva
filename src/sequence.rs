use std::ops::Range;

use crate::{unicode::CodePoint, viewport::ListViewport};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SequenceMove {
    Previous,
    Next,
    First,
    Last,
}

#[derive(Debug, Clone)]
pub struct SequenceState {
    code_points: Vec<CodePoint>,
    selected_index: usize,
    viewport: ListViewport,
}

impl SequenceState {
    pub fn new(code_points: Vec<CodePoint>) -> Self {
        assert!(
            code_points.len() > 1,
            "a sequence contains at least two code points"
        );
        Self {
            code_points,
            selected_index: 0,
            viewport: ListViewport::new(),
        }
    }

    pub fn code_points(&self) -> &[CodePoint] {
        &self.code_points
    }

    pub const fn selected_index(&self) -> usize {
        self.selected_index
    }

    pub fn selected(&self) -> CodePoint {
        self.code_points[self.selected_index]
    }

    pub fn visible_range(&self) -> Range<usize> {
        self.viewport.visible_range(self.code_points.len())
    }

    pub fn move_selection(&mut self, movement: SequenceMove) -> bool {
        let last = self.code_points.len() - 1;
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
        self.viewport.ensure_visible(next, self.code_points.len());
        true
    }

    pub fn resize_viewport(&mut self, height: usize) {
        self.viewport
            .resize(height, self.selected_index, self.code_points.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> SequenceState {
        SequenceState::new("A→A".chars().map(CodePoint::from).collect())
    }

    #[test]
    fn preserves_input_order_and_duplicates() {
        let state = state();

        assert_eq!(
            state
                .code_points()
                .iter()
                .map(|code_point| code_point.value())
                .collect::<Vec<_>>(),
            [0x0041, 0x2192, 0x0041]
        );
        assert_eq!(state.selected_index(), 0);
        assert_eq!(state.selected().value(), 0x0041);
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
