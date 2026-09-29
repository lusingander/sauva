pub mod browser;
mod glyph_preview;
pub mod help;
pub mod inspector;
mod key_value;
pub mod layout;
pub mod render;
mod scrollbar;
mod search;
mod selection_preview;
mod sequence;
pub mod settings;
pub mod theme;
mod workspace;

use ratatui::layout::Rect;
use ratatui::text::{Line, Span};

use crate::{
    app::{AppState, View},
    ui::theme::SelectionColors,
    unicode::CodePoint,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlyphPreviewRequest {
    pub code_point: CodePoint,
    pub placeholder: Rect,
}

pub fn glyph_preview_request(area: Rect, state: &AppState) -> Option<GlyphPreviewRequest> {
    let shell = layout::calculate(area)?;
    let code_point = state.preview_code_point()?;
    let placeholder = match state.view() {
        View::Inspector => layout::inspector(shell.main)
            .preview
            .map(|preview| preview.placeholder),
        View::Browser => state.browse().and_then(|browse| {
            layout::browser(shell.main, browse.level())
                .context
                .and_then(|context| selection_preview::glyph_area(context, code_point))
        }),
        View::Search => layout::search(shell.main)
            .preview
            .and_then(|preview| selection_preview::glyph_area(preview, code_point)),
        View::Sequence => layout::sequence(shell.main)
            .context
            .and_then(|context| selection_preview::glyph_area(context, code_point)),
    }?;
    Some(GlyphPreviewRequest {
        code_point,
        placeholder,
    })
}

fn selectable_list_line(
    mut line: Line<'static>,
    selected: bool,
    content_width: u16,
    colors: SelectionColors,
) -> Line<'static> {
    if !selected {
        return line;
    }

    let padding = usize::from(content_width).saturating_sub(line.width());
    line.push_span(Span::raw(" ".repeat(padding)));
    line.style(colors.style())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures;

    #[test]
    fn requests_the_full_inspector_preview() {
        let request = glyph_preview_request(Rect::new(0, 0, 100, 30), &fixtures::startup())
            .expect("the standard inspector has a glyph preview");

        assert_eq!(request.code_point.value(), 0x0041);
        assert_eq!(request.placeholder, Rect::new(62, 4, 36, 24));
    }

    #[test]
    fn requests_inline_previews_for_search_and_code_points() {
        let search =
            glyph_preview_request(Rect::new(0, 0, 100, 30), &fixtures::search_name_results())
                .expect("a selected search result has an inline glyph preview");
        let browse =
            glyph_preview_request(Rect::new(0, 0, 100, 30), &fixtures::browse_code_points())
                .expect("a selected code point has an inline glyph preview");
        let block = glyph_preview_request(
            Rect::new(0, 0, 100, 30),
            &fixtures::browse_block_code_points_short(),
        )
        .expect("a block table selection has an inline glyph preview");

        assert_eq!(search.code_point.value(), 0x2192);
        assert_eq!(search.placeholder, Rect::new(62, 9, 36, 19));
        assert_eq!(browse.code_point.value(), 0x0041);
        assert_eq!(browse.placeholder, Rect::new(62, 11, 36, 17));
        assert_eq!(block.code_point.value(), 0x2ff5);
    }

    #[test]
    fn omits_inline_previews_without_a_selection_or_enough_space() {
        assert_eq!(
            glyph_preview_request(Rect::new(0, 0, 100, 30), &fixtures::search_empty()),
            None
        );
        assert_eq!(
            glyph_preview_request(Rect::new(0, 0, 100, 30), &fixtures::browse_planes()),
            None
        );
        assert_eq!(
            glyph_preview_request(Rect::new(0, 0, 100, 16), &fixtures::browse_code_points()),
            None
        );
    }
}
