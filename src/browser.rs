use crate::{
    unicode::{CodePoint, Plane, plane::PlaneRange},
    viewport::ListViewport,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowseLevel {
    Plane,
    Range,
    CodePointTable,
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
    level: BrowseLevel,
    cursor: CodePoint,
    plane_viewport: ListViewport,
    range_viewport: ListViewport,
    table_viewport: ListViewport,
}

impl BrowseState {
    pub const fn at(level: BrowseLevel, cursor: CodePoint) -> Self {
        Self {
            level,
            cursor,
            plane_viewport: ListViewport::new(),
            range_viewport: ListViewport::new(),
            table_viewport: ListViewport::new(),
        }
    }

    pub const fn level(self) -> BrowseLevel {
        self.level
    }

    pub const fn cursor(self) -> CodePoint {
        self.cursor
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
        self.table_viewport.resize(
            height.saturating_sub(1),
            usize::from(table_row(self.cursor)),
            16,
        );
    }

    pub fn visible_list_items(&self) -> Option<std::ops::Range<usize>> {
        match self.level {
            BrowseLevel::Plane => Some(self.plane_viewport.visible_range(Plane::COUNT)),
            BrowseLevel::Range => Some(
                self.range_viewport
                    .visible_range(PlaneRange::COUNT_PER_PLANE),
            ),
            BrowseLevel::CodePointTable => None,
        }
    }

    pub fn visible_table_rows(&self) -> Option<std::ops::Range<usize>> {
        (self.level == BrowseLevel::CodePointTable).then(|| self.table_viewport.visible_range(16))
    }

    pub fn advance(&mut self) -> BrowseTarget {
        match self.level {
            BrowseLevel::Plane => {
                self.level = BrowseLevel::Range;
                self.ensure_current_item_visible();
            }
            BrowseLevel::Range => {
                self.level = BrowseLevel::CodePointTable;
                self.ensure_current_item_visible();
            }
            BrowseLevel::CodePointTable => return BrowseTarget::Inspector,
        }
        BrowseTarget::Browser
    }

    pub fn back(&mut self) {
        match self.level {
            BrowseLevel::Plane => {}
            BrowseLevel::Range => {
                self.level = BrowseLevel::Plane;
                self.ensure_current_item_visible();
            }
            BrowseLevel::CodePointTable => {
                self.level = BrowseLevel::Range;
                self.ensure_current_item_visible();
            }
        }
    }

    pub fn move_cursor(&mut self, movement: BrowseMove) -> bool {
        let previous = self.cursor;
        self.cursor = match self.level {
            BrowseLevel::Plane => move_plane(self.cursor, movement),
            BrowseLevel::Range => move_range(self.cursor, movement),
            BrowseLevel::CodePointTable => move_table(self.cursor, movement),
        };
        self.ensure_current_item_visible();

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
            BrowseLevel::CodePointTable => {
                self.table_viewport
                    .ensure_visible(usize::from(table_row(self.cursor)), 16);
            }
        }
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
        state.back();
        assert_eq!(state.level(), BrowseLevel::Range);
        state.back();
        assert_eq!(state.level(), BrowseLevel::Plane);
        state.back();
        assert_eq!(state.level(), BrowseLevel::Plane);
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

    fn browse_state(value: u32) -> BrowseState {
        BrowseState::at(BrowseLevel::Plane, CodePoint::new(value).unwrap())
    }

    fn browse_state_at(level: BrowseLevel, value: u32) -> BrowseState {
        let mut state = browse_state(value);
        state.level = level;
        state
    }
}
