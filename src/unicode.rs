mod code_point;
mod database;
mod decomposition;
mod display;
mod encoding;
mod generated;
mod notation;
pub mod plane;
mod properties;

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
