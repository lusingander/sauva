use jiff::tz::TimeZone;
use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::{
    app::AppState,
    ui::{
        key_value::{self, KeyValue},
        layout,
        scrollbar::{self, ViewportScrollbar},
        selectable_list_line, selection_preview,
        settings::{UiSettings, format_bookmark_date},
        theme::ColorTheme,
        workspace,
    },
    unicode::{CodePoint, UnicodeDatabase},
};

pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    color_theme: &ColorTheme,
    ui: &UiSettings,
) {
    let layout = layout::bookmarks(area);
    let bookmarks = state.bookmarks();
    let content = workspace::render_primary_heading(
        frame,
        layout.list,
        "Bookmarks",
        Some("Newest first"),
        color_theme,
    );
    let visible = bookmarks.visible_range();
    let lines = if bookmarks.entries().is_empty() {
        vec![
            Line::from("No bookmarks yet"),
            Line::styled(
                "Save code points from the Inspector",
                Style::new().fg(color_theme.muted),
            ),
        ]
    } else {
        visible
            .clone()
            .map(|index| {
                selectable_list_line(
                    bookmark_line(bookmarks.entries()[index].code_point()),
                    index == bookmarks.selected_index(),
                    content.width,
                    color_theme.selection,
                )
            })
            .collect()
    };
    frame.render_widget(Paragraph::new(lines), content);
    frame.render_widget(
        ViewportScrollbar::new(bookmarks.entries().len(), visible)
            .style(color_theme.border_style()),
        scrollbar::area_for_primary(frame.area(), layout.list, content),
    );
    if let Some(preview) = layout.preview {
        if let Some(bookmark) = bookmarks.selected() {
            selection_preview::render_bookmark(
                frame,
                preview,
                bookmark,
                state.glyph_preview(),
                ui,
                color_theme,
            );
        } else {
            let content =
                workspace::render_rail_heading(frame, preview, "Selection", None, color_theme);
            frame.render_widget(
                Paragraph::new("No bookmark selected").style(Style::new().fg(color_theme.muted)),
                content,
            );
        }
    }
    if let Some(summary) = layout.summary {
        workspace::render_divider(
            frame,
            Rect::new(summary.x + 1, summary.y, summary.width.saturating_sub(3), 1),
            color_theme,
        );
        if let Some(bookmark) = bookmarks.selected() {
            let formatted = format_bookmark_date(
                bookmark.added_at(),
                &ui.bookmark_date_format,
                TimeZone::system(),
            )
            .unwrap_or_else(|_| bookmark.added_at().to_string());
            let entries = [
                KeyValue::new("Added at", &formatted),
                KeyValue::new(
                    "Name",
                    UnicodeDatabase::primary_name_or_fallback(bookmark.code_point()),
                ),
            ];
            let area = workspace::primary_canvas(Rect::new(
                summary.x,
                summary.y + 1,
                summary.width,
                summary.height.saturating_sub(1),
            ));
            key_value::render_entries(frame, area, 10, &entries, color_theme);
        }
    }
}

fn bookmark_line(code_point: CodePoint) -> Line<'static> {
    let representation = UnicodeDatabase::display_representation(code_point);
    let text = representation.as_str();
    let width = Line::from(text).width();
    let mut spans = if matches!(width, 1 | 2) {
        vec![Span::raw(text.to_owned()), Span::raw(" ".repeat(4 - width))]
    } else {
        vec![Span::raw("·   ")]
    };
    spans.push(Span::raw(format!("{:<8}  ", code_point.to_string())));
    if !matches!(width, 1 | 2) {
        spans.push(Span::raw(format!("{text} — ")));
    }
    spans.push(Span::raw(UnicodeDatabase::primary_name_or_fallback(
        code_point,
    )));
    Line::from(spans)
}
