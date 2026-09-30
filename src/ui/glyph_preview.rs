use ratatui::{
    Frame,
    buffer::CellDiffOption,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::{
    graphics::{GraphicsProtocol, GraphicsUnavailableReason},
    image::kitty,
    preview::{GlyphPreviewError, GlyphPreviewState, GlyphPreviewStatus},
    ui::{layout::GlyphPreviewLayout, theme::ColorTheme, workspace},
};

pub fn render(
    frame: &mut Frame,
    layout: GlyphPreviewLayout,
    state: &GlyphPreviewState,
    color_theme: &ColorTheme,
) {
    let content = workspace::render_rail_heading(frame, layout.panel, "Glyph", None, color_theme);

    render_metadata(
        frame,
        Rect::new(content.x, content.y, content.width, 2),
        state,
        color_theme,
    );
    if renders_image(state.status()) {
        render_image(frame, layout.placeholder, state, color_theme);
    } else {
        render_message(frame, layout.placeholder, state.status(), color_theme);
    }
}

pub fn render_image_only(
    frame: &mut Frame,
    area: Rect,
    state: &GlyphPreviewState,
    color_theme: &ColorTheme,
) {
    if !renders_image(state.status())
        || state.geometry().is_none_or(|geometry| {
            geometry.columns() != area.width || geometry.rows() != area.height
        })
    {
        return;
    }

    match state.protocol() {
        Some(GraphicsProtocol::Kitty) if state.image_id().is_some() => {
            render_kitty_placeholder(frame, area, state, color_theme);
        }
        Some(GraphicsProtocol::Iterm2) => reserve_iterm2_area(frame, area, state, color_theme),
        Some(GraphicsProtocol::Kitty) | None => {}
    }
}

fn render_metadata(
    frame: &mut Frame,
    area: Rect,
    state: &GlyphPreviewState,
    color_theme: &ColorTheme,
) {
    let label = Style::new().fg(color_theme.muted);
    let font = state
        .font()
        .map(|font| format!("{} {}", font.family(), font.style()))
        .unwrap_or_else(|| "—".to_owned());
    let version = state
        .font()
        .and_then(crate::glyph::font::FontInfo::version)
        .unwrap_or("—");
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(vec![Span::styled("Font     ", label), Span::raw(font)]),
            Line::from(vec![Span::styled("Version  ", label), Span::raw(version)]),
        ]),
        area,
    );
}

fn renders_image(status: GlyphPreviewStatus) -> bool {
    matches!(
        status,
        GlyphPreviewStatus::Ready | GlyphPreviewStatus::CombiningContext
    )
}

fn render_image(
    frame: &mut Frame,
    area: Rect,
    state: &GlyphPreviewState,
    color_theme: &ColorTheme,
) {
    match state.protocol() {
        Some(GraphicsProtocol::Kitty) => render_kitty_placeholder(frame, area, state, color_theme),
        Some(GraphicsProtocol::Iterm2) => reserve_iterm2_area(frame, area, state, color_theme),
        None => render_message(frame, area, GlyphPreviewStatus::Pending, color_theme),
    }
}

fn render_kitty_placeholder(
    frame: &mut Frame,
    area: Rect,
    state: &GlyphPreviewState,
    color_theme: &ColorTheme,
) {
    let Some(image_id) = state.image_id() else {
        render_message(frame, area, GlyphPreviewStatus::Pending, color_theme);
        return;
    };
    let Some(geometry) = state.geometry() else {
        render_message(frame, area, GlyphPreviewStatus::Pending, color_theme);
        return;
    };
    if geometry.columns() != area.width || geometry.rows() != area.height {
        render_message(frame, area, GlyphPreviewStatus::Pending, color_theme);
        return;
    }

    let style = Style::new().fg(kitty::image_id_color(image_id));
    let rows = (0..area.height)
        .map(|row| {
            Line::from(
                (0..area.width)
                    .map(|column| {
                        Span::styled(
                            kitty::placeholder(row, column)
                                .expect("validated preview layout fits the placeholder table"),
                            style,
                        )
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();
    frame.render_widget(Paragraph::new(rows), area);
}

fn reserve_iterm2_area(
    frame: &mut Frame,
    area: Rect,
    state: &GlyphPreviewState,
    color_theme: &ColorTheme,
) {
    let Some(geometry) = state.geometry() else {
        render_message(frame, area, GlyphPreviewStatus::Pending, color_theme);
        return;
    };
    if geometry.columns() != area.width || geometry.rows() != area.height {
        render_message(frame, area, GlyphPreviewStatus::Pending, color_theme);
        return;
    }

    let buffer = frame.buffer_mut();
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if let Some(cell) = buffer.cell_mut((x, y)) {
                cell.set_diff_option(CellDiffOption::Skip);
            }
        }
    }
}

fn render_message(
    frame: &mut Frame,
    area: Rect,
    status: GlyphPreviewStatus,
    color_theme: &ColorTheme,
) {
    let (heading, explanation) = message(status);
    let height = u16::from(explanation.is_some()) + 1;
    let area = Rect::new(
        area.x,
        area.y + area.height.saturating_sub(height) / 2,
        area.width,
        height.min(area.height),
    );
    let mut lines = vec![Line::from(Span::styled(
        heading,
        Style::new()
            .fg(color_theme.status.warning)
            .add_modifier(Modifier::BOLD),
    ))];
    if let Some(explanation) = explanation {
        lines.push(Line::from(Span::styled(
            explanation,
            Style::new().fg(color_theme.muted),
        )));
    }
    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), area);
}

#[rustfmt::skip]
fn message(status: GlyphPreviewStatus) -> (&'static str, Option<&'static str>) {
    match status {
        GlyphPreviewStatus::Detecting => ("Detecting graphics support", None),
        GlyphPreviewStatus::Hidden => ("Glyph preview hidden", None),
        GlyphPreviewStatus::Pending => ("Preparing glyph preview", None),
        GlyphPreviewStatus::Ready => ("Preparing glyph preview", None),
        GlyphPreviewStatus::CombiningContext => ("Preparing glyph preview", None),
        GlyphPreviewStatus::Blank => ("No visible pixels", Some("The resolved glyph is blank.")),
        GlyphPreviewStatus::Missing => ("Glyph unavailable", Some("No configured font contains it.")),
        GlyphPreviewStatus::NotScalar => ("Not a Unicode scalar value", Some("Surrogates cannot be rendered.")),
        GlyphPreviewStatus::GraphicsUnavailable(GraphicsUnavailableReason::Disabled) => ("Graphics disabled", Some("Enable with --graphics auto/force.")),
        GlyphPreviewStatus::GraphicsUnavailable(GraphicsUnavailableReason::UnsupportedTerminal) => ("Graphics unavailable", Some("Use --graphics force to try anyway.")),
        GlyphPreviewStatus::Error(GlyphPreviewError::InvalidGeometry) => ("Preview area unavailable", Some("Resize the terminal and try again.")),
        GlyphPreviewStatus::Error(GlyphPreviewError::Rendering) => ("Glyph rendering failed", Some("Text details remain available.")),
        GlyphPreviewStatus::Error(GlyphPreviewError::Transmission) => ("Image transmission failed", Some("Text details remain available.")),
    }
}

#[cfg(test)]
mod tests {
    use ratatui::{
        Terminal,
        backend::TestBackend,
        buffer::CellDiffOption,
        layout::Rect,
        style::{Color, Modifier},
        text::Line,
    };

    use crate::{
        fixtures,
        graphics::GraphicsUnavailableReason,
        image::kitty,
        preview::{GlyphPreviewError, GlyphPreviewStatus},
        ui::{glyph_preview, layout::GlyphPreviewLayout, theme::ColorTheme},
    };

    #[test]
    fn ready_preview_fills_the_exact_placeholder_with_the_image_id_color() {
        let fixture = fixtures::glyph_basic();
        let state = fixture.glyph_preview();
        let layout = GlyphPreviewLayout {
            panel: Rect::new(0, 0, 40, 28),
            placeholder: Rect::new(2, 3, 36, 24),
        };
        let backend = TestBackend::new(40, 28);
        let mut terminal = Terminal::new(backend).unwrap();

        let completed = terminal
            .draw(|frame| {
                glyph_preview::render(frame, layout, state, &ColorTheme::default());
            })
            .unwrap();

        let buffer = completed.buffer;
        let image_id = state.image_id().unwrap();
        assert_eq!(
            buffer.cell((2, 3)).unwrap().symbol(),
            kitty::placeholder(0, 0).unwrap()
        );
        assert_eq!(
            buffer.cell((37, 26)).unwrap().symbol(),
            kitty::placeholder(23, 35).unwrap()
        );
        assert_eq!(
            buffer.cell((2, 3)).unwrap().fg,
            kitty::image_id_color(image_id)
        );
    }

    #[test]
    fn image_only_preview_draws_no_metadata_or_status() {
        let fixture = fixtures::glyph_basic();
        let state = fixture.glyph_preview();
        let area = Rect::new(0, 0, 36, 24);
        let backend = TestBackend::new(36, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                glyph_preview::render_image_only(frame, area, state, &ColorTheme::default());
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        assert_eq!(
            buffer.cell((0, 0)).unwrap().symbol(),
            kitty::placeholder(0, 0).unwrap()
        );
        assert_eq!(
            buffer.cell((35, 23)).unwrap().symbol(),
            kitty::placeholder(23, 35).unwrap()
        );
        assert!(!buffer.content.iter().any(|cell| cell.symbol() == "F"));
    }

    #[test]
    fn image_only_preview_leaves_non_image_statuses_blank() {
        let fixture = fixtures::glyph_missing();
        let state = fixture.glyph_preview();
        let area = Rect::new(0, 0, 36, 22);
        let backend = TestBackend::new(36, 22);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                glyph_preview::render_image_only(frame, area, state, &ColorTheme::default());
            })
            .unwrap();

        assert!(
            terminal
                .backend()
                .buffer()
                .content
                .iter()
                .all(|cell| cell.symbol() == " ")
        );
    }

    #[test]
    fn iterm2_preview_reserves_the_exact_image_area_without_placeholders() {
        let fixture = fixtures::glyph_basic_iterm2();
        let state = fixture.glyph_preview();
        let layout = GlyphPreviewLayout {
            panel: Rect::new(0, 0, 40, 28),
            placeholder: Rect::new(2, 3, 36, 24),
        };
        let backend = TestBackend::new(40, 28);
        let mut terminal = Terminal::new(backend).unwrap();

        let completed = terminal
            .draw(|frame| {
                glyph_preview::render(frame, layout, state, &ColorTheme::default());
            })
            .unwrap();

        let buffer = completed.buffer;
        for y in 0..28 {
            for x in 0..40 {
                let expected = if layout.placeholder.contains((x, y).into()) {
                    CellDiffOption::Skip
                } else {
                    CellDiffOption::None
                };
                assert_eq!(buffer.cell((x, y)).unwrap().diff_option, expected);
            }
        }
        assert!(
            !buffer
                .content
                .iter()
                .any(|cell| cell.symbol().contains('\u{10eeee}'))
        );
    }

    #[test]
    fn non_image_message_does_not_render_a_kitty_placeholder() {
        let fixture = fixtures::glyph_missing();
        let state = fixture.glyph_preview();
        let layout = GlyphPreviewLayout {
            panel: Rect::new(0, 0, 40, 26),
            placeholder: Rect::new(2, 3, 36, 22),
        };
        let backend = TestBackend::new(40, 26);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                glyph_preview::render(frame, layout, state, &ColorTheme::default());
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        assert!(
            !buffer
                .content
                .iter()
                .any(|cell| cell.symbol().contains('\u{10eeee}'))
        );
        assert_eq!(buffer.cell((3, 1)).unwrap().fg, Color::DarkGray);
        let heading = (3..25)
            .flat_map(|y| (0..40).filter_map(move |x| buffer.cell((x, y))))
            .find(|cell| cell.symbol() == "G")
            .expect("the centered message heading must be visible");
        assert_eq!(heading.fg, Color::Yellow);
        assert!(heading.modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn defines_every_status_message_within_the_minimum_preview_width() {
        let messages = [
            (
                GlyphPreviewStatus::Detecting,
                "Detecting graphics support",
                None,
            ),
            (GlyphPreviewStatus::Hidden, "Glyph preview hidden", None),
            (GlyphPreviewStatus::Pending, "Preparing glyph preview", None),
            (GlyphPreviewStatus::Ready, "Preparing glyph preview", None),
            (
                GlyphPreviewStatus::CombiningContext,
                "Preparing glyph preview",
                None,
            ),
            (
                GlyphPreviewStatus::Blank,
                "No visible pixels",
                Some("The resolved glyph is blank."),
            ),
            (
                GlyphPreviewStatus::Missing,
                "Glyph unavailable",
                Some("No configured font contains it."),
            ),
            (
                GlyphPreviewStatus::NotScalar,
                "Not a Unicode scalar value",
                Some("Surrogates cannot be rendered."),
            ),
            (
                GlyphPreviewStatus::GraphicsUnavailable(GraphicsUnavailableReason::Disabled),
                "Graphics disabled",
                Some("Enable with --graphics auto/force."),
            ),
            (
                GlyphPreviewStatus::GraphicsUnavailable(
                    GraphicsUnavailableReason::UnsupportedTerminal,
                ),
                "Graphics unavailable",
                Some("Use --graphics force to try anyway."),
            ),
            (
                GlyphPreviewStatus::Error(GlyphPreviewError::InvalidGeometry),
                "Preview area unavailable",
                Some("Resize the terminal and try again."),
            ),
            (
                GlyphPreviewStatus::Error(GlyphPreviewError::Rendering),
                "Glyph rendering failed",
                Some("Text details remain available."),
            ),
            (
                GlyphPreviewStatus::Error(GlyphPreviewError::Transmission),
                "Image transmission failed",
                Some("Text details remain available."),
            ),
        ];

        for (status, expected_heading, expected_explanation) in messages {
            let (heading, explanation) = glyph_preview::message(status);
            assert_eq!(heading, expected_heading);
            assert_eq!(explanation, expected_explanation);
            assert!(Line::from(heading).width() <= 36, "heading: {heading}");
            if let Some(explanation) = explanation {
                assert!(
                    Line::from(explanation).width() <= 36,
                    "explanation: {explanation}"
                );
            }
        }
    }
}
