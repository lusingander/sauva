use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpMove {
    LineBackward,
    LineForward,
    PageBackward,
    PageForward,
    First,
    Last,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HelpState {
    open: bool,
    offset: usize,
    viewport_height: usize,
    document_height: usize,
}

impl HelpState {
    pub const fn new() -> Self {
        Self {
            open: false,
            offset: 0,
            viewport_height: 0,
            document_height: 0,
        }
    }

    pub const fn is_open(self) -> bool {
        self.open
    }

    #[cfg(test)]
    pub const fn offset(self) -> usize {
        self.offset
    }

    pub fn visible_range(self) -> Range<usize> {
        let end = self
            .offset
            .saturating_add(self.viewport_height)
            .min(self.document_height);
        self.offset..end
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
        self.offset = 0;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.offset = 0;
    }

    pub fn resize_viewport(&mut self, viewport_height: usize, document_height: usize) {
        self.viewport_height = viewport_height;
        self.document_height = document_height;
        self.clamp();
    }

    pub fn move_viewport(&mut self, movement: HelpMove) {
        let page = self.viewport_height;
        self.offset = match movement {
            HelpMove::LineBackward => self.offset.saturating_sub(1),
            HelpMove::LineForward => self.offset.saturating_add(1),
            HelpMove::PageBackward => self.offset.saturating_sub(page),
            HelpMove::PageForward => self.offset.saturating_add(page),
            HelpMove::First => 0,
            HelpMove::Last => self.maximum_offset(),
        };
        self.clamp();
    }

    const fn maximum_offset(self) -> usize {
        if self.viewport_height == 0 {
            0
        } else {
            self.document_height.saturating_sub(self.viewport_height)
        }
    }

    fn clamp(&mut self) {
        self.offset = self.offset.min(self.maximum_offset());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opens_at_the_top_and_closes_cleanly() {
        let mut state = HelpState::new();
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
        let mut state = HelpState::new();
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
