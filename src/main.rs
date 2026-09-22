mod app;
mod browser;
mod cli;
mod clipboard;
mod config;
mod fixtures;
mod glyph;
mod graphics;
mod help;
mod image;
mod input;
mod inspector;
mod keybindings;
mod preview;
mod search;
mod sequence;
mod terminal;
mod ui;
mod unicode;
mod viewport;

use std::{error::Error, process::ExitCode};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("sauva: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let options = cli::parse();
    let config = config::load()?;
    let mut state = initial_state(options.demo(), options.initial_code_point());

    terminal::run(
        &mut state,
        options.graphics(),
        config.glyph_preview(),
        config.color_theme(),
        config.ui(),
        config.keymap(),
    )?;
    Ok(())
}

fn initial_state(
    demo: Option<cli::Demo>,
    initial_code_point: Option<unicode::CodePoint>,
) -> app::AppState {
    debug_assert!(demo.is_none() || initial_code_point.is_none());
    if let Some(code_point) = initial_code_point {
        return app::AppState::with_selected(code_point);
    }

    match demo {
        None => fixtures::startup(),
        Some(cli::Demo::DefaultIgnorable) => fixtures::default_ignorable(),
        Some(cli::Demo::Unassigned) => fixtures::unassigned(),
        Some(cli::Demo::Surrogate) => fixtures::surrogate(),
        Some(cli::Demo::BrowsePlanes) => fixtures::browse_planes(),
        Some(cli::Demo::BrowseRanges) => fixtures::browse_ranges(),
        Some(cli::Demo::BrowseBlocks) => fixtures::browse_blocks(),
        Some(cli::Demo::BrowseCodePoints) => fixtures::browse_code_points(),
        Some(cli::Demo::BrowseSpecial) => fixtures::browse_special(),
        Some(cli::Demo::BrowsePlane16End) => fixtures::browse_plane_16_end(),
        Some(cli::Demo::SearchEmpty) => fixtures::search_empty(),
        Some(cli::Demo::SearchNameResults) => fixtures::search_name_results(),
        Some(cli::Demo::SearchCodePoint) => fixtures::search_code_point(),
        Some(cli::Demo::SearchLiteral) => fixtures::search_literal(),
        Some(cli::Demo::SearchNoResults) => fixtures::search_no_results(),
        Some(cli::Demo::SearchInvalidNotation) => fixtures::search_invalid_notation(),
        Some(cli::Demo::SearchSpecial) => fixtures::search_special(),
        Some(cli::Demo::DetailsAliases) => fixtures::details_aliases(),
        Some(cli::Demo::DetailsCanonicalDecomposition) => {
            fixtures::details_canonical_decomposition()
        }
        Some(cli::Demo::DetailsSupplementary) => fixtures::details_supplementary(),
        Some(cli::Demo::DetailsSurrogate) => fixtures::details_surrogate(),
        Some(cli::Demo::DetailsScrolledMinimum) => fixtures::details_scrolled_minimum(),
        Some(cli::Demo::GlyphBasic) => fixtures::glyph_basic(),
        Some(cli::Demo::GlyphJapanese) => fixtures::glyph_japanese(),
        Some(cli::Demo::GlyphCjk) => fixtures::glyph_cjk(),
        Some(cli::Demo::GlyphCombining) => fixtures::glyph_combining(),
        Some(cli::Demo::GlyphBlank) => fixtures::glyph_blank(),
        Some(cli::Demo::GlyphMissing) => fixtures::glyph_missing(),
        Some(cli::Demo::GlyphSurrogate) => fixtures::glyph_surrogate(),
        Some(cli::Demo::GlyphDisabled) => fixtures::glyph_disabled(),
    }
}

#[cfg(test)]
mod tests {
    use crate::{initial_state, unicode::CodePoint};

    #[test]
    fn initial_code_point_selects_the_inspected_value() {
        let code_point = CodePoint::new(0x2192).unwrap();

        let state = initial_state(None, Some(code_point));

        assert_eq!(state.selected(), code_point);
    }
}
