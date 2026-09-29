use std::ops::Range;

use ratatui::{buffer::Buffer, layout::Rect, style::Style, widgets::Widget};

const THUMB_SYMBOL: &str = "┃";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewportScrollbar {
    content_length: usize,
    visible_range: Range<usize>,
    style: Style,
}

impl ViewportScrollbar {
    pub const fn new(content_length: usize, visible_range: Range<usize>) -> Self {
        Self {
            content_length,
            visible_range,
            style: Style::new(),
        }
    }

    pub const fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    fn thumb_range(&self, track_length: usize) -> Option<Range<usize>> {
        if track_length == 0 || self.content_length == 0 {
            return None;
        }

        let start = self.visible_range.start.min(self.content_length);
        let end = self.visible_range.end.max(start).min(self.content_length);
        let visible_length = end - start;
        if visible_length == 0 || visible_length >= self.content_length {
            return None;
        }

        let thumb_length = track_length
            .saturating_mul(visible_length)
            .checked_div(self.content_length)
            .unwrap_or_default()
            .clamp(1, track_length);
        let maximum_offset = self.content_length - visible_length;
        let offset = start.min(maximum_offset);
        let thumb_start = track_length
            .saturating_sub(thumb_length)
            .saturating_mul(offset)
            / maximum_offset;

        Some(thumb_start..thumb_start + thumb_length)
    }
}

impl Widget for ViewportScrollbar {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        if area.width == 0 {
            return;
        }
        let Some(thumb) = self.thumb_range(usize::from(area.height)) else {
            return;
        };

        for offset in thumb {
            let y = area.y.saturating_add(offset as u16);
            if let Some(cell) = buffer.cell_mut((area.x, y)) {
                cell.set_symbol(THUMB_SYMBOL).set_style(self.style);
            }
        }
    }
}

pub fn area_after(content: Rect) -> Rect {
    Rect::new(content.right(), content.y, 1, content.height)
}

pub fn area_for_primary(frame: Rect, primary: Rect, content: Rect) -> Rect {
    if primary.right() < frame.right() {
        Rect::new(primary.right(), content.y, 1, content.height)
    } else {
        area_after(content)
    }
}

#[cfg(test)]
mod tests {
    use ratatui::buffer::Buffer;

    use super::*;

    #[test]
    fn follows_the_visible_range_from_top_to_bottom() {
        assert_eq!(render(20, 0..10, 10), "┃┃┃┃┃     ");
        assert_eq!(render(20, 5..15, 10), "  ┃┃┃┃┃   ");
        assert_eq!(render(20, 10..20, 10), "     ┃┃┃┃┃");
    }

    #[test]
    fn keeps_the_thumb_visible_for_a_small_viewport_fraction() {
        assert_eq!(render(100, 50..51, 5), "  ┃  ");
    }

    #[test]
    fn omits_the_scrollbar_when_the_whole_range_is_visible() {
        assert_eq!(render(5, 0..5, 5), "     ");
        assert_eq!(render(5, 0..5, 10), "          ");
    }

    #[test]
    fn omits_the_scrollbar_for_empty_content_ranges_and_areas() {
        assert_eq!(render(0, 0..0, 5), "     ");
        assert_eq!(render(5, 0..0, 5), "     ");
        assert_eq!(render(5, 0..1, 0), "");
    }

    fn render(content_length: usize, visible_range: Range<usize>, height: u16) -> String {
        let area = Rect::new(0, 0, 1, height);
        let mut buffer = Buffer::empty(area);
        ViewportScrollbar::new(content_length, visible_range).render(area, &mut buffer);
        (0..height)
            .map(|y| buffer.cell((0, y)).unwrap().symbol())
            .collect()
    }
}
