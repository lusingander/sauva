mod app;
mod browser;
mod cli;
mod clipboard;
mod config;
mod copy_dialog;
mod fixtures;
mod glyph;
mod graphics;
mod help;
mod image;
mod input;
mod inspector;
mod keybindings;
mod normalization;
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
    let options = cli::parse()?;
    if options.print_default_config() {
        print!("{}", config::default_toml()?);
        return Ok(());
    }
    let config = config::load()?;
    let mut state = initial_state(options.demo(), options.target());

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

fn initial_state(demo: Option<cli::Demo>, target: Option<&cli::LaunchTarget>) -> app::AppState {
    debug_assert!(demo.is_none() || target.is_none());
    if let Some(target) = target {
        return match target {
            cli::LaunchTarget::CodePoint(code_point) => app::AppState::with_selected(*code_point),
            cli::LaunchTarget::Sequence(source) => app::AppState::with_sequence(source.clone()),
        };
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
    use crate::{cli::LaunchTarget, initial_state, unicode::CodePoint};

    #[test]
    fn initial_code_point_selects_the_inspected_value() {
        let code_point = CodePoint::new(0x2192).unwrap();

        let state = initial_state(None, Some(&LaunchTarget::CodePoint(code_point)));

        assert_eq!(state.selected(), code_point);
    }

    #[test]
    fn initial_sequence_opens_the_sequence_view() {
        let source = "A\u{0301} 👩‍💻";

        let state = initial_state(None, Some(&LaunchTarget::Sequence(source.to_owned())));

        assert_eq!(state.view(), crate::app::View::Sequence);
        assert_eq!(state.sequence().unwrap().selected().value(), 0x0041);
        let analysis = state.sequence().unwrap().analysis();
        assert_eq!(analysis.source(), source);
        assert_eq!(analysis.code_points().len(), 6);
        assert_eq!(analysis.graphemes().len(), 3);
    }
}
