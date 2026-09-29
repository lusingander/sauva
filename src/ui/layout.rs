use ratatui::layout::{Constraint, Layout, Rect};

use crate::browser::BrowseLevel;

#[cfg(test)]
pub const STANDARD_SIZE: (u16, u16) = (100, 30);
pub const MINIMUM_SIZE: (u16, u16) = (60, 16);
#[cfg(test)]
pub const WIDE_SIZE: (u16, u16) = (140, 40);
const SPLIT_MINIMUM_WIDTH: u16 = 100;
const AUXILIARY_WIDTH: u16 = 40;
const CODE_POINT_GRID_WIDTH: u16 = 60;
const CODE_POINT_CONTEXT_MINIMUM_WIDTH: u16 = 40;
pub const GLYPH_MAXIMUM_WIDTH: u16 = 36;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiLayout {
    pub header: Rect,
    pub main: Rect,
    pub footer: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrowserLayout {
    pub navigator: Rect,
    pub context: Option<Rect>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchLayout {
    pub input: Rect,
    pub results: Rect,
    pub preview: Option<Rect>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InspectorLayout {
    pub details: Rect,
    pub preview: Option<GlyphPreviewLayout>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlyphPreviewLayout {
    pub panel: Rect,
    pub placeholder: Rect,
}

pub fn calculate(area: Rect) -> Option<UiLayout> {
    let (minimum_width, minimum_height) = MINIMUM_SIZE;
    if area.width < minimum_width || area.height < minimum_height {
        return None;
    }

    let [header, main, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .areas(area);

    Some(UiLayout {
        header,
        main,
        footer,
    })
}

pub fn browser(area: Rect, level: BrowseLevel) -> BrowserLayout {
    if level == BrowseLevel::CodePointTable {
        code_point_grid_and_context(area)
    } else {
        flexible_primary_and_auxiliary(area)
    }
}

pub fn inspector(area: Rect) -> InspectorLayout {
    let panes = flexible_primary_and_auxiliary(area);
    InspectorLayout {
        details: panes.navigator,
        preview: panes.context.map(|panel| GlyphPreviewLayout {
            panel,
            placeholder: centered_glyph_area(Rect::new(
                panel.x.saturating_add(2),
                panel.y.saturating_add(3),
                panel.width.saturating_sub(3),
                panel.height.saturating_sub(4),
            )),
        }),
    }
}

pub fn search(area: Rect) -> SearchLayout {
    let panes = flexible_primary_and_auxiliary(area);
    let [input, results] =
        Layout::vertical([Constraint::Length(2), Constraint::Min(1)]).areas(panes.navigator);

    SearchLayout {
        input,
        results,
        preview: panes.context,
    }
}

pub fn sequence(area: Rect) -> BrowserLayout {
    flexible_primary_and_auxiliary(area)
}

fn flexible_primary_and_auxiliary(area: Rect) -> BrowserLayout {
    if area.width < SPLIT_MINIMUM_WIDTH {
        return BrowserLayout {
            navigator: area,
            context: None,
        };
    }

    let [navigator, context] = Layout::horizontal([
        Constraint::Min(SPLIT_MINIMUM_WIDTH - AUXILIARY_WIDTH),
        Constraint::Length(AUXILIARY_WIDTH),
    ])
    .areas(area);
    BrowserLayout {
        navigator,
        context: Some(context),
    }
}

fn code_point_grid_and_context(area: Rect) -> BrowserLayout {
    if area.width < CODE_POINT_GRID_WIDTH + CODE_POINT_CONTEXT_MINIMUM_WIDTH {
        return BrowserLayout {
            navigator: area,
            context: None,
        };
    }

    let [navigator, context] = Layout::horizontal([
        Constraint::Length(CODE_POINT_GRID_WIDTH),
        Constraint::Min(CODE_POINT_CONTEXT_MINIMUM_WIDTH),
    ])
    .areas(area);
    BrowserLayout {
        navigator,
        context: Some(context),
    }
}

pub fn centered_glyph_area(area: Rect) -> Rect {
    let width = area.width.min(GLYPH_MAXIMUM_WIDTH);
    Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y,
        width,
        area.height,
    )
}

pub fn browser_list_height(area: Rect) -> usize {
    calculate(area).map_or(0, |layout| {
        usize::from(layout.main.height.saturating_sub(2))
    })
}

pub fn search_result_height(area: Rect) -> usize {
    calculate(area).map_or(0, |layout| {
        usize::from(search(layout.main).results.height.saturating_sub(2))
    })
}

pub fn sequence_list_height(area: Rect) -> usize {
    calculate(area).map_or(0, |layout| {
        usize::from(sequence(layout.main).navigator.height.saturating_sub(2))
    })
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(
        STANDARD_SIZE,
        UiLayout {
            header: Rect::new(0, 0, 100, 1),
            main: Rect::new(0, 1, 100, 28),
            footer: Rect::new(0, 29, 100, 1),
        }
    )]
    #[case(
        MINIMUM_SIZE,
        UiLayout {
            header: Rect::new(0, 0, 60, 1),
            main: Rect::new(0, 1, 60, 14),
            footer: Rect::new(0, 15, 60, 1),
        }
    )]
    #[case(
        WIDE_SIZE,
        UiLayout {
            header: Rect::new(0, 0, 140, 1),
            main: Rect::new(0, 1, 140, 38),
            footer: Rect::new(0, 39, 140, 1),
        }
    )]
    fn calculates_supported_shell_layouts(#[case] size: (u16, u16), #[case] expected: UiLayout) {
        let (width, height) = size;

        assert_eq!(calculate(Rect::new(0, 0, width, height)), Some(expected));
    }

    #[test]
    fn rejects_width_below_minimum() {
        assert_eq!(calculate(Rect::new(0, 0, 59, 16)), None);
    }

    #[test]
    fn rejects_height_below_minimum() {
        assert_eq!(calculate(Rect::new(0, 0, 60, 15)), None);
    }

    #[rstest]
    #[case(
        Rect::new(0, 3, 60, 12),
        BrowserLayout {
            navigator: Rect::new(0, 3, 60, 12),
            context: None,
        }
    )]
    #[case(
        Rect::new(0, 3, 99, 12),
        BrowserLayout {
            navigator: Rect::new(0, 3, 99, 12),
            context: None,
        }
    )]
    #[case(
        Rect::new(0, 3, 100, 26),
        BrowserLayout {
            navigator: Rect::new(0, 3, 60, 26),
            context: Some(Rect::new(60, 3, 40, 26)),
        }
    )]
    #[case(
        Rect::new(0, 3, 140, 36),
        BrowserLayout {
            navigator: Rect::new(0, 3, 100, 36),
            context: Some(Rect::new(100, 3, 40, 36)),
        }
    )]
    fn lays_out_the_browser_at_responsive_width_boundaries(
        #[case] area: Rect,
        #[case] expected: BrowserLayout,
    ) {
        assert_eq!(browser(area, BrowseLevel::Plane), expected);
    }

    #[test]
    fn keeps_the_code_point_grid_fixed_and_gives_the_preview_remaining_width() {
        assert_eq!(
            browser(Rect::new(0, 3, 100, 26), BrowseLevel::CodePointTable),
            BrowserLayout {
                navigator: Rect::new(0, 3, 60, 26),
                context: Some(Rect::new(60, 3, 40, 26)),
            }
        );
        assert_eq!(
            browser(Rect::new(0, 3, 140, 36), BrowseLevel::CodePointTable),
            BrowserLayout {
                navigator: Rect::new(0, 3, 60, 36),
                context: Some(Rect::new(60, 3, 80, 36)),
            }
        );
    }

    #[rstest]
    #[case(
        Rect::new(0, 3, 60, 12),
        InspectorLayout {
            details: Rect::new(0, 3, 60, 12),
            preview: None,
        }
    )]
    #[case(
        Rect::new(0, 3, 99, 12),
        InspectorLayout {
            details: Rect::new(0, 3, 99, 12),
            preview: None,
        }
    )]
    #[case(
        Rect::new(0, 3, 100, 26),
        InspectorLayout {
            details: Rect::new(0, 3, 60, 26),
            preview: Some(GlyphPreviewLayout {
                panel: Rect::new(60, 3, 40, 26),
                placeholder: Rect::new(62, 6, 36, 22),
            }),
        }
    )]
    #[case(
        Rect::new(0, 3, 140, 36),
        InspectorLayout {
            details: Rect::new(0, 3, 100, 36),
            preview: Some(GlyphPreviewLayout {
                panel: Rect::new(100, 3, 40, 36),
                placeholder: Rect::new(102, 6, 36, 32),
            }),
        }
    )]
    fn lays_out_the_inspector_at_preview_boundaries(
        #[case] area: Rect,
        #[case] expected: InspectorLayout,
    ) {
        assert_eq!(inspector(area), expected);
    }

    #[rstest]
    #[case(
        Rect::new(0, 3, 60, 12),
        SearchLayout {
            input: Rect::new(0, 3, 60, 2),
            results: Rect::new(0, 5, 60, 10),
            preview: None,
        }
    )]
    #[case(
        Rect::new(0, 3, 99, 12),
        SearchLayout {
            input: Rect::new(0, 3, 99, 2),
            results: Rect::new(0, 5, 99, 10),
            preview: None,
        }
    )]
    #[case(
        Rect::new(0, 3, 100, 26),
        SearchLayout {
            input: Rect::new(0, 3, 60, 2),
            results: Rect::new(0, 5, 60, 24),
            preview: Some(Rect::new(60, 3, 40, 26)),
        }
    )]
    #[case(
        Rect::new(0, 3, 140, 36),
        SearchLayout {
            input: Rect::new(0, 3, 100, 2),
            results: Rect::new(0, 5, 100, 34),
            preview: Some(Rect::new(100, 3, 40, 36)),
        }
    )]
    fn lays_out_search_at_responsive_width_boundaries(
        #[case] area: Rect,
        #[case] expected: SearchLayout,
    ) {
        assert_eq!(search(area), expected);
    }

    #[test]
    fn calculates_browser_list_height_from_the_terminal_area() {
        assert_eq!(browser_list_height(Rect::new(0, 0, 100, 30)), 26);
        assert_eq!(browser_list_height(Rect::new(0, 0, 60, 16)), 12);
        assert_eq!(browser_list_height(Rect::new(0, 0, 140, 40)), 36);
        assert_eq!(browser_list_height(Rect::new(0, 0, 59, 15)), 0);
    }

    #[test]
    fn calculates_search_result_height_from_the_terminal_area() {
        assert_eq!(search_result_height(Rect::new(0, 0, 100, 30)), 24);
        assert_eq!(search_result_height(Rect::new(0, 0, 60, 16)), 10);
        assert_eq!(search_result_height(Rect::new(0, 0, 140, 40)), 34);
        assert_eq!(search_result_height(Rect::new(0, 0, 59, 15)), 0);
    }
}
