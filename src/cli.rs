use std::io::{self, Read};

use clap::{Parser, ValueEnum};

use crate::{graphics::GraphicsMode, unicode::CodePoint};

const CODE_POINT_ARGUMENT_HELP: &str =
    "use one character, U+XXXX, 0xXXXX, or 2-6 hexadecimal digits";
const INPUT_ARGUMENT_HELP: &str =
    "use text, one character, U+XXXX, 0xXXXX, or 2-6 hexadecimal digits";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaunchTarget {
    CodePoint(CodePoint),
    Sequence(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum InputSource {
    Target(LaunchTarget),
    Stdin { literal: bool },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchOptions {
    target: Option<LaunchTarget>,
    demo: Option<Demo>,
    graphics: GraphicsMode,
    print_default_config: bool,
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

    pub const fn print_default_config(&self) -> bool {
        self.print_default_config
    }
}

/// Sauva - Terminal Unicode Explorer 🪄
#[derive(Debug, Parser)]
#[command(version)]
struct Cli {
    /// Text or code point to inspect, or - to read stdin
    #[arg(
        value_name = "INPUT",
        value_parser = parse_input_source,
        conflicts_with_all = ["text", "demo"]
    )]
    input: Option<InputSource>,

    /// Treat the input as literal text, without code point notation parsing; - reads stdin
    #[arg(short, long, value_name = "TEXT", value_parser = parse_text_source, conflicts_with = "demo")]
    text: Option<InputSource>,

    /// Control glyph preview graphics
    #[arg(short, long, value_enum, default_value_t, value_name = "MODE")]
    graphics: GraphicsMode,

    /// Print the complete default configuration to standard output
    #[arg(long, exclusive = true)]
    print_default_config: bool,

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

pub fn parse() -> io::Result<LaunchOptions> {
    Cli::parse().resolve(io::stdin().lock())
}

impl Cli {
    fn resolve(self, stdin: impl Read) -> io::Result<LaunchOptions> {
        let target = match self.input.or(self.text) {
            None => None,
            Some(InputSource::Target(target)) => Some(target),
            Some(InputSource::Stdin { literal }) => {
                let input = io::read_to_string(stdin).map_err(|error| {
                    io::Error::new(error.kind(), format!("failed to read stdin: {error}"))
                })?;
                let target = if literal {
                    parse_text(&input)
                } else {
                    parse_input(&input)
                }
                .map_err(|error| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("invalid input from stdin: {error}"),
                    )
                })?;
                Some(target)
            }
        };

        Ok(LaunchOptions {
            target,
            demo: self.demo,
            graphics: self.graphics,
            print_default_config: self.print_default_config,
        })
    }
}

fn parse_input_source(input: &str) -> Result<InputSource, String> {
    if input == "-" {
        Ok(InputSource::Stdin { literal: false })
    } else {
        parse_input(input).map(InputSource::Target)
    }
}

fn parse_text_source(input: &str) -> Result<InputSource, String> {
    if input == "-" {
        Ok(InputSource::Stdin { literal: true })
    } else {
        parse_text(input).map(InputSource::Target)
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
    let mut characters = input.chars();
    match (characters.next(), characters.next()) {
        (None, _) => Err("text must not be empty".to_owned()),
        (Some(character), None) => Ok(LaunchTarget::CodePoint(CodePoint::from(character))),
        _ => Ok(LaunchTarget::Sequence(input.to_owned())),
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

    struct UnusedStdin;

    impl Read for UnusedStdin {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            panic!("stdin must not be read unless requested");
        }
    }

    fn try_parse(arguments: &[&str]) -> Result<LaunchOptions, clap::Error> {
        Cli::try_parse_from(arguments)
            .map(|cli| cli.resolve(UnusedStdin).expect("stdin is not requested"))
    }

    #[test]
    fn no_arguments_selects_normal_mode() {
        let options = try_parse(&["sauva"]).unwrap();

        assert_eq!(options.target(), None);
        assert_eq!(options.demo(), None);
        assert_eq!(options.graphics(), GraphicsMode::Auto);
        assert!(!options.print_default_config());
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

        let Some(LaunchTarget::Sequence(source)) = options.target() else {
            panic!("expected a sequence");
        };
        assert_eq!(source, argument);
        assert_eq!(
            source
                .chars()
                .map(|character| CodePoint::from(character).value())
                .collect::<Vec<_>>(),
            expected
        );
    }

    #[test]
    fn text_option_bypasses_code_point_notation_parsing() {
        let options = try_parse(&["sauva", "--text", "41"]).unwrap();

        assert_eq!(
            options.target(),
            Some(&LaunchTarget::Sequence("41".to_owned()))
        );
    }

    #[test]
    fn short_text_option_bypasses_code_point_notation_parsing() {
        let options = try_parse(&["sauva", "-t", "41"]).unwrap();

        assert_eq!(
            options.target(),
            Some(&LaunchTarget::Sequence("41".to_owned()))
        );
    }

    #[rstest]
    #[case(" \tA\r\n ")]
    #[case("q\u{0301}\u{0323}")]
    #[case("A\u{001b}B")]
    fn preserves_literal_source_verbatim(#[case] source: &str) {
        for arguments in [
            &["sauva", source][..],
            &["sauva", "--text", source],
            &["sauva", "-t", source],
        ] {
            let options = try_parse(arguments).unwrap();
            assert_eq!(
                options.target(),
                Some(&LaunchTarget::Sequence(source.to_owned()))
            );
        }
    }

    #[rstest]
    #[case('A')]
    #[case('é')]
    #[case('\r')]
    #[case('\u{0301}')]
    #[case('👩')]
    fn one_character_text_still_selects_a_code_point(#[case] character: char) {
        let input = character.to_string();
        let options = try_parse(&["sauva", "--text", &input]).unwrap();

        assert_eq!(
            options.target(),
            Some(&LaunchTarget::CodePoint(CodePoint::from(character)))
        );
    }

    #[rstest]
    #[case("A", 0x0041)]
    #[case("1", 0x0031)]
    #[case("→", 0x2192)]
    #[case("あ", 0x3042)]
    #[case("U+2192", 0x2192)]
    #[case("0x1F600", 0x1f600)]
    #[case("41", 0x0041)]
    #[case("U+D800", 0xd800)]
    #[case("-", 0x002d)]
    #[case(" ", 0x0020)]
    #[case("\n", 0x000a)]
    #[case("\r", 0x000d)]
    fn reads_code_points_from_stdin(#[case] input: &str, #[case] expected: u32) {
        let options = Cli::try_parse_from(["sauva", "-"])
            .unwrap()
            .resolve(input.as_bytes())
            .unwrap();

        assert_eq!(
            options.target(),
            Some(&LaunchTarget::CodePoint(CodePoint::new(expected).unwrap()))
        );
    }

    #[rstest]
    #[case("41")]
    #[case("U+2192")]
    #[case("U+GG")]
    #[case("0000041")]
    fn literal_stdin_bypasses_code_point_notation_parsing(#[case] input: &str) {
        for text_option in ["--text", "-t"] {
            let options = Cli::try_parse_from(["sauva", text_option, "-"])
                .unwrap()
                .resolve(input.as_bytes())
                .unwrap();

            assert_eq!(
                options.target(),
                Some(&LaunchTarget::Sequence(input.to_owned()))
            );
        }
    }

    #[rstest]
    #[case(" \tA\r\n ")]
    #[case("A\nB\n")]
    #[case("A\n\n")]
    #[case("q\u{0301}\u{0323}")]
    #[case("❤️\n")]
    fn preserves_stdin_text_verbatim(#[case] source: &str) {
        for arguments in [
            &["sauva", "-"][..],
            &["sauva", "--text", "-"],
            &["sauva", "-t", "-"],
        ] {
            let options = Cli::try_parse_from(arguments)
                .unwrap()
                .resolve(source.as_bytes())
                .unwrap();

            assert_eq!(
                options.target(),
                Some(&LaunchTarget::Sequence(source.to_owned()))
            );
        }
    }

    #[test]
    fn reads_stdin_text_longer_than_a_code_point_notation() {
        let source = "hello".repeat(100);
        let options = Cli::try_parse_from(["sauva", "-"])
            .unwrap()
            .resolve(source.as_bytes())
            .unwrap();

        assert_eq!(options.target(), Some(&LaunchTarget::Sequence(source)));
    }

    #[rstest]
    #[case("")]
    #[case("U+GG")]
    #[case("U+2192\n")]
    #[case("U+110000")]
    fn rejects_invalid_stdin_input(#[case] input: &str) {
        let error = Cli::try_parse_from(["sauva", "-"])
            .unwrap()
            .resolve(input.as_bytes())
            .unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        assert!(error.to_string().contains("invalid input from stdin"));
    }

    #[test]
    fn rejects_empty_literal_stdin() {
        let error = Cli::try_parse_from(["sauva", "--text", "-"])
            .unwrap()
            .resolve(io::empty())
            .unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        assert!(error.to_string().contains("text must not be empty"));
    }

    #[rstest]
    #[case(&["sauva", "-"])]
    #[case(&["sauva", "--text", "-"])]
    fn rejects_non_utf8_stdin(#[case] arguments: &[&str]) {
        let error = Cli::try_parse_from(arguments)
            .unwrap()
            .resolve([0xff].as_slice())
            .unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(error.to_string().contains("failed to read stdin"));
    }

    #[rstest]
    #[case(&["sauva", "-", "--demo", "glyph-basic"])]
    #[case(&["sauva", "--text", "-", "--demo", "glyph-basic"])]
    #[case(&["sauva", "-", "--text", "-"])]
    #[case(&["sauva", "-", "--print-default-config"])]
    #[case(&["sauva", "--text", "-", "--print-default-config"])]
    fn stdin_requests_respect_argument_conflicts(#[case] arguments: &[&str]) {
        let error = Cli::try_parse_from(arguments).unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::ArgumentConflict);
    }

    #[test]
    fn hyphen_can_be_inspected_with_code_point_notation() {
        let options = try_parse(&["sauva", "U+002D"]).unwrap();

        assert_eq!(
            options.target(),
            Some(&LaunchTarget::CodePoint(CodePoint::from('-')))
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
        assert!(help.contains("or - to read stdin"));
        assert!(help.contains("-t, --text <TEXT>"));
        assert!(help.contains("- reads stdin"));
        assert!(help.contains("--graphics <MODE>"));
        assert!(help.contains("possible values: auto, force, iterm2, off"));
        assert!(help.contains("--print-default-config"));
    }

    #[test]
    fn print_default_config_is_a_standalone_action() {
        let options = try_parse(&["sauva", "--print-default-config"]).unwrap();

        assert!(options.print_default_config());
        assert_eq!(options.target(), None);

        for arguments in [
            &["sauva", "--print-default-config", "A"][..],
            &["sauva", "--print-default-config", "--text", "A"][..],
            &["sauva", "--print-default-config", "--graphics", "off"][..],
        ] {
            assert_eq!(
                try_parse(arguments).unwrap_err().kind(),
                clap::error::ErrorKind::ArgumentConflict
            );
        }
    }

    #[test]
    fn disabled_glyph_demo_overrides_the_graphics_option() {
        let options =
            try_parse(&["sauva", "--demo", "glyph-disabled", "--graphics", "force"]).unwrap();

        assert_eq!(options.graphics(), GraphicsMode::Off);
    }
}
