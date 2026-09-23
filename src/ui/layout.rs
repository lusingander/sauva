use ratatui::layout::{Constraint, Layout, Rect};

#[cfg(test)]
pub const STANDARD_SIZE: (u16, u16) = (100, 30);
pub const MINIMUM_SIZE: (u16, u16) = (60, 16);
#[cfg(test)]
pub const WIDE_SIZE: (u16, u16) = (140, 40);
const NAVIGATOR_WIDTH: u16 = 60;
const CONTEXT_MINIMUM_WIDTH: u16 = 40;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiLayout {
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

    let [main, footer] = Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).areas(area);

    Some(UiLayout { main, footer })
}

pub fn browser(area: Rect) -> BrowserLayout {
    navigator_and_context(area)
}

pub fn inspector(area: Rect) -> InspectorLayout {
    let panes = navigator_and_context(area);
    InspectorLayout {
        details: panes.navigator,
        preview: panes.context.map(|panel| GlyphPreviewLayout {
            panel,
            placeholder: Rect::new(
                panel.x.saturating_add(2),
                panel.y.saturating_add(3),
                panel.width.saturating_sub(4),
                panel.height.saturating_sub(4),
            ),
        }),
    }
}

pub fn search(area: Rect) -> SearchLayout {
    let panes = navigator_and_context(area);
    let [input, results] =
        Layout::vertical([Constraint::Length(3), Constraint::Min(1)]).areas(panes.navigator);

    SearchLayout {
        input,
        results,
        preview: panes.context,
    }
}

pub fn sequence(area: Rect) -> BrowserLayout {
    navigator_and_context(area)
}

fn navigator_and_context(area: Rect) -> BrowserLayout {
    if area.width < NAVIGATOR_WIDTH + CONTEXT_MINIMUM_WIDTH {
        return BrowserLayout {
            navigator: area,
            context: None,
        };
    }

    let [navigator, context] = Layout::horizontal([
        Constraint::Length(NAVIGATOR_WIDTH),
        Constraint::Min(CONTEXT_MINIMUM_WIDTH),
    ])
    .areas(area);
    BrowserLayout {
        navigator,
        context: Some(context),
    }
}

pub fn browser_list_height(area: Rect) -> usize {
    calculate(area).map_or(0, |layout| {
        usize::from(browser(layout.main).navigator.height.saturating_sub(2))
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
            main: Rect::new(0, 0, 100, 29),
            footer: Rect::new(0, 29, 100, 1),
        }
    )]
    #[case(
        MINIMUM_SIZE,
        UiLayout {
            main: Rect::new(0, 0, 60, 15),
            footer: Rect::new(0, 15, 60, 1),
        }
    )]
    #[case(
        WIDE_SIZE,
        UiLayout {
            main: Rect::new(0, 0, 140, 39),
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
            navigator: Rect::new(0, 3, 60, 36),
            context: Some(Rect::new(60, 3, 80, 36)),
        }
    )]
    fn lays_out_the_browser_at_responsive_width_boundaries(
        #[case] area: Rect,
        #[case] expected: BrowserLayout,
    ) {
        assert_eq!(browser(area), expected);
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
            details: Rect::new(0, 3, 60, 36),
            preview: Some(GlyphPreviewLayout {
                panel: Rect::new(60, 3, 80, 36),
                placeholder: Rect::new(62, 6, 76, 32),
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
            input: Rect::new(0, 3, 60, 3),
            results: Rect::new(0, 6, 60, 9),
            preview: None,
        }
    )]
    #[case(
        Rect::new(0, 3, 99, 12),
        SearchLayout {
            input: Rect::new(0, 3, 99, 3),
            results: Rect::new(0, 6, 99, 9),
            preview: None,
        }
    )]
    #[case(
        Rect::new(0, 3, 100, 26),
        SearchLayout {
            input: Rect::new(0, 3, 60, 3),
            results: Rect::new(0, 6, 60, 23),
            preview: Some(Rect::new(60, 3, 40, 26)),
        }
    )]
    #[case(
        Rect::new(0, 3, 140, 36),
        SearchLayout {
            input: Rect::new(0, 3, 60, 3),
            results: Rect::new(0, 6, 60, 33),
            preview: Some(Rect::new(60, 3, 80, 36)),
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
        assert_eq!(browser_list_height(Rect::new(0, 0, 100, 30)), 27);
        assert_eq!(browser_list_height(Rect::new(0, 0, 60, 16)), 13);
        assert_eq!(browser_list_height(Rect::new(0, 0, 140, 40)), 37);
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
