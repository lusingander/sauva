use std::ops::Range;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ListViewport {
    offset: usize,
    height: usize,
}

impl ListViewport {
    pub const fn new() -> Self {
        Self {
            offset: 0,
            height: 0,
        }
    }

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
        let mut viewport = ListViewport::new();
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
        let mut viewport = ListViewport::new();
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
        let mut viewport = ListViewport::new();
        viewport.resize(10, 128, 256);
        assert_eq!(viewport.visible_range(256), 119..129);

        viewport.ensure_visible(255, 256);
        assert_eq!(viewport.visible_range(256), 246..256);
        viewport.ensure_visible(0, 256);
        assert_eq!(viewport.visible_range(256), 0..10);
    }

    #[test]
    fn resizing_preserves_the_offset_when_possible_and_keeps_selection_visible() {
        let mut viewport = ListViewport::new();
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
        let mut viewport = ListViewport::new();

        viewport.resize(10, 0, 0);
        assert_eq!(viewport.visible_range(0), 0..0);
        viewport.resize(0, 10, 256);
        assert_eq!(viewport.visible_range(256), 0..0);
    }
}
