use ratatui::{
    Frame,
    layout::Rect,
    text::Line,
    widgets::{Block, Padding, Paragraph},
};

use crate::{
    app::AppState,
    ui::{
        layout,
        scrollbar::{self, ViewportScrollbar},
        selectable_list_line, selection_preview,
        settings::UiSettings,
        theme::ColorTheme,
    },
    unicode::UnicodeDatabase,
};

pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    color_theme: &ColorTheme,
    ui: &UiSettings,
) {
    let layout = layout::sequence(area);
    let sequence = state
        .sequence()
        .expect("the sequence view always has sequence state");
    let count = sequence.code_points().len();
    let block = Block::bordered()
        .title(format!(" Sequence · {count} code points "))
        .padding(Padding::horizontal(1));
    let content = block.inner(layout.navigator);
    let position_width = count.to_string().len();
    let visible = sequence.visible_range();
    let rows = visible
        .clone()
        .map(|index| {
            let code_point = sequence.code_points()[index];
            let selected = sequence.selected_index() == index;
            let marker = ui.selection_marker(selected);
            let representation = UnicodeDatabase::display_representation(code_point);
            let name = UnicodeDatabase::primary_name_or_fallback(code_point);
            let code_point = format!("{code_point:<8}", code_point = code_point.to_string());
            selectable_list_line(
                Line::from(format!(
                    "{marker} {:>position_width$}  {code_point}  {representation} — {name}",
                    index + 1
                )),
                selected,
                content.width,
                color_theme.selection,
            )
        })
        .collect::<Vec<_>>();

    frame.render_widget(Paragraph::new(rows).block(block), layout.navigator);
    frame.render_widget(
        ViewportScrollbar::new(count, visible).style(color_theme.base_style()),
        scrollbar::area_after(content),
    );

    if let Some(context) = layout.context {
        selection_preview::render(
            frame,
            context,
            sequence.selected(),
            state.glyph_preview(),
            color_theme,
        );
    }
}

#[cfg(test)]
mod tests {
    use ratatui::{Terminal, backend::TestBackend};

    use super::*;
    use crate::{
        app::{Action, AppState, update},
        unicode::CodePoint,
    };

    #[test]
    fn renders_positions_code_points_and_names_in_input_order() {
        let mut state = AppState::with_sequence("A→A".chars().map(CodePoint::from).collect());
        update(&mut state, Action::ResizeSequenceViewport(27));
        let backend = TestBackend::new(100, 29);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                render(
                    frame,
                    frame.area(),
                    &state,
                    &ColorTheme::default(),
                    &UiSettings::default(),
                );
            })
            .unwrap();
        let text =
            terminal
                .backend()
                .buffer()
                .content()
                .iter()
                .fold(String::new(), |mut text, cell| {
                    text.push_str(cell.symbol());
                    text
                });

        assert!(text.contains("Sequence · 3 code points"));
        assert!(text.contains("1  U+0041"));
        assert!(text.contains("2  U+2192"));
        assert_eq!(text.matches("LATIN CAPITAL LETTER A").count(), 2);
    }
}
