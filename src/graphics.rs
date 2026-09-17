use std::env;

use clap::ValueEnum;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
pub enum GraphicsMode {
    #[default]
    Auto,
    Force,
    Iterm2,
    Off,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphicsProtocol {
    Kitty,
    Iterm2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphicsUnavailableReason {
    Disabled,
    UnsupportedTerminal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphicsAvailability {
    Available(GraphicsProtocol),
    Unavailable(GraphicsUnavailableReason),
}

impl GraphicsAvailability {
    pub const fn is_available(self) -> bool {
        matches!(self, Self::Available(_))
    }
}

pub fn detect(mode: GraphicsMode) -> GraphicsAvailability {
    let term = env::var("TERM").ok();
    detect_from_term(mode, term.as_deref())
}

fn detect_from_term(mode: GraphicsMode, term: Option<&str>) -> GraphicsAvailability {
    match mode {
        GraphicsMode::Force => GraphicsAvailability::Available(GraphicsProtocol::Kitty),
        GraphicsMode::Iterm2 => GraphicsAvailability::Available(GraphicsProtocol::Iterm2),
        GraphicsMode::Off => GraphicsAvailability::Unavailable(GraphicsUnavailableReason::Disabled),
        GraphicsMode::Auto => match term {
            Some("xterm-kitty" | "xterm-ghostty") => {
                GraphicsAvailability::Available(GraphicsProtocol::Kitty)
            }
            _ => GraphicsAvailability::Unavailable(GraphicsUnavailableReason::UnsupportedTerminal),
        },
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::graphics::{
        GraphicsAvailability, GraphicsMode, GraphicsProtocol, GraphicsUnavailableReason,
        detect_from_term,
    };

    #[rstest]
    #[case("xterm-kitty")]
    #[case("xterm-ghostty")]
    fn auto_mode_recognizes_kitty_protocol_terminals(#[case] term: &str) {
        assert_eq!(
            detect_from_term(GraphicsMode::Auto, Some(term)),
            GraphicsAvailability::Available(GraphicsProtocol::Kitty)
        );
    }

    #[rstest]
    #[case(None)]
    #[case(Some("xterm-256color"))]
    #[case(Some("XTERM-KITTY"))]
    fn auto_mode_conservatively_rejects_other_term_values(#[case] term: Option<&str>) {
        assert_eq!(
            detect_from_term(GraphicsMode::Auto, term),
            GraphicsAvailability::Unavailable(GraphicsUnavailableReason::UnsupportedTerminal)
        );
    }

    #[test]
    fn force_mode_ignores_terminal_detection() {
        assert_eq!(
            detect_from_term(GraphicsMode::Force, Some("xterm-256color")),
            GraphicsAvailability::Available(GraphicsProtocol::Kitty)
        );
    }

    #[test]
    fn iterm2_mode_ignores_terminal_detection() {
        assert_eq!(
            detect_from_term(GraphicsMode::Iterm2, Some("xterm-kitty")),
            GraphicsAvailability::Available(GraphicsProtocol::Iterm2)
        );
    }

    #[test]
    fn off_mode_always_disables_graphics() {
        assert_eq!(
            detect_from_term(GraphicsMode::Off, Some("xterm-kitty")),
            GraphicsAvailability::Unavailable(GraphicsUnavailableReason::Disabled)
        );
    }
}
