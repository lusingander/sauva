mod code_point;
mod database;
mod decomposition;
pub mod diff;
mod display;
mod encoding;
mod generated;
mod notation;
pub mod plane;
mod properties;
pub mod text;

pub use crate::unicode::{
    code_point::{CodePoint, CodePointStructure, InvalidCodePoint},
    database::{GeneralCategory, UnicodeBlock, UnicodeDatabase, UnicodeRecord},
    decomposition::{Decomposition, DecompositionType},
    display::{DisplayKind, DisplayRepresentation},
    encoding::UnicodeScalarEncoding,
    notation::{CodePointNotationError, parse_code_point_notation},
    plane::Plane,
    properties::{BidiClass, CanonicalCombiningClass, EastAsianWidth, NameAlias, NameAliasType},
};
