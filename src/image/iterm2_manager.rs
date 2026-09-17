use std::{io, num::NonZeroU16};

use ratatui::{
    buffer::{Buffer, CellDiffOption},
    layout::Rect,
};

use crate::image::{ImageKey, RgbaImage, iterm2};

struct ActiveImage<Content> {
    key: ImageKey<Content, Rect>,
    sequence: String,
}

pub struct Iterm2ImageManager<Content> {
    active: Option<ActiveImage<Content>>,
    generation: u64,
}

impl<Content> Iterm2ImageManager<Content> {
    pub const fn new() -> Self {
        Self {
            active: None,
            generation: 0,
        }
    }

    pub fn prepare(
        &mut self,
        content: Content,
        image: RgbaImage<'_>,
        area: Rect,
    ) -> io::Result<bool>
    where
        Content: Copy + PartialEq,
    {
        let key = ImageKey::new(content, area);
        if self.active.as_ref().is_some_and(|active| active.key == key) {
            return Ok(false);
        }

        self.generation = self.generation.wrapping_add(1);
        self.active = Some(ActiveImage {
            key,
            sequence: iterm2::encode_rgba(image, area, self.generation)?,
        });
        Ok(true)
    }

    pub fn hide(&mut self) -> bool {
        self.active.take().is_some()
    }

    pub fn render(&self, buffer: &mut Buffer) {
        let Some(active) = self.active.as_ref() else {
            return;
        };
        let area = active.key.placement;
        let Some(width) = NonZeroU16::new(area.width) else {
            return;
        };

        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                let Some(cell) = buffer.cell_mut((x, y)) else {
                    continue;
                };
                if x == area.left() && y == area.top() {
                    cell.set_symbol(&active.sequence)
                        .set_diff_option(CellDiffOption::ForcedWidth(width));
                } else {
                    cell.set_diff_option(CellDiffOption::Skip);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU16;

    use ratatui::{
        buffer::{Buffer, CellDiffOption},
        layout::Rect,
    };

    use crate::image::{RgbaImage, iterm2_manager::Iterm2ImageManager};

    const AREA: Rect = Rect::new(3, 4, 2, 2);
    const PIXELS: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
    const CONTENT: u8 = 1;
    const CHANGED_CONTENT: u8 = 2;

    #[test]
    fn suppresses_an_unchanged_preparation() {
        let mut manager = Iterm2ImageManager::new();
        let image = RgbaImage::new(2, 2, &PIXELS).unwrap();

        assert!(manager.prepare(CONTENT, image, AREA).unwrap());
        assert!(!manager.prepare(CONTENT, image, AREA).unwrap());
    }

    #[test]
    fn prepares_changed_content_again() {
        let mut manager = Iterm2ImageManager::new();
        let image = RgbaImage::new(2, 2, &PIXELS).unwrap();

        assert!(manager.prepare(CONTENT, image, AREA).unwrap());
        assert!(manager.prepare(CHANGED_CONTENT, image, AREA).unwrap());
    }

    #[test]
    fn renders_the_sequence_once_and_skips_the_remaining_image_area() {
        let mut manager = Iterm2ImageManager::new();
        let image = RgbaImage::new(2, 2, &PIXELS).unwrap();
        manager.prepare(CONTENT, image, AREA).unwrap();
        let mut buffer = Buffer::empty(Rect::new(0, 0, 8, 8));

        manager.render(&mut buffer);

        let first = buffer.cell((AREA.x, AREA.y)).unwrap();
        assert!(first.symbol().contains("\x1b]1337;"));
        assert_eq!(
            first.diff_option,
            CellDiffOption::ForcedWidth(NonZeroU16::new(AREA.width).unwrap())
        );
        for y in AREA.top()..AREA.bottom() {
            for x in AREA.left()..AREA.right() {
                if (x, y) != (AREA.x, AREA.y) {
                    assert_eq!(
                        buffer.cell((x, y)).unwrap().diff_option,
                        CellDiffOption::Skip
                    );
                }
            }
        }
    }

    #[test]
    fn hides_an_active_image_only_once() {
        let mut manager = Iterm2ImageManager::new();
        let image = RgbaImage::new(2, 2, &PIXELS).unwrap();
        manager.prepare(CONTENT, image, AREA).unwrap();

        assert!(manager.hide());
        assert!(!manager.hide());

        let mut buffer = Buffer::empty(Rect::new(0, 0, 8, 8));
        manager.render(&mut buffer);
        assert!(
            buffer
                .content
                .iter()
                .all(|cell| cell.diff_option == CellDiffOption::None)
        );
    }
}
