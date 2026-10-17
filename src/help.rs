use std::ops::Range;

pub use crate::viewport::DocumentMove as HelpMove;
use crate::viewport::DocumentViewport;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HelpState {
    open: bool,
    viewport: DocumentViewport,
}

impl HelpState {
    pub const fn is_open(self) -> bool {
        self.open
    }

    #[cfg(test)]
    pub fn offset(self) -> usize {
        self.viewport.visible_range().start
    }

    pub fn visible_range(self) -> Range<usize> {
        self.viewport.visible_range()
    }

    pub fn toggle(&mut self) {
        if self.open {
            self.close();
        } else {
            self.open();
        }
    }

    pub fn open(&mut self) {
        self.open = true;
        self.viewport.move_viewport(HelpMove::First);
    }

    pub fn close(&mut self) {
        self.open = false;
        self.viewport.move_viewport(HelpMove::First);
    }

    pub fn resize_viewport(&mut self, viewport_height: usize, document_height: usize) {
        self.viewport
            .resize_viewport(viewport_height, document_height);
    }

    pub fn move_viewport(&mut self, movement: HelpMove) {
        self.viewport.move_viewport(movement);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opens_at_the_top_and_closes_cleanly() {
        let mut state = HelpState::default();
        state.open();
        state.resize_viewport(5, 20);
        state.move_viewport(HelpMove::Last);

        assert!(state.is_open());
        assert_eq!(state.offset(), 15);

        state.close();

        assert!(!state.is_open());
        assert_eq!(state.offset(), 0);
    }

    #[test]
    fn movement_is_clamped_to_the_document() {
        let mut state = HelpState::default();
        state.open();
        state.resize_viewport(5, 12);

        state.move_viewport(HelpMove::PageForward);
        assert_eq!(state.offset(), 5);
        state.move_viewport(HelpMove::PageForward);
        assert_eq!(state.offset(), 7);
        state.move_viewport(HelpMove::LineBackward);
        assert_eq!(state.offset(), 6);
        state.move_viewport(HelpMove::First);
        assert_eq!(state.offset(), 0);
    }
}
