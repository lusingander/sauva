mod code_point;
mod database;
mod decomposition;
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "Used by the forthcoming normalization view.")
)]
pub mod diff;
mod display;
mod encoding;
mod generated;
mod notation;
pub mod plane;
mod properties;
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "Grapheme and normalization metadata will be used by the Sequence Analyzer UI."
    )
)]
pub mod text;

pub use code_point::{CodePoint, CodePointStructure, InvalidCodePoint};
pub use database::{GeneralCategory, UnicodeBlock, UnicodeDatabase, UnicodeRecord};
pub use decomposition::Decomposition;
pub use decomposition::DecompositionType;
pub use display::{DisplayKind, DisplayRepresentation};
pub use encoding::UnicodeScalarEncoding;
pub use notation::{CodePointNotationError, parse_code_point_notation};
pub use plane::Plane;
pub use properties::{
    BidiClass, CanonicalCombiningClass, EastAsianWidth, NameAlias, NameAliasType,
};
