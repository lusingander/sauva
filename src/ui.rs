pub mod browser;
mod copy_dialog;
mod glyph_preview;
pub mod help;
pub mod inspector;
mod key_value;
pub mod layout;
mod normalization;
pub mod normalization_result;
pub mod render;
mod scrollbar;
mod search;
mod selection_preview;
mod sequence;
pub mod settings;
mod text_preview;
pub mod theme;
mod workspace;

use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use unicode_segmentation::UnicodeSegmentation;

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
                .and_then(|context| selection_preview::glyph_area(context, code_point, None))
        }),
        View::Search => {
            let matched_alias = state
                .search()
                .and_then(|search| search.selected_result())
                .and_then(|result| result.preferred_alias_match())
                .map(|alias_match| alias_match.alias());
            layout::search(shell.main).preview.and_then(|preview| {
                selection_preview::glyph_area(preview, code_point, matched_alias)
            })
        }
        View::Sequence if state.showing_normalization_result() => None,
        View::Sequence => {
            let sequence = state.sequence()?;
            layout::sequence(shell.main)
                .context
                .and_then(|context| selection_preview::glyph_area_for_sequence(context, sequence))
        }
        View::Normalization => None,
    }?;
    Some(GlyphPreviewRequest {
        code_point,
        placeholder,
    })
}

fn selectable_list_line(
    line: Line<'static>,
    selected: bool,
    content_width: u16,
    colors: SelectionColors,
) -> Line<'static> {
    let line = padded_line(line, usize::from(content_width));
    if selected {
        line.style(colors.style())
    } else {
        line
    }
}

fn padded_line_content_width(width: usize) -> usize {
    width.saturating_sub(2)
}

fn padded_line(mut line: Line<'static>, width: usize) -> Line<'static> {
    let mut remaining = padded_line_content_width(width);
    let mut spans = vec![Span::raw(" ".repeat(width.min(1)))];
    for mut span in line.spans {
        let mut end = 0;
        let mut clipped = false;
        for (index, grapheme) in span.content.grapheme_indices(true) {
            let grapheme_width = Span::raw(grapheme).width();
            if grapheme_width > remaining {
                clipped = true;
                break;
            }
            remaining -= grapheme_width;
            end = index + grapheme.len();
        }
        if end < span.content.len() {
            span.content = span.content[..end].to_owned().into();
        }
        spans.push(span);
        if clipped {
            break;
        }
    }
    spans.push(Span::raw(" ".repeat(remaining + usize::from(width >= 2))));
    line.spans = spans;
    line
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures;

    #[test]
    fn copy_dialog_and_its_help_hide_preview_requests_until_returning_to_sequence() {
        use crate::app::{Action, update};
        let mut state = fixtures::sequence();
        let area = Rect::new(0, 0, 100, 30);
        let original = glyph_preview_request(area, &state).unwrap();
        update(&mut state, Action::OpenCopyDialog);
        assert!(glyph_preview_request(area, &state).is_none());
        update(&mut state, Action::ToggleHelp);
        assert!(glyph_preview_request(area, &state).is_none());
        update(&mut state, Action::CloseHelp);
        assert!(glyph_preview_request(area, &state).is_none());
        update(&mut state, Action::CloseCopyDialog);
        assert_eq!(glyph_preview_request(area, &state).unwrap(), original);
    }

    #[rstest::rstest]
    #[case(crate::graphics::GraphicsProtocol::Kitty)]
    #[case(crate::graphics::GraphicsProtocol::Iterm2)]
    fn copy_dialog_hides_and_restores_the_runtime_image(
        #[case] protocol: crate::graphics::GraphicsProtocol,
    ) {
        use crate::{
            app::{Action, update},
            glyph::runtime::GlyphPreviewRuntime,
            graphics::{GraphicsAvailability, GraphicsProtocol},
            preview::GlyphPreviewStatus,
        };
        use ratatui::buffer::Buffer;

        let mut state = fixtures::sequence();
        let area = Rect::new(0, 0, 100, 30);
        let mut runtime = GlyphPreviewRuntime::new(GraphicsAvailability::Available(protocol));
        let mut synchronize = |state: &mut AppState| {
            let request = glyph_preview_request(area, state);
            let mut output = Vec::new();
            let preview = runtime.synchronize(
                &mut output,
                request.map_or(state.selected(), |request| request.code_point),
                request.map(|request| request.placeholder),
                None,
            );
            update(state, Action::UpdateGlyphPreview(preview));
            let mut rendered = Buffer::empty(area);
            runtime.render_image(&mut rendered);
            (output, rendered)
        };
        synchronize(&mut state);
        assert_eq!(state.glyph_preview().status(), GlyphPreviewStatus::Ready);

        update(&mut state, Action::OpenCopyDialog);
        let (hidden_output, hidden_image) = synchronize(&mut state);
        assert_eq!(state.glyph_preview().status(), GlyphPreviewStatus::Hidden);
        assert!(
            hidden_image
                .content
                .iter()
                .all(|cell| !cell.symbol().contains("]1337;"))
        );
        if protocol == GraphicsProtocol::Kitty {
            assert!(String::from_utf8_lossy(&hidden_output).contains("a=d"));
        }
        for action in [Action::ToggleHelp, Action::CloseHelp] {
            update(&mut state, action);
            assert!(synchronize(&mut state).0.is_empty());
            assert_eq!(state.glyph_preview().status(), GlyphPreviewStatus::Hidden);
        }

        update(&mut state, Action::CloseCopyDialog);
        let (restored_output, restored_image) = synchronize(&mut state);
        assert_eq!(state.glyph_preview().status(), GlyphPreviewStatus::Ready);
        if protocol == GraphicsProtocol::Kitty {
            assert!(!restored_output.is_empty());
        } else {
            assert!(
                restored_image
                    .content
                    .iter()
                    .any(|cell| cell.symbol().contains("]1337;"))
            );
        }
    }

    #[test]
    fn list_padding_survives_clipping_wide_and_combining_graphemes() {
        use ratatui::{buffer::Buffer, style::Modifier, widgets::Widget};

        let theme = crate::ui::theme::ColorTheme::default();
        for selected in [false, true] {
            let line = Line::from(vec![
                Span::styled(
                    "AB",
                    ratatui::style::Style::new().add_modifier(Modifier::BOLD),
                ),
                Span::raw("あe\u{0301}👩‍💻TAIL"),
            ]);
            let line = selectable_list_line(line, selected, 8, theme.selection);
            assert_eq!(line.to_string(), " ABあe\u{0301}  ");
            assert_eq!(line.width(), 8);

            let area = Rect::new(0, 0, 8, 1);
            let mut buffer = Buffer::empty(area);
            line.render(area, &mut buffer);
            assert_eq!(buffer[(0, 0)].symbol(), " ");
            assert_eq!(buffer[(7, 0)].symbol(), " ");
            assert!(buffer[(1, 0)].modifier.contains(Modifier::BOLD));
            assert_eq!(buffer[(5, 0)].symbol(), "e\u{0301}");
            if selected {
                for x in [0, 1, 2, 3, 5, 6, 7] {
                    assert_eq!(buffer[(x, 0)].bg, theme.selection.bg);
                }
            }
        }
    }

    #[test]
    fn list_padding_fits_areas_narrower_than_the_padding() {
        for width in 0..=2 {
            let line = selectable_list_line(
                Line::from("あABC"),
                true,
                width,
                crate::ui::theme::ColorTheme::default().selection,
            );
            assert_eq!(line.to_string(), " ".repeat(usize::from(width)));
        }
    }

    #[test]
    fn normalization_result_uses_the_reference_pane_instead_of_a_glyph_preview() {
        use crate::app::{Action, update};
        let mut state = AppState::with_sequence("A\u{0301}".to_owned());
        let area = Rect::new(0, 0, 100, 30);
        assert!(glyph_preview_request(area, &state).is_some());
        update(&mut state, Action::OpenNormalization);
        update(&mut state, Action::InspectNormalizationResult);
        assert_eq!(glyph_preview_request(area, &state), None);
        update(&mut state, Action::InspectSequenceCodePoint);
        assert!(glyph_preview_request(area, &state).is_some());
    }

    #[test]
    fn requests_the_full_inspector_preview() {
        let request = glyph_preview_request(Rect::new(0, 0, 100, 30), &fixtures::startup())
            .expect("the standard inspector has a glyph preview");

        assert_eq!(request.code_point.value(), 0x0041);
        assert_eq!(request.placeholder, Rect::new(62, 6, 36, 20));
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
        assert_eq!(search.placeholder, Rect::new(63, 14, 36, 14));
        assert_eq!(browse.code_point.value(), 0x0041);
        assert_eq!(browse.placeholder, Rect::new(63, 14, 36, 14));
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

    #[test]
    fn sequence_preview_remains_a_single_code_point_with_cluster_aware_geometry() {
        let mut state =
            AppState::with_sequence(format!("{}A{}", "X".repeat(10), "\u{0301}".repeat(11)));
        crate::app::update(
            &mut state,
            crate::app::Action::MoveSequence(crate::sequence::SequenceMove::Last),
        );

        let request = glyph_preview_request(Rect::new(0, 0, 100, 30), &state).unwrap();

        assert_eq!(request.code_point.value(), 0x0301);
        assert_eq!(request.placeholder, Rect::new(63, 15, 36, 13));
        assert_eq!(glyph_preview_request(Rect::new(0, 0, 60, 16), &state), None);
    }
}
