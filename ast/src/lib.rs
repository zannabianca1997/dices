#![doc = include_str!("../README.md")]

pub mod fmt;
pub mod ident;
pub mod intrisics;

pub mod value;
use dices_version::Version;
pub use value::Value;

pub mod expression;
#[cfg(feature = "parse_expression")]
pub use expression::parse_file;
pub use expression::Expression;

#[cfg(feature = "matcher")]
pub mod matcher;
#[cfg(feature = "matcher")]
pub use matcher::Matcher;

pub const VERSION: Version = Version::new(
    env!("CARGO_PKG_VERSION_MAJOR"),
    env!("CARGO_PKG_VERSION_MINOR"),
    env!("CARGO_PKG_VERSION_PATCH"),
);
