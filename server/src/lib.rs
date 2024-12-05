#![feature(duration_constructors)]

mod boot;

mod app;
mod domains;

include!(env!("ENTITY_MODULE"));

pub use app::{App, BuildError, Config, DefaultConfig, FatalError};
pub use boot::{main, Cli, MainError};
pub use clap::Parser as ClapParser;
pub use domains::ErrorCodes;

use dices_version::Version;
pub const VERSION: Version = Version::new(
    env!("CARGO_PKG_VERSION_MAJOR"),
    env!("CARGO_PKG_VERSION_MINOR"),
    env!("CARGO_PKG_VERSION_PATCH"),
);
