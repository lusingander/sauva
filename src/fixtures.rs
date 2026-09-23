use tui_input::InputRequest;

use crate::{
    app::{Action, AppState, update},
    browser::BrowseLevel,
    glyph::CanvasSize,
    graphics::{GraphicsAvailability, GraphicsProtocol, GraphicsUnavailableReason},
    image::kitty::ImageId,
    inspector::InspectorMove,
    preview::{GlyphPreviewGeometry, GlyphPreviewStatus, GlyphPreviewUpdate},
    unicode::CodePoint,
};
#[cfg(test)]
use crate::{browser::BrowseMove, search::SearchMove};

pub fn startup() -> AppState {
    selected(0x0041)
}

#[cfg(test)]
pub fn sequence() -> AppState {
    let mut state = AppState::with_sequence(
        "A\u{0301} 👩‍💻"
            .chars()
            .map(CodePoint::from)
            .collect::<Vec<_>>(),
    );
    update(
        &mut state,
        Action::UpdateGlyphPreview(GlyphPreviewUpdate::Configure {
            availability: GraphicsAvailability::Unavailable(
                GraphicsUnavailableReason::UnsupportedTerminal,
            ),
            image_id: None,
        }),
    );
    state
}

pub fn default_ignorable() -> AppState {
    selected(0x115f)
}

pub fn unassigned() -> AppState {
    selected(0x0378)
}

pub fn surrogate() -> AppState {
    selected(0xd800)
}

pub fn details_aliases() -> AppState {
    selected(0x0000)
}

pub fn details_canonical_decomposition() -> AppState {
    selected(0x00e9)
}

#[cfg(test)]
pub fn details_compatibility_decomposition() -> AppState {
    selected(0xfb01)
}

pub fn details_supplementary() -> AppState {
    selected(0x1f600)
}

pub fn details_surrogate() -> AppState {
    surrogate()
}

pub fn glyph_basic() -> AppState {
    prepared_glyph(0x0041, GlyphPreviewStatus::Ready, 36, 25, 288, 400)
}

pub fn glyph_japanese() -> AppState {
    prepared_glyph(0x3042, GlyphPreviewStatus::Ready, 36, 25, 288, 400)
}

pub fn glyph_cjk() -> AppState {
    prepared_glyph(0x6f22, GlyphPreviewStatus::Ready, 36, 25, 288, 400)
}

pub fn glyph_combining() -> AppState {
    prepared_glyph(
        0x0301,
        GlyphPreviewStatus::CombiningContext,
        36,
        25,
        288,
        400,
    )
}

pub fn glyph_blank() -> AppState {
    prepared_glyph(0x0020, GlyphPreviewStatus::Blank, 36, 25, 288, 400)
}

pub fn glyph_missing() -> AppState {
    prepared_glyph(0x10ffff, GlyphPreviewStatus::Missing, 36, 25, 288, 400)
}

pub fn glyph_surrogate() -> AppState {
    prepared_glyph(0xd800, GlyphPreviewStatus::NotScalar, 36, 25, 288, 400)
}

pub fn glyph_disabled() -> AppState {
    configured_glyph(
        0x0041,
        GraphicsAvailability::Unavailable(GraphicsUnavailableReason::Disabled),
        None,
    )
}

#[cfg(test)]
pub fn glyph_wide() -> AppState {
    prepared_glyph(0x3042, GlyphPreviewStatus::Ready, 76, 35, 608, 560)
}

#[cfg(test)]
pub fn glyph_basic_iterm2() -> AppState {
    let geometry = preview_geometry(36, 25, 288, 400);
    let mut state = configured_glyph(
        0x0041,
        GraphicsAvailability::Available(GraphicsProtocol::Iterm2),
        None,
    );
    update(
        &mut state,
        Action::UpdateGlyphPreview(GlyphPreviewUpdate::Prepared {
            image_id: None,
            geometry,
            status: GlyphPreviewStatus::Ready,
            font: Some(crate::glyph::font::fixture_font_info()),
        }),
    );
    state
}

pub fn details_scrolled_minimum() -> AppState {
    let mut state = details_canonical_decomposition();
    // At the supported minimum size (60x16), U+00E9 has 35 logical
    // inspector lines and thirteen visible content lines.
    update(
        &mut state,
        Action::ResizeInspectorViewport {
            viewport_height: 13,
            document_height: 35,
            field_ranges: inspector_field_ranges(),
        },
    );
    update(&mut state, Action::MoveInspector(InspectorMove::Last));
    state
}

fn inspector_field_ranges() -> Vec<std::ops::Range<usize>> {
    (0..21)
        .map(|index| index..index + 1)
        .chain(std::iter::once(34..35))
        .collect()
}

pub fn search_empty() -> AppState {
    let mut state = startup();
    update(&mut state, Action::OpenSearch);
    state
}

pub fn search_name_results() -> AppState {
    search_with_query("rightwards arrow")
}

#[cfg(test)]
pub fn search_name_results_minimum() -> AppState {
    let mut state = search_name_results();
    update(&mut state, Action::ResizeSearchViewport(7));
    for _ in 0..8 {
        update(&mut state, Action::MoveSearch(SearchMove::Next));
    }
    state
}

pub fn search_code_point() -> AppState {
    search_with_query("U+2192")
}

#[cfg(test)]
pub fn search_combined_results() -> AppState {
    search_with_query("face")
}

pub fn search_literal() -> AppState {
    search_with_query("あ")
}

pub fn search_no_results() -> AppState {
    search_with_query("not a unicode name")
}

pub fn search_invalid_notation() -> AppState {
    search_with_query("U+110000")
}

pub fn search_special() -> AppState {
    search_with_query("U+D800")
}

pub fn browse_planes() -> AppState {
    browse(0x0041, BrowseLevel::Plane)
}

#[cfg(test)]
pub fn browse_planes_middle() -> AppState {
    browse(0x8_0041, BrowseLevel::Plane)
}

#[cfg(test)]
pub fn browse_planes_last() -> AppState {
    browse(0x10_ffff, BrowseLevel::Plane)
}

pub fn browse_ranges() -> AppState {
    browse(0x0041, BrowseLevel::Range)
}

pub fn browse_blocks() -> AppState {
    browse(0x0041, BrowseLevel::Block)
}

#[cfg(test)]
pub fn browse_blocks_from_gap() -> AppState {
    browse(0x2fe0, BrowseLevel::Block)
}

#[cfg(test)]
pub fn browse_block_code_points_short() -> AppState {
    let mut state = browse(0x2ff5, BrowseLevel::Block);
    update(&mut state, Action::AdvanceBrowser);
    state
}

#[cfg(test)]
pub fn browse_block_code_points_long_end() -> AppState {
    let mut state = browse(0x2_00ab, BrowseLevel::Block);
    update(&mut state, Action::AdvanceBrowser);
    update(&mut state, Action::MoveBrowser(BrowseMove::Last));
    state
}

pub fn browse_code_points() -> AppState {
    browse(0x0041, BrowseLevel::CodePointTable)
}

#[cfg(test)]
pub fn browse_code_points_minimum() -> AppState {
    browse(0x00f0, BrowseLevel::CodePointTable)
}

pub fn browse_special() -> AppState {
    browse(0xd800, BrowseLevel::CodePointTable)
}

pub fn browse_plane_16_end() -> AppState {
    browse(0x10_ffff, BrowseLevel::CodePointTable)
}

#[cfg(test)]
pub fn inspector_plane_16_end() -> AppState {
    selected(0x10_ffff)
}

#[cfg(test)]
pub fn browse_ranges_middle() -> AppState {
    browse(0x8041, BrowseLevel::Range)
}

#[cfg(test)]
pub fn browse_ranges_last() -> AppState {
    browse(0xff41, BrowseLevel::Range)
}

#[cfg(test)]
pub fn browse_ranges_after_reverse() -> AppState {
    let mut state = browse_ranges();
    update(&mut state, Action::ResizeBrowserViewport(10));
    for _ in 0..10 {
        update(&mut state, Action::MoveBrowser(BrowseMove::Down));
    }
    update(&mut state, Action::MoveBrowser(BrowseMove::Up));
    state
}

fn selected(value: u32) -> AppState {
    let code_point = CodePoint::new(value).expect("fixture code point must be valid");
    let mut state = AppState::with_selected(code_point);
    update(
        &mut state,
        Action::UpdateGlyphPreview(GlyphPreviewUpdate::Configure {
            availability: GraphicsAvailability::Unavailable(
                GraphicsUnavailableReason::UnsupportedTerminal,
            ),
            image_id: None,
        }),
    );
    state
}

fn prepared_glyph(
    value: u32,
    status: GlyphPreviewStatus,
    columns: u16,
    rows: u16,
    width: u16,
    height: u16,
) -> AppState {
    let image_id = preview_image_id();
    let geometry = preview_geometry(columns, rows, width, height);
    let mut state = configured_glyph(
        value,
        GraphicsAvailability::Available(GraphicsProtocol::Kitty),
        Some(image_id),
    );
    update(
        &mut state,
        Action::UpdateGlyphPreview(GlyphPreviewUpdate::Prepared {
            image_id: Some(image_id),
            geometry,
            status,
            font: matches!(
                status,
                GlyphPreviewStatus::Ready | GlyphPreviewStatus::CombiningContext
            )
            .then(crate::glyph::font::fixture_font_info),
        }),
    );
    state
}

fn configured_glyph(
    value: u32,
    availability: GraphicsAvailability,
    image_id: Option<ImageId>,
) -> AppState {
    let mut state = selected(value);
    update(
        &mut state,
        Action::UpdateGlyphPreview(GlyphPreviewUpdate::Configure {
            availability,
            image_id,
        }),
    );
    state
}

fn preview_geometry(columns: u16, rows: u16, width: u16, height: u16) -> GlyphPreviewGeometry {
    GlyphPreviewGeometry::new(
        columns,
        rows,
        CanvasSize::new(width, height).expect("fixture canvas must be valid"),
    )
}

fn preview_image_id() -> ImageId {
    ImageId::new(0x47_50_59).expect("fixture image ID must be valid")
}

fn browse(value: u32, level: BrowseLevel) -> AppState {
    let mut state = selected(value);
    update(&mut state, Action::OpenBrowser(level));
    state
}

fn search_with_query(value: &str) -> AppState {
    let mut state = search_empty();
    for character in value.chars() {
        update(
            &mut state,
            Action::EditSearch(InputRequest::InsertChar(character)),
        );
    }
    state
}
