#![doc = include_str!("../README.md")]

use snafu::Snafu;

use crate::{cli::Cli, config::Config};

pub mod cli;
pub mod config;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(transparent)]
    Config { source: figment::Error },
}

pub fn main(Cli { config, seed: _ }: Cli) -> Result<(), Error> {
    let Config {
        history,
        skin,
        skins,
    } = Config::extract(config)?;
    todo!()
}
