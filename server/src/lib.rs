#![feature(duration_constructors)]

mod app;
mod domains;
mod entities;

pub use app::{App, BuildError, Config, DefaultConfig, FatalError};
pub use domains::ErrorCodes;

use dices_version::Version;
pub const VERSION: Version = Version::new(
    env!("CARGO_PKG_VERSION_MAJOR"),
    env!("CARGO_PKG_VERSION_MINOR"),
    env!("CARGO_PKG_VERSION_PATCH"),
);
