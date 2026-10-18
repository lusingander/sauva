use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentMove {
    LineBackward,
    LineForward,
    PageBackward,
    PageForward,
    First,
    Last,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DocumentViewport {
    offset: usize,
    viewport_height: usize,
    document_height: usize,
}

impl DocumentViewport {
    pub fn visible_range(self) -> Range<usize> {
        let end = self
            .offset
            .saturating_add(self.viewport_height)
            .min(self.document_height);
        self.offset..end
    }

    pub fn resize_viewport(&mut self, viewport_height: usize, document_height: usize) {
        self.viewport_height = viewport_height;
        self.document_height = document_height;
        self.clamp();
    }

    pub fn move_viewport(&mut self, movement: DocumentMove) {
        let page = self.viewport_height;
        self.offset = match movement {
            DocumentMove::LineBackward => self.offset.saturating_sub(1),
            DocumentMove::LineForward => self.offset.saturating_add(1),
            DocumentMove::PageBackward => self.offset.saturating_sub(page),
            DocumentMove::PageForward => self.offset.saturating_add(page),
            DocumentMove::First => 0,
            DocumentMove::Last => self.maximum_offset(),
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ListViewport {
    offset: usize,
    height: usize,
}

impl ListViewport {
    pub fn resize(&mut self, height: usize, selected: usize, item_count: usize) {
        self.height = height;
        self.ensure_visible(selected, item_count);
    }

    pub fn ensure_visible(&mut self, selected: usize, item_count: usize) {
        let visible = self.height.min(item_count);
        if visible == 0 {
            self.offset = 0;
            return;
        }

        let selected = selected.min(item_count - 1);
        self.offset = self.offset.min(item_count - visible);
        if selected < self.offset {
            self.offset = selected;
        } else if selected >= self.offset + visible {
            self.offset = selected + 1 - visible;
        }
    }

    pub fn align_start(&mut self, selected: usize, item_count: usize) {
        self.offset = selected;
        self.ensure_visible(selected, item_count);
    }

    pub fn visible_range(self, item_count: usize) -> Range<usize> {
        let visible = self.height.min(item_count);
        let offset = self.offset.min(item_count.saturating_sub(visible));
        offset..offset + visible
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrolls_only_after_the_selection_crosses_an_edge() {
        let mut viewport = ListViewport::default();
        viewport.resize(10, 0, 256);

        for selected in 1..10 {
            viewport.ensure_visible(selected, 256);
            assert_eq!(viewport.visible_range(256), 0..10);
        }

        viewport.ensure_visible(10, 256);
        assert_eq!(viewport.visible_range(256), 1..11);
    }

    #[test]
    fn reversing_direction_moves_within_the_window_before_scrolling() {
        let mut viewport = ListViewport::default();
        viewport.resize(10, 10, 256);
        assert_eq!(viewport.visible_range(256), 1..11);

        for selected in (1..10).rev() {
            viewport.ensure_visible(selected, 256);
            assert_eq!(viewport.visible_range(256), 1..11);
        }

        viewport.ensure_visible(0, 256);
        assert_eq!(viewport.visible_range(256), 0..10);
    }

    #[test]
    fn jumps_scroll_only_enough_to_reveal_the_selection() {
        let mut viewport = ListViewport::default();
        viewport.resize(10, 128, 256);
        assert_eq!(viewport.visible_range(256), 119..129);

        viewport.ensure_visible(255, 256);
        assert_eq!(viewport.visible_range(256), 246..256);
        viewport.ensure_visible(0, 256);
        assert_eq!(viewport.visible_range(256), 0..10);
    }

    #[test]
    fn resizing_preserves_the_offset_when_possible_and_keeps_selection_visible() {
        let mut viewport = ListViewport::default();
        viewport.resize(10, 10, 256);
        assert_eq!(viewport.visible_range(256), 1..11);

        viewport.resize(5, 10, 256);
        assert_eq!(viewport.visible_range(256), 6..11);
        viewport.resize(8, 10, 256);
        assert_eq!(viewport.visible_range(256), 6..14);
        viewport.resize(256, 10, 256);
        assert_eq!(viewport.visible_range(256), 0..256);
    }

    #[test]
    fn empty_lists_and_zero_height_have_empty_ranges() {
        let mut viewport = ListViewport::default();

        viewport.resize(10, 0, 0);
        assert_eq!(viewport.visible_range(0), 0..0);
        viewport.resize(0, 10, 256);
        assert_eq!(viewport.visible_range(256), 0..0);
    }
}
