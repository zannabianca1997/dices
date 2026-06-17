use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use figment::providers::{Env, Format, Serialized, Toml};
use figment::value::magic::Either;
use figment::{Figment, value::magic::RelativePathBuf};
use serde::{Deserialize, Serialize};

use crate::{
    cli::CliConfig,
    config::skin::{SelectedSkin, Skins},
};

pub mod skin;

fn directories() -> Option<ProjectDirs> {
    ProjectDirs::from("site.zannabianca1997.dices", "", "dices")
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// History file
    pub history: Either<RelativePathBuf, Option<PathBuf>>,
    /// Skin to use
    pub skin: SelectedSkin,
    /// Skins for the TUI
    pub skins: Skins,
}

impl Config {
    pub fn extract(
        CliConfig {
            config,
            no_default_config,
            skin,
        }: CliConfig,
    ) -> Result<Self, figment::Error> {
        let mut figment = Figment::new().merge(Serialized::defaults(Config::default()));

        if !no_default_config {
            if let Some(dirs) = directories()
                && let config = dirs.config_dir().join("Config.toml")
                && write_config_file_if_not_exists(&config).is_ok()
            {
                figment = figment.merge(Toml::file_exact(config));
            }
            figment = figment.merge(Toml::file("Dices.toml"));
        }

        if let Some(path) = config {
            figment = figment.merge(Toml::file_exact(path));
        }

        figment = figment.merge(Env::prefixed("DICES_").split("_"));

        let mut config: Self = figment.extract()?;
        config.skin = skin;
        Ok(config)
    }
}

fn write_config_file_if_not_exists(config: &Path) -> io::Result<()> {
    fs::create_dir_all(config.parent().unwrap())?;
    let mut file = match File::create_new(config) {
        Ok(file) => file,
        Err(err) if err.kind() == io::ErrorKind::AlreadyExists => return Ok(()),
        Err(err) => return Err(err),
    };
    let config = toml::to_string_pretty(&Config::default()).unwrap();
    file.write_all(config.as_bytes())
}

impl Default for Config {
    fn default() -> Self {
        Self {
            history: Either::Right(directories().map(|dirs| dirs.data_dir().join("history.txt"))),
            skin: Default::default(),
            skins: Default::default(),
        }
    }
}
