//! The setup for the CLI REPL

use std::{ffi::OsString, path::PathBuf};

use clap::Args;
use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};

use crate::{Graphic, TerminalLightness};

#[derive(Debug, Clone, Args, Deserialize, Serialize, Default)]
pub struct Setup {
    #[clap(long, short)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// The grafic level of the REPL
    pub(crate) graphic: Option<Graphic>,

    /// If the terminal is light or dark
    #[clap(long, short)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) teminal: Option<TerminalLightness>,

    /// The seed to use to initialize the random number generator
    #[clap(long, short)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) seed: Option<OsString>,
}

impl Setup {
    /// Extract the setup from the possible configuration sources
    pub fn extract_setups(file: Option<PathBuf>, cli: Setup) -> Result<Setup, figment::Error> {
        // Extracting the setup
        let mut figment = Figment::new().merge(Serialized::defaults(Setup::default()));
        // Seek first the default values
        if let Some(home) = home::home_dir() {
            // Then if the user has an home directory,
            let home_file = home.join("Dices.toml"); //  search a file called `Dices.toml` inside it
            if home_file.exists() {
                figment = figment.merge(Toml::file_exact(home_file))
            }
        }
        figment = figment.merge(Toml::file("./Dices.toml"));
        // Then any file called `Dices.toml` in this directory or superior ones
        if let Some(file_setup) = file {
            figment = figment.merge(Toml::file_exact(file_setup)) // If the user provided a setup file, look into it
        }
        figment = figment
            .merge(Env::prefixed("DICES_")) // Then all environmental variable
            .merge(Serialized::defaults(cli)); // Finally, the values provided by the CLI
        figment.extract()
    }
}
