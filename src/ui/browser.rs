use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::{
    app::AppState,
    browser::{BrowseLevel, BrowseState},
    ui::{
        key_value::{self, KeyValue},
        layout::browser,
        scrollbar::{self, ViewportScrollbar},
        selectable_list_line, selection_preview,
        settings::UiSettings,
        theme::ColorTheme,
        workspace,
    },
    unicode::{CodePoint, Plane, UnicodeDatabase, plane::PlaneRange},
};

pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    color_theme: &ColorTheme,
    ui: &UiSettings,
) {
    let browse = state
        .browse()
        .expect("the browser view always has browse state");
    let layout = browser(area, browse.level());

    match browse.level() {
        BrowseLevel::Plane => {
            if let Some(context) = layout.context {
                render_plane_context(frame, context, browse.cursor(), color_theme);
            }
            render_plane_navigator(
                frame,
                layout.navigator,
                browse.cursor(),
                browse
                    .visible_list_items()
                    .expect("the plane level has a list viewport"),
                color_theme,
                ui,
            );
        }
        BrowseLevel::Range => {
            if let Some(context) = layout.context {
                render_range_context(frame, context, browse.cursor(), color_theme);
            }
            render_range_navigator(
                frame,
                layout.navigator,
                browse.cursor(),
                browse
                    .visible_list_items()
                    .expect("the range level has a list viewport"),
                color_theme,
                ui,
            );
        }
        BrowseLevel::Block => {
            if let Some(context) = layout.context {
                render_block_context(frame, context, browse, color_theme);
            }
            render_block_navigator(
                frame,
                layout.navigator,
                browse,
                browse
                    .visible_list_items()
                    .expect("the block level has a list viewport"),
                color_theme,
                ui,
            );
        }
        BrowseLevel::CodePointTable => {
            if let Some(context) = layout.context {
                selection_preview::render(
                    frame,
                    context,
                    browse.cursor(),
                    state.glyph_preview(),
                    color_theme,
                );
            }
            render_code_point_table(
                frame,
                layout.navigator,
                browse,
                browse
                    .visible_table_rows()
                    .expect("the code point table level has a row viewport"),
                color_theme,
                ui,
            );
        }
    }
}

fn render_block_navigator(
    frame: &mut Frame,
    area: Rect,
    browse: &BrowseState,
    visible_items: std::ops::Range<usize>,
    color_theme: &ColorTheme,
    ui: &UiSettings,
) {
    let selected = browse
        .selected_block()
        .expect("the block level has a selected block");
    let content = workspace::render_primary_heading(frame, area, "Blocks", None, color_theme);
    let rows = visible_items
        .clone()
        .map(|index| {
            let item = UnicodeDatabase::block(index).expect("the viewport contains valid blocks");
            let is_selected = item == selected;
            let marker = ui.selection_marker(is_selected);
            selectable_list_line(
                Line::from(format!(
                    "{marker} {:06X}–{:06X}  {}",
                    item.start().value(),
                    item.end().value(),
                    item.name()
                )),
                is_selected,
                content.width,
                color_theme.selection,
            )
        })
        .collect::<Vec<_>>();

    frame.render_widget(Paragraph::new(rows), content);
    frame.render_widget(
        ViewportScrollbar::new(UnicodeDatabase::blocks().len(), visible_items)
            .style(color_theme.border_style()),
        scrollbar::area_for_primary(frame.area(), area, content),
    );
}

fn render_block_context(
    frame: &mut Frame,
    area: Rect,
    browse: &BrowseState,
    color_theme: &ColorTheme,
) {
    let block = browse
        .selected_block()
        .expect("the block level has a selected block");
    let start = block.start().to_string();
    let end = block.end().to_string();
    let size = (block.end().value() - block.start().value() + 1).to_string();
    let plane = block.start().plane().to_string();
    let entries = [
        KeyValue::new("Name", block.name()),
        KeyValue::new("Plane", &plane),
        KeyValue::new("Start", &start),
        KeyValue::new("End", &end),
        KeyValue::new("Size", &size),
    ];

    key_value::render(frame, area, "Selection", 7, &entries, color_theme);
}

fn render_plane_navigator(
    frame: &mut Frame,
    area: Rect,
    cursor: CodePoint,
    visible_items: std::ops::Range<usize>,
    color_theme: &ColorTheme,
    ui: &UiSettings,
) {
    let selected = Plane::for_code_point(cursor);
    let content = workspace::render_primary_heading(frame, area, "Planes", None, color_theme);
    let rows = visible_items
        .clone()
        .map(|number| {
            let plane = Plane::new(number as u8).expect("the viewport contains valid planes");
            let is_selected = plane == selected;
            let marker = ui.selection_marker(is_selected);
            selectable_list_line(
                Line::from(format!(
                    "{marker} Plane {:>2}  {}",
                    plane.number(),
                    plane.name().unwrap_or("Reserved")
                )),
                is_selected,
                content.width,
                color_theme.selection,
            )
        })
        .collect::<Vec<_>>();

    frame.render_widget(Paragraph::new(rows), content);
    frame.render_widget(
        ViewportScrollbar::new(Plane::COUNT, visible_items).style(color_theme.border_style()),
        scrollbar::area_for_primary(frame.area(), area, content),
    );
}

fn render_plane_context(
    frame: &mut Frame,
    area: Rect,
    cursor: CodePoint,
    color_theme: &ColorTheme,
) {
    let plane = Plane::for_code_point(cursor);
    let number = plane.number().to_string();
    let start = plane.start().to_string();
    let end = plane.end().to_string();
    let entries = [
        KeyValue::new("Plane", &number),
        KeyValue::new("Name", plane.name().unwrap_or("Reserved")),
        KeyValue::new("Start", &start),
        KeyValue::new("End", &end),
    ];

    key_value::render(frame, area, "Selection", 7, &entries, color_theme);
}

fn render_range_navigator(
    frame: &mut Frame,
    area: Rect,
    cursor: CodePoint,
    visible_items: std::ops::Range<usize>,
    color_theme: &ColorTheme,
    ui: &UiSettings,
) {
    let selected = PlaneRange::for_code_point(cursor);
    let plane = format!("Plane {}", selected.plane().number());
    let content =
        workspace::render_primary_heading(frame, area, "Ranges", Some(&plane), color_theme);
    let rows = visible_items
        .clone()
        .map(|index| {
            let range = selected.plane().range(index as u8);
            let is_selected = range == selected;
            let marker = ui.selection_marker(is_selected);
            selectable_list_line(
                Line::from(format!("{marker} {}–{}", range.start(), range.end())),
                is_selected,
                content.width,
                color_theme.selection,
            )
        })
        .collect::<Vec<_>>();

    frame.render_widget(Paragraph::new(rows), content);
    frame.render_widget(
        ViewportScrollbar::new(PlaneRange::COUNT_PER_PLANE, visible_items)
            .style(color_theme.border_style()),
        scrollbar::area_for_primary(frame.area(), area, content),
    );
}

fn render_range_context(
    frame: &mut Frame,
    area: Rect,
    cursor: CodePoint,
    color_theme: &ColorTheme,
) {
    let range = PlaneRange::for_code_point(cursor);
    let plane = range.plane();
    let plane_number = plane.number().to_string();
    let range_number = format!("{} of 255", range.index());
    let start = range.start().to_string();
    let end = range.end().to_string();
    let blocks = {
        let names =
            UnicodeDatabase::block_names_in_range(range.start(), range.end()).collect::<Vec<_>>();
        if names.is_empty() {
            "No Block".to_owned()
        } else {
            names.join("\n")
        }
    };
    let entries = [
        KeyValue::new("Plane", &plane_number),
        KeyValue::new("Name", plane.name().unwrap_or("Reserved")),
        KeyValue::new("Range", &range_number),
        KeyValue::new("Start", &start),
        KeyValue::new("End", &end),
        KeyValue::new("Blocks", &blocks),
    ];

    key_value::render(frame, area, "Selection", 7, &entries, color_theme);
}

fn render_code_point_table(
    frame: &mut Frame,
    area: Rect,
    browse: &BrowseState,
    visible_rows: std::ops::Range<usize>,
    color_theme: &ColorTheme,
    ui: &UiSettings,
) {
    let cursor = browse.cursor();
    let (page_start, page_end) = browse
        .table_page()
        .expect("the code point table has a visible page");
    let mut rows = Vec::with_capacity(visible_rows.len() + 1);
    rows.push(table_column_header(
        usize::try_from(cursor.value() % 16).expect("a table column fits usize"),
        color_theme,
    ));
    rows.extend(visible_rows.map(|row| {
        let row_start = page_start.value() + row as u32 * 16;
        let row_selected = cursor.value() >= row_start && cursor.value() < row_start + 16;
        let row_style = if row_selected {
            color_theme.accent_style()
        } else {
            Style::new().fg(color_theme.muted)
        };
        let mut spans = vec![Span::styled(format!("{row_start:06X} "), row_style)];
        for column in 0..16 {
            let code_point = CodePoint::new(row_start + column as u32)
                .expect("table pages contain valid code points");
            let selected = code_point == cursor;
            let cell = table_cell(code_point, selected, ui);
            spans.push(if selected {
                Span::styled(cell, color_theme.selection.style())
            } else {
                Span::raw(cell)
            });
        }
        Line::from(spans)
    }));

    let title = if browse.is_block_table() {
        "Block Code Points"
    } else {
        "Code Points"
    };
    let range = format!("{page_start}–{page_end}");
    let content = workspace::render_primary_heading(frame, area, title, Some(&range), color_theme);
    frame.render_widget(Paragraph::new(rows), content);
}

fn table_column_header(selected_column: usize, color_theme: &ColorTheme) -> Line<'static> {
    let mut spans = vec![Span::raw("       ")];
    for column in 0..16 {
        let style = if column == selected_column {
            color_theme.accent_style()
        } else {
            Style::new().fg(color_theme.muted)
        };
        spans.push(Span::styled(format!(" {column:X} "), style));
    }
    Line::from(spans)
}

fn table_cell(code_point: CodePoint, selected: bool, ui: &UiSettings) -> String {
    let representation = UnicodeDatabase::display_representation(code_point);
    let width = Line::from(representation.as_str()).width();
    let display = if matches!(width, 1 | 2) {
        representation.as_str()
    } else {
        "·"
    };
    let display_width = Line::from(display).width();
    let marker = ui.selection_marker(selected);

    format!("{marker}{display}{}", " ".repeat(2 - display_width))
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(0x0041, false, " A ")]
    #[case(0x4e00, false, " 一")]
    #[case(0x0301, true, ">◌́ ")]
    #[case(0x000a, false, " · ")]
    #[case(0x0378, false, " · ")]
    #[case(0xd800, true, ">· ")]
    #[case(0xe000, false, " · ")]
    #[case(0xfdd0, false, " · ")]
    fn table_cells_use_only_safe_one_or_two_column_representations(
        #[case] value: u32,
        #[case] selected: bool,
        #[case] expected: &str,
    ) {
        let code_point = CodePoint::new(value).unwrap();
        let ui = UiSettings {
            selection_cursor: ">".to_owned(),
            ..Default::default()
        };
        let cell = table_cell(code_point, selected, &ui);

        assert_eq!(cell, expected, "{code_point}");
        assert_eq!(Line::from(cell).width(), 3, "{code_point}");
    }

    #[test]
    fn table_header_and_rows_fit_the_minimum_navigator_width() {
        assert_eq!(table_column_header(0, &ColorTheme::default()).width(), 55);

        let cursor = CodePoint::new(0x0041).unwrap();
        let range = PlaneRange::for_code_point(cursor);
        let ui = UiSettings::default();
        let mut row = format!("{:06X} ", range.start().value());
        for offset in 0..16 {
            let code_point = range.code_point(offset);
            row.push_str(&table_cell(code_point, code_point == cursor, &ui));
        }

        assert_eq!(Line::from(row).width(), 55);
    }

    #[test]
    fn empty_selection_cursor_preserves_the_code_point_cell_width() {
        let cursor = CodePoint::new(0x0041).unwrap();

        assert_eq!(table_cell(cursor, true, &UiSettings::default()), " A ");
        assert_eq!(
            Line::from(table_cell(cursor, true, &UiSettings::default())).width(),
            3
        );
    }
}
