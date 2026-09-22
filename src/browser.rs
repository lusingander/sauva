use crate::{
    unicode::{CodePoint, Plane, UnicodeBlock, UnicodeDatabase, plane::PlaneRange},
    viewport::ListViewport,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowseLevel {
    Plane,
    Range,
    Block,
    CodePointTable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TableSource {
    Range,
    Block,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowseMove {
    Up,
    Down,
    Left,
    Right,
    LargeBackward,
    LargeForward,
    First,
    Last,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowseTarget {
    Browser,
    Inspector,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrowseState {
    entry_level: BrowseLevel,
    level: BrowseLevel,
    cursor: CodePoint,
    block_index: usize,
    table_source: TableSource,
    plane_viewport: ListViewport,
    range_viewport: ListViewport,
    block_viewport: ListViewport,
    table_viewport: ListViewport,
}

impl BrowseState {
    pub fn at(level: BrowseLevel, cursor: CodePoint) -> Self {
        let block = UnicodeDatabase::block_at_or_after(cursor)
            .expect("the final Unicode block includes the last code point");
        Self {
            entry_level: level,
            level,
            cursor: if level == BrowseLevel::Block && !block.contains(cursor) {
                block.start()
            } else {
                cursor
            },
            block_index: block.index(),
            table_source: TableSource::Range,
            plane_viewport: ListViewport::new(),
            range_viewport: ListViewport::new(),
            block_viewport: ListViewport::new(),
            table_viewport: ListViewport::new(),
        }
    }

    pub const fn level(self) -> BrowseLevel {
        self.level
    }

    pub const fn cursor(self) -> CodePoint {
        self.cursor
    }

    pub fn selected_block(self) -> Option<UnicodeBlock> {
        (self.level == BrowseLevel::Block || self.table_source == TableSource::Block)
            .then(|| UnicodeDatabase::block(self.block_index))
            .flatten()
    }

    pub fn table_page(self) -> Option<(CodePoint, CodePoint)> {
        (self.level == BrowseLevel::CodePointTable).then(|| match self.table_source {
            TableSource::Range => {
                let range = PlaneRange::for_code_point(self.cursor);
                (range.start(), range.end())
            }
            TableSource::Block => {
                let block = self.block();
                let start = block.start().value()
                    + ((self.cursor.value() - block.start().value()) / 256) * 256;
                let end = (start + 255).min(block.end().value());
                (valid_code_point(start), valid_code_point(end))
            }
        })
    }

    pub fn is_block_table(self) -> bool {
        self.level == BrowseLevel::CodePointTable && self.table_source == TableSource::Block
    }

    pub fn resize_list_viewports(&mut self, height: usize) {
        self.plane_viewport.resize(
            height,
            usize::from(Plane::for_code_point(self.cursor).number()),
            Plane::COUNT,
        );
        self.range_viewport.resize(
            height,
            usize::from(PlaneRange::for_code_point(self.cursor).index()),
            PlaneRange::COUNT_PER_PLANE,
        );
        self.block_viewport
            .resize(height, self.block_index, UnicodeDatabase::blocks().len());
        let table_rows = self.table_row_count();
        self.table_viewport
            .resize(height.saturating_sub(1), self.table_row(), table_rows);
    }

    pub fn visible_list_items(&self) -> Option<std::ops::Range<usize>> {
        match self.level {
            BrowseLevel::Plane => Some(self.plane_viewport.visible_range(Plane::COUNT)),
            BrowseLevel::Range => Some(
                self.range_viewport
                    .visible_range(PlaneRange::COUNT_PER_PLANE),
            ),
            BrowseLevel::Block => Some(
                self.block_viewport
                    .visible_range(UnicodeDatabase::blocks().len()),
            ),
            BrowseLevel::CodePointTable => None,
        }
    }

    pub fn visible_table_rows(&self) -> Option<std::ops::Range<usize>> {
        (self.level == BrowseLevel::CodePointTable)
            .then(|| self.table_viewport.visible_range(self.table_row_count()))
    }

    pub fn advance(&mut self) -> BrowseTarget {
        match self.level {
            BrowseLevel::Plane => {
                self.level = BrowseLevel::Range;
                self.ensure_current_item_visible();
            }
            BrowseLevel::Range => {
                self.table_source = TableSource::Range;
                self.level = BrowseLevel::CodePointTable;
                self.ensure_current_item_visible();
            }
            BrowseLevel::Block => {
                self.table_source = TableSource::Block;
                self.level = BrowseLevel::CodePointTable;
                self.ensure_current_item_visible();
            }
            BrowseLevel::CodePointTable => return BrowseTarget::Inspector,
        }
        BrowseTarget::Browser
    }

    pub fn back(&mut self) -> BrowseTarget {
        if self.level == self.entry_level {
            return BrowseTarget::Inspector;
        }
        match self.level {
            BrowseLevel::Plane => return BrowseTarget::Inspector,
            BrowseLevel::Range => {
                self.level = BrowseLevel::Plane;
                self.ensure_current_item_visible();
            }
            BrowseLevel::Block => return BrowseTarget::Inspector,
            BrowseLevel::CodePointTable => {
                self.level = match self.table_source {
                    TableSource::Range => BrowseLevel::Range,
                    TableSource::Block => BrowseLevel::Block,
                };
                self.ensure_current_item_visible();
            }
        }
        BrowseTarget::Browser
    }

    pub fn move_cursor(&mut self, movement: BrowseMove) -> bool {
        let previous = self.cursor;
        self.cursor = match self.level {
            BrowseLevel::Plane => move_plane(self.cursor, movement),
            BrowseLevel::Range => move_range(self.cursor, movement),
            BrowseLevel::Block => self.move_block(movement),
            BrowseLevel::CodePointTable => match self.table_source {
                TableSource::Range => move_table(self.cursor, movement),
                TableSource::Block => move_block_table(self.cursor, movement, self.block()),
            },
        };
        self.ensure_current_item_visible();

        debug_assert!(
            !self.is_block_table() || self.block().contains(self.cursor),
            "block table cursor must remain within the selected block"
        );
        debug_assert!(PlaneRange::for_code_point(self.cursor).contains(self.cursor));
        self.cursor != previous
    }

    fn ensure_current_item_visible(&mut self) {
        match self.level {
            BrowseLevel::Plane => self.plane_viewport.ensure_visible(
                usize::from(Plane::for_code_point(self.cursor).number()),
                Plane::COUNT,
            ),
            BrowseLevel::Range => self.range_viewport.ensure_visible(
                usize::from(PlaneRange::for_code_point(self.cursor).index()),
                PlaneRange::COUNT_PER_PLANE,
            ),
            BrowseLevel::Block => self
                .block_viewport
                .ensure_visible(self.block_index, UnicodeDatabase::blocks().len()),
            BrowseLevel::CodePointTable => {
                self.table_viewport
                    .ensure_visible(self.table_row(), self.table_row_count());
            }
        }
    }

    fn block(self) -> UnicodeBlock {
        UnicodeDatabase::block(self.block_index).expect("browser block index is valid")
    }

    fn table_row_count(self) -> usize {
        self.table_page().map_or(16, |(start, end)| {
            ((end.value() - start.value() + 1) / 16) as usize
        })
    }

    fn table_row(self) -> usize {
        self.table_page()
            .map_or(usize::from(table_row(self.cursor)), |(start, _)| {
                ((self.cursor.value() - start.value()) / 16) as usize
            })
    }

    fn move_block(&mut self, movement: BrowseMove) -> CodePoint {
        let last = UnicodeDatabase::blocks().len() - 1;
        let target = match movement {
            BrowseMove::Up => self.block_index.saturating_sub(1),
            BrowseMove::Down => self.block_index.saturating_add(1).min(last),
            BrowseMove::LargeBackward => self.block_index.saturating_sub(16),
            BrowseMove::LargeForward => self.block_index.saturating_add(16).min(last),
            BrowseMove::First => 0,
            BrowseMove::Last => last,
            _ => self.block_index,
        };
        if target == self.block_index {
            return self.cursor;
        }
        self.block_index = target;
        self.block().start()
    }
}

fn table_row(cursor: CodePoint) -> u8 {
    (cursor.value() & 0xff) as u8 / 16
}

fn move_plane(cursor: CodePoint, movement: BrowseMove) -> CodePoint {
    let plane = Plane::for_code_point(cursor);
    let target = match movement {
        BrowseMove::Up => plane.number().checked_sub(1),
        BrowseMove::Down => plane.number().checked_add(1),
        BrowseMove::First => Some(0),
        BrowseMove::Last => Some(Plane::LAST_NUMBER),
        _ => None,
    };

    target.and_then(Plane::new).map_or(cursor, |target| {
        valid_code_point(target.start().value() | (cursor.value() & 0xffff))
    })
}

fn move_range(cursor: CodePoint, movement: BrowseMove) -> CodePoint {
    let range = PlaneRange::for_code_point(cursor);
    let target = match movement {
        BrowseMove::Up => Some(range.index().saturating_sub(1)),
        BrowseMove::Down => Some(range.index().saturating_add(1)),
        BrowseMove::LargeBackward => Some(range.index().saturating_sub(16)),
        BrowseMove::LargeForward => Some(range.index().saturating_add(16)),
        BrowseMove::First => Some(0),
        BrowseMove::Last => Some(u8::MAX),
        _ => None,
    };

    let offset = range
        .offset_of(cursor)
        .expect("a code point belongs to its derived range");
    target.map_or(cursor, |index| {
        range.plane().range(index).code_point(offset)
    })
}

fn move_table(cursor: CodePoint, movement: BrowseMove) -> CodePoint {
    let plane = Plane::for_code_point(cursor);
    let range = PlaneRange::for_code_point(cursor);
    let value = cursor.value();
    let target = match movement {
        BrowseMove::Left if value > plane.start().value() => Some(value - 1),
        BrowseMove::Right if value < plane.end().value() => Some(value + 1),
        BrowseMove::Up if value - plane.start().value() >= 16 => Some(value - 16),
        BrowseMove::Down if plane.end().value() - value >= 16 => Some(value + 16),
        BrowseMove::LargeBackward => range.index().checked_sub(1).map(|index| {
            range
                .plane()
                .range(index)
                .code_point(range_offset(range, cursor))
                .value()
        }),
        BrowseMove::LargeForward => range.index().checked_add(1).map(|index| {
            range
                .plane()
                .range(index)
                .code_point(range_offset(range, cursor))
                .value()
        }),
        BrowseMove::First => Some(range.start().value()),
        BrowseMove::Last => Some(range.end().value()),
        _ => None,
    };

    target.map_or(cursor, valid_code_point)
}

fn move_block_table(cursor: CodePoint, movement: BrowseMove, block: UnicodeBlock) -> CodePoint {
    let start = block.start().value();
    let end = block.end().value();
    let value = cursor.value();
    let page = (value - start) / 256;
    let target = match movement {
        BrowseMove::Left if value > start => Some(value - 1),
        BrowseMove::Right if value < end => Some(value + 1),
        BrowseMove::Up if value - start >= 16 => Some(value - 16),
        BrowseMove::Down if end - value >= 16 => Some(value + 16),
        BrowseMove::LargeBackward if page > 0 => {
            Some(start + (page - 1) * 256 + (value - start) % 256)
        }
        BrowseMove::LargeForward if start + (page + 1) * 256 <= end => {
            Some((start + (page + 1) * 256 + (value - start) % 256).min(end))
        }
        BrowseMove::First => Some(start),
        BrowseMove::Last => Some(end),
        _ => None,
    };
    target.map_or(cursor, valid_code_point)
}

fn range_offset(range: PlaneRange, cursor: CodePoint) -> u8 {
    range
        .offset_of(cursor)
        .expect("a code point belongs to its derived range")
}

fn valid_code_point(value: u32) -> CodePoint {
    CodePoint::new(value).expect("browser-derived values are valid Unicode code points")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advances_and_goes_back_through_the_browse_hierarchy() {
        let mut state = browse_state(0x0041);

        assert_eq!(state.level(), BrowseLevel::Plane);
        assert_eq!(state.advance(), BrowseTarget::Browser);
        assert_eq!(state.level(), BrowseLevel::Range);
        assert_eq!(state.advance(), BrowseTarget::Browser);
        assert_eq!(state.level(), BrowseLevel::CodePointTable);
        assert_eq!(state.advance(), BrowseTarget::Inspector);
        assert_eq!(state.level(), BrowseLevel::CodePointTable);
        assert_eq!(state.back(), BrowseTarget::Browser);
        assert_eq!(state.level(), BrowseLevel::Range);
        assert_eq!(state.back(), BrowseTarget::Browser);
        assert_eq!(state.level(), BrowseLevel::Plane);
        assert_eq!(state.back(), BrowseTarget::Inspector);
        assert_eq!(state.level(), BrowseLevel::Plane);

        let mut direct_range = browse_state_at(BrowseLevel::Range, 0x0041);
        assert_eq!(direct_range.back(), BrowseTarget::Inspector);
        let mut direct_table = browse_state_at(BrowseLevel::CodePointTable, 0x0041);
        assert_eq!(direct_table.back(), BrowseTarget::Inspector);
    }

    #[test]
    fn moves_between_planes_while_preserving_the_plane_offset() {
        let mut state = browse_state(0x2_abcd);

        assert!(state.move_cursor(BrowseMove::Up));
        assert_eq!(state.cursor().value(), 0x1_abcd);
        assert!(state.move_cursor(BrowseMove::First));
        assert_eq!(state.cursor().value(), 0xabcd);
        assert!(!state.move_cursor(BrowseMove::Up));
        assert!(state.move_cursor(BrowseMove::Last));
        assert_eq!(state.cursor().value(), 0x10_abcd);
        assert!(!state.move_cursor(BrowseMove::Down));
    }

    #[test]
    fn moves_between_ranges_while_preserving_the_range_offset() {
        let mut state = browse_state_at(BrowseLevel::Range, 0x1_23ab);

        assert!(state.move_cursor(BrowseMove::Up));
        assert_eq!(state.cursor().value(), 0x1_22ab);
        assert!(state.move_cursor(BrowseMove::LargeBackward));
        assert_eq!(state.cursor().value(), 0x1_12ab);
        assert!(state.move_cursor(BrowseMove::First));
        assert_eq!(state.cursor().value(), 0x1_00ab);
        assert!(!state.move_cursor(BrowseMove::Up));
        assert!(state.move_cursor(BrowseMove::Last));
        assert_eq!(state.cursor().value(), 0x1_ffab);
        assert!(!state.move_cursor(BrowseMove::LargeForward));

        let mut near_first = browse_state_at(BrowseLevel::Range, 0x1_05ab);
        assert!(near_first.move_cursor(BrowseMove::LargeBackward));
        assert_eq!(near_first.cursor().value(), 0x1_00ab);
        let mut near_last = browse_state_at(BrowseLevel::Range, 0x1_faab);
        assert!(near_last.move_cursor(BrowseMove::LargeForward));
        assert_eq!(near_last.cursor().value(), 0x1_ffab);
    }

    #[test]
    fn table_directional_movement_crosses_ranges_but_not_planes() {
        let mut state = browse_state_at(BrowseLevel::CodePointTable, 0x00ff);

        assert!(state.move_cursor(BrowseMove::Right));
        assert_eq!(state.cursor().value(), 0x0100);
        assert!(state.move_cursor(BrowseMove::Up));
        assert_eq!(state.cursor().value(), 0x00f0);

        let mut first = browse_state_at(BrowseLevel::CodePointTable, 0x0000);
        assert!(!first.move_cursor(BrowseMove::Left));
        assert!(!first.move_cursor(BrowseMove::Up));

        let mut last = browse_state_at(BrowseLevel::CodePointTable, 0x10_ffff);
        assert!(!last.move_cursor(BrowseMove::Right));
        assert!(!last.move_cursor(BrowseMove::Down));
    }

    #[test]
    fn table_large_movement_preserves_offset_and_stops_at_plane_boundaries() {
        let mut state = browse_state_at(BrowseLevel::CodePointTable, 0x1_23ab);

        assert!(state.move_cursor(BrowseMove::LargeBackward));
        assert_eq!(state.cursor().value(), 0x1_22ab);
        assert!(state.move_cursor(BrowseMove::LargeForward));
        assert_eq!(state.cursor().value(), 0x1_23ab);
        assert!(state.move_cursor(BrowseMove::First));
        assert_eq!(state.cursor().value(), 0x1_2300);
        assert!(state.move_cursor(BrowseMove::Last));
        assert_eq!(state.cursor().value(), 0x1_23ff);

        let mut first = browse_state_at(BrowseLevel::CodePointTable, 0x1_00ab);
        assert!(!first.move_cursor(BrowseMove::LargeBackward));
        let mut last = browse_state_at(BrowseLevel::CodePointTable, 0x1_ffab);
        assert!(!last.move_cursor(BrowseMove::LargeForward));
    }

    #[test]
    fn list_viewports_scroll_at_edges_and_are_retained_per_level() {
        let mut state = browse_state(0x0041);
        state.resize_list_viewports(10);

        for _ in 0..10 {
            state.move_cursor(BrowseMove::Down);
        }
        assert_eq!(state.visible_list_items(), Some(1..11));
        state.move_cursor(BrowseMove::Up);
        assert_eq!(state.visible_list_items(), Some(1..11));

        state.advance();
        assert_eq!(state.visible_list_items(), Some(0..10));

        for _ in 0..10 {
            state.move_cursor(BrowseMove::Down);
        }
        assert_eq!(state.visible_list_items(), Some(1..11));

        state.move_cursor(BrowseMove::Up);
        assert_eq!(state.visible_list_items(), Some(1..11));
        state.back();
        assert_eq!(state.visible_list_items(), Some(1..11));
        state.advance();
        assert_eq!(state.visible_list_items(), Some(1..11));
    }

    #[test]
    fn table_viewport_keeps_the_column_header_and_scrolls_only_at_row_edges() {
        let mut state = browse_state_at(BrowseLevel::CodePointTable, 0x0041);
        state.resize_list_viewports(10);

        assert_eq!(state.visible_table_rows(), Some(0..9));
        for _ in 0..5 {
            state.move_cursor(BrowseMove::Down);
        }
        assert_eq!(state.cursor().value(), 0x0091);
        assert_eq!(state.visible_table_rows(), Some(1..10));

        state.move_cursor(BrowseMove::Up);
        assert_eq!(state.cursor().value(), 0x0081);
        assert_eq!(state.visible_table_rows(), Some(1..10));

        state.move_cursor(BrowseMove::Last);
        assert_eq!(state.visible_table_rows(), Some(7..16));
        state.move_cursor(BrowseMove::Right);
        assert_eq!(state.cursor().value(), 0x0100);
        assert_eq!(state.visible_table_rows(), Some(0..9));
    }

    #[test]
    fn block_list_selects_the_next_named_block_from_a_gap() {
        let mut state = browse_state_at(BrowseLevel::Block, 0x2fe0);
        state.resize_list_viewports(10);

        assert_eq!(
            state.selected_block().unwrap().name(),
            "Ideographic Description Characters"
        );
        assert_eq!(state.cursor().value(), 0x2ff0);
        assert!(state.move_cursor(BrowseMove::Up));
        assert_eq!(state.selected_block().unwrap().name(), "Kangxi Radicals");
        assert_eq!(state.cursor().value(), 0x2f00);
        let index = state.selected_block().unwrap().index();
        assert!(state.move_cursor(BrowseMove::LargeBackward));
        assert_eq!(state.selected_block().unwrap().index(), index - 16);
        assert!(state.move_cursor(BrowseMove::LargeForward));
        assert_eq!(state.selected_block().unwrap().index(), index);
        assert!(state.move_cursor(BrowseMove::Last));
        assert_eq!(
            state.selected_block().unwrap().end().value(),
            CodePoint::MAX_VALUE
        );
        assert!(!state.move_cursor(BrowseMove::Down));
        assert_eq!(
            state.visible_list_items().unwrap().end,
            UnicodeDatabase::blocks().len()
        );
        assert!(state.move_cursor(BrowseMove::First));
        assert_eq!(state.selected_block().unwrap().name(), "Basic Latin");
    }

    #[test]
    fn block_table_limits_movement_to_a_short_block_and_returns_to_its_list() {
        let mut state = browse_state_at(BrowseLevel::Block, 0x2ff5);
        state.resize_list_viewports(10);
        assert_eq!(state.advance(), BrowseTarget::Browser);
        assert!(state.is_block_table());
        assert_eq!(state.table_page().unwrap().0.value(), 0x2ff0);
        assert_eq!(state.visible_table_rows(), Some(0..1));
        assert!(!state.move_cursor(BrowseMove::LargeForward));
        assert!(!state.move_cursor(BrowseMove::Up));
        assert!(state.move_cursor(BrowseMove::Last));
        assert_eq!(state.cursor().value(), 0x2fff);
        assert!(!state.move_cursor(BrowseMove::Right));
        assert!(!state.move_cursor(BrowseMove::Down));
        assert_eq!(state.back(), BrowseTarget::Browser);
        assert_eq!(state.level(), BrowseLevel::Block);
        assert_eq!(
            state.selected_block().unwrap().name(),
            "Ideographic Description Characters"
        );
        assert_eq!(state.back(), BrowseTarget::Inspector);
    }

    #[test]
    fn block_table_pages_through_a_long_block_and_clips_the_final_page() {
        let mut state = browse_state_at(BrowseLevel::Block, 0x2_00ab);
        state.resize_list_viewports(10);
        assert_eq!(
            state.selected_block().unwrap().name(),
            "CJK Unified Ideographs Extension B"
        );
        state.advance();
        assert_eq!(state.table_page().unwrap().0.value(), 0x2_0000);
        assert_eq!(state.table_page().unwrap().1.value(), 0x2_00ff);
        assert!(state.move_cursor(BrowseMove::LargeForward));
        assert_eq!(state.cursor().value(), 0x2_01ab);
        assert!(state.move_cursor(BrowseMove::Last));
        assert_eq!(state.cursor().value(), 0x2_a6df);
        assert_eq!(state.table_page().unwrap().0.value(), 0x2_a600);
        assert_eq!(state.visible_table_rows().unwrap().end, 14);
        assert!(!state.move_cursor(BrowseMove::LargeForward));
        assert!(!state.move_cursor(BrowseMove::Right));
        assert!(state.move_cursor(BrowseMove::LargeBackward));
        assert_eq!(state.cursor().value(), 0x2_a5df);
    }

    #[test]
    fn block_table_pages_are_relative_to_the_block_start() {
        let mut state = browse_state_at(BrowseLevel::Block, 0xfb60);
        assert_eq!(
            state.selected_block().unwrap().name(),
            "Arabic Presentation Forms-A"
        );
        state.advance();
        assert_eq!(state.table_page().unwrap().0.value(), 0xfb50);
        assert_eq!(state.table_page().unwrap().1.value(), 0xfc4f);
        assert!(state.move_cursor(BrowseMove::LargeForward));
        assert_eq!(state.cursor().value(), 0xfc60);
        assert_eq!(state.table_page().unwrap().0.value(), 0xfc50);
        assert!(state.move_cursor(BrowseMove::Last));
        assert_eq!(state.table_page().unwrap().0.value(), 0xfd50);
        assert_eq!(state.table_page().unwrap().1.value(), 0xfdff);
    }

    fn browse_state(value: u32) -> BrowseState {
        BrowseState::at(BrowseLevel::Plane, CodePoint::new(value).unwrap())
    }

    fn browse_state_at(level: BrowseLevel, value: u32) -> BrowseState {
        BrowseState::at(level, CodePoint::new(value).unwrap())
    }
}
