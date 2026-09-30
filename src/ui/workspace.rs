use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::Style,
    text::Line,
    widgets::{Block, Borders, Paragraph},
};

use crate::ui::theme::ColorTheme;

const PRIMARY_HORIZONTAL_INSET: u16 = 1;
const PRIMARY_RIGHT_INSET: u16 = 2;
const RAIL_LEFT_INSET: u16 = 2;
const RAIL_RIGHT_INSET: u16 = 1;
const BOTTOM_INSET: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SectionLayout {
    pub heading: Rect,
    pub content: Rect,
}

pub fn primary_canvas(area: Rect) -> Rect {
    inset(
        area,
        PRIMARY_HORIZONTAL_INSET,
        PRIMARY_RIGHT_INSET,
        BOTTOM_INSET,
    )
}

pub fn rail_canvas(area: Rect) -> Rect {
    inset(area, RAIL_LEFT_INSET, RAIL_RIGHT_INSET, BOTTOM_INSET)
}

pub fn primary_section(area: Rect) -> SectionLayout {
    section(primary_canvas(area))
}

pub fn rail_section(area: Rect) -> SectionLayout {
    section(rail_canvas(area))
}

pub fn render_primary_heading(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    secondary: Option<&str>,
    color_theme: &ColorTheme,
) -> Rect {
    let section = primary_section(area);
    render_heading(frame, section.heading, title, secondary, color_theme);
    section.content
}

pub fn render_rail_heading(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    secondary: Option<&str>,
    color_theme: &ColorTheme,
) -> Rect {
    frame.render_widget(
        Block::default()
            .borders(Borders::LEFT)
            .border_style(color_theme.border_style()),
        area,
    );
    let section = rail_section(area);
    render_heading(frame, section.heading, title, secondary, color_theme);
    section.content
}

pub fn render_divider(frame: &mut Frame, area: Rect, color_theme: &ColorTheme) {
    frame.render_widget(
        Block::default()
            .borders(Borders::TOP)
            .border_style(color_theme.border_style()),
        area,
    );
}

fn render_heading(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    secondary: Option<&str>,
    color_theme: &ColorTheme,
) {
    frame.render_widget(
        Paragraph::new(Line::styled(title.to_owned(), color_theme.heading_style())),
        area,
    );
    if let Some(secondary) = secondary {
        frame.render_widget(
            Paragraph::new(Line::styled(
                secondary.to_owned(),
                Style::new().fg(color_theme.muted),
            ))
            .alignment(Alignment::Right),
            area,
        );
    }
}

fn section(area: Rect) -> SectionLayout {
    SectionLayout {
        heading: Rect::new(area.x, area.y, area.width, area.height.min(1)),
        content: Rect::new(
            area.x,
            area.y.saturating_add(1),
            area.width,
            area.height.saturating_sub(1),
        ),
    }
}

fn inset(area: Rect, left: u16, right: u16, bottom: u16) -> Rect {
    Rect::new(
        area.x.saturating_add(left.min(area.width)),
        area.y,
        area.width.saturating_sub(left.saturating_add(right)),
        area.height.saturating_sub(bottom),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primary_section_uses_whitespace_instead_of_a_box() {
        assert_eq!(
            primary_section(Rect::new(0, 1, 60, 28)),
            SectionLayout {
                heading: Rect::new(1, 1, 57, 1),
                content: Rect::new(1, 2, 57, 26),
            }
        );
    }

    #[test]
    fn rail_section_reserves_a_divider_and_open_padding() {
        assert_eq!(
            rail_section(Rect::new(60, 1, 40, 28)),
            SectionLayout {
                heading: Rect::new(62, 1, 37, 1),
                content: Rect::new(62, 2, 37, 26),
            }
        );
    }
}
