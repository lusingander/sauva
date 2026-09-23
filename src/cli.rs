use clap::{Parser, ValueEnum};

use crate::{graphics::GraphicsMode, unicode::CodePoint};

const CODE_POINT_ARGUMENT_HELP: &str =
    "use one character, U+XXXX, 0xXXXX, or 2-6 hexadecimal digits";
const INPUT_ARGUMENT_HELP: &str =
    "use text, one character, U+XXXX, 0xXXXX, or 2-6 hexadecimal digits";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaunchTarget {
    CodePoint(CodePoint),
    Sequence(Vec<CodePoint>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchOptions {
    target: Option<LaunchTarget>,
    demo: Option<Demo>,
    graphics: GraphicsMode,
}

impl LaunchOptions {
    pub fn target(&self) -> Option<&LaunchTarget> {
        self.target.as_ref()
    }

    pub const fn demo(&self) -> Option<Demo> {
        self.demo
    }

    pub const fn graphics(&self) -> GraphicsMode {
        if matches!(self.demo, Some(Demo::GlyphDisabled)) {
            GraphicsMode::Off
        } else {
            self.graphics
        }
    }
}

/// Sauva - Terminal Unicode Explorer 🪄
#[derive(Debug, Parser)]
#[command(version)]
struct Cli {
    /// Text or code point to inspect
    #[arg(
        value_name = "INPUT",
        value_parser = parse_input,
        conflicts_with_all = ["text", "demo"]
    )]
    input: Option<LaunchTarget>,

    /// Treat the input as literal text, without code point notation parsing
    #[arg(short, long, value_name = "TEXT", value_parser = parse_text, conflicts_with = "demo")]
    text: Option<LaunchTarget>,

    /// Control glyph preview graphics
    #[arg(short, long, value_enum, default_value_t, value_name = "MODE")]
    graphics: GraphicsMode,

    /// Show a deterministic UI state
    #[arg(long, value_enum, value_name = "STATE", hide = true)]
    demo: Option<Demo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Demo {
    DefaultIgnorable,
    Unassigned,
    Surrogate,
    BrowsePlanes,
    BrowseRanges,
    BrowseBlocks,
    BrowseCodePoints,
    BrowseSpecial,
    #[value(name = "browse-plane-16-end")]
    BrowsePlane16End,
    SearchEmpty,
    SearchNameResults,
    SearchCodePoint,
    SearchLiteral,
    SearchNoResults,
    SearchInvalidNotation,
    SearchSpecial,
    DetailsAliases,
    DetailsCanonicalDecomposition,
    DetailsSupplementary,
    DetailsSurrogate,
    DetailsScrolledMinimum,
    GlyphBasic,
    GlyphJapanese,
    GlyphCjk,
    GlyphCombining,
    GlyphBlank,
    GlyphMissing,
    GlyphSurrogate,
    GlyphDisabled,
}

pub fn parse() -> LaunchOptions {
    Cli::parse().into()
}

impl From<Cli> for LaunchOptions {
    fn from(cli: Cli) -> Self {
        Self {
            target: cli.input.or(cli.text),
            demo: cli.demo,
            graphics: cli.graphics,
        }
    }
}

fn parse_input(input: &str) -> Result<LaunchTarget, String> {
    let mut characters = input.chars();
    if let (Some(character), None) = (characters.next(), characters.next()) {
        return Ok(LaunchTarget::CodePoint(CodePoint::from(character)));
    }

    let prefixed_digits = input
        .strip_prefix("U+")
        .or_else(|| input.strip_prefix("u+"))
        .or_else(|| input.strip_prefix("0x"))
        .or_else(|| input.strip_prefix("0X"));
    if let Some(digits) = prefixed_digits {
        return parse_code_point_digits(digits, 1).map(LaunchTarget::CodePoint);
    }
    if !input.is_empty() && input.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return parse_code_point_digits(input, 2).map(LaunchTarget::CodePoint);
    }

    parse_text(input).map_err(|_| INPUT_ARGUMENT_HELP.to_owned())
}

fn parse_text(input: &str) -> Result<LaunchTarget, String> {
    let code_points = input.chars().map(CodePoint::from).collect::<Vec<_>>();
    match code_points.as_slice() {
        [] => Err("text must not be empty".to_owned()),
        [code_point] => Ok(LaunchTarget::CodePoint(*code_point)),
        _ => Ok(LaunchTarget::Sequence(code_points)),
    }
}

fn parse_code_point_digits(digits: &str, minimum_digits: usize) -> Result<CodePoint, String> {
    if !(minimum_digits..=6).contains(&digits.len())
        || !digits.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(CODE_POINT_ARGUMENT_HELP.to_owned());
    }

    let value = u32::from_str_radix(digits, 16).map_err(|_| CODE_POINT_ARGUMENT_HELP.to_owned())?;
    CodePoint::new(value).map_err(|_| {
        format!(
            "code point must be between U+{:04X} and U+{:X}",
            CodePoint::MIN_VALUE,
            CodePoint::MAX_VALUE
        )
    })
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;
    use rstest::rstest;

    use super::*;

    fn try_parse(arguments: &[&str]) -> Result<LaunchOptions, clap::Error> {
        Cli::try_parse_from(arguments).map(Into::into)
    }

    #[test]
    fn no_arguments_selects_normal_mode() {
        let options = try_parse(&["sauva"]).unwrap();

        assert_eq!(options.target(), None);
        assert_eq!(options.demo(), None);
        assert_eq!(options.graphics(), GraphicsMode::Auto);
    }

    #[rstest]
    #[case("A", 0x0041)]
    #[case("1", 0x0031)]
    #[case("→", 0x2192)]
    #[case("あ", 0x3042)]
    #[case("U+A", 0x000a)]
    #[case("u+1f600", 0x1f600)]
    #[case("0x41", 0x0041)]
    #[case("0X10FFFF", 0x10_ffff)]
    #[case("41", 0x0041)]
    #[case("01F600", 0x1f600)]
    fn accepts_literal_and_numeric_code_points(#[case] argument: &str, #[case] expected: u32) {
        let options = try_parse(&["sauva", argument]).unwrap();

        assert_eq!(
            options.target(),
            Some(&LaunchTarget::CodePoint(CodePoint::new(expected).unwrap()))
        );
    }

    #[rstest]
    #[case("hello", &[0x68, 0x65, 0x6c, 0x6c, 0x6f])]
    #[case("❤️", &[0x2764, 0xfe0f])]
    #[case("👩‍💻", &[0x1f469, 0x200d, 0x1f4bb])]
    fn accepts_literal_sequences(#[case] argument: &str, #[case] expected: &[u32]) {
        let options = try_parse(&["sauva", argument]).unwrap();

        let Some(LaunchTarget::Sequence(code_points)) = options.target() else {
            panic!("expected a sequence");
        };
        assert_eq!(
            code_points
                .iter()
                .map(|code_point| code_point.value())
                .collect::<Vec<_>>(),
            expected
        );
    }

    #[test]
    fn text_option_bypasses_code_point_notation_parsing() {
        let options = try_parse(&["sauva", "--text", "41"]).unwrap();

        let Some(LaunchTarget::Sequence(code_points)) = options.target() else {
            panic!("expected a sequence");
        };
        assert_eq!(
            code_points
                .iter()
                .map(|code_point| code_point.value())
                .collect::<Vec<_>>(),
            [0x0034, 0x0031]
        );
    }

    #[test]
    fn short_text_option_bypasses_code_point_notation_parsing() {
        let options = try_parse(&["sauva", "-t", "41"]).unwrap();

        let Some(LaunchTarget::Sequence(code_points)) = options.target() else {
            panic!("expected a sequence");
        };
        assert_eq!(
            code_points
                .iter()
                .map(|code_point| code_point.value())
                .collect::<Vec<_>>(),
            [0x0034, 0x0031]
        );
    }

    #[test]
    fn one_character_text_still_selects_a_code_point() {
        let options = try_parse(&["sauva", "--text", "A"]).unwrap();

        assert_eq!(
            options.target(),
            Some(&LaunchTarget::CodePoint(CodePoint::from('A')))
        );
    }

    #[rstest]
    #[case("U+")]
    #[case("0x")]
    #[case("0xGG")]
    #[case("U+00_41")]
    #[case("0000041")]
    fn rejects_invalid_code_point_arguments(#[case] argument: &str) {
        let error = try_parse(&["sauva", argument]).unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::ValueValidation);
        assert!(error.to_string().contains(CODE_POINT_ARGUMENT_HELP));
    }

    #[test]
    fn rejects_empty_positional_input() {
        let error = try_parse(&["sauva", ""]).unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::ValueValidation);
        assert!(error.to_string().contains(INPUT_ARGUMENT_HELP));
    }

    #[test]
    fn rejects_empty_text() {
        let error = try_parse(&["sauva", "--text", ""]).unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::ValueValidation);
        assert!(error.to_string().contains("text must not be empty"));
    }

    #[test]
    fn positional_input_and_text_option_conflict() {
        let error = try_parse(&["sauva", "A", "--text", "B"]).unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::ArgumentConflict);
    }

    #[test]
    fn rejects_code_points_outside_the_unicode_code_space() {
        let error = try_parse(&["sauva", "U+110000"]).unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::ValueValidation);
        assert!(error.to_string().contains("between U+0000 and U+10FFFF"));
    }

    #[test]
    fn code_point_argument_conflicts_with_demo_state() {
        let error = try_parse(&["sauva", "U+2192", "--demo", "glyph-basic"]).unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::ArgumentConflict);
    }

    #[test]
    fn unknown_graphics_mode_is_rejected() {
        let error = try_parse(&["sauva", "--graphics", "sometimes"]).unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::InvalidValue);
    }

    #[test]
    fn help_documents_the_graphics_modes() {
        let help = Cli::command().render_long_help().to_string();

        assert!(help.contains("Usage: sauva [OPTIONS] [INPUT]"));
        assert!(help.contains("Text or code point to inspect"));
        assert!(help.contains("-t, --text <TEXT>"));
        assert!(help.contains("--graphics <MODE>"));
        assert!(help.contains("possible values: auto, force, iterm2, off"));
    }

    #[test]
    fn disabled_glyph_demo_overrides_the_graphics_option() {
        let options =
            try_parse(&["sauva", "--demo", "glyph-disabled", "--graphics", "force"]).unwrap();

        assert_eq!(options.graphics(), GraphicsMode::Off);
    }
}
