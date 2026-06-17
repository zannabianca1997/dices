use directories::ProjectDirs;
use figment::providers::{Env, Format, Serialized, Toml};
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
    pub history: Option<RelativePathBuf>,
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
                && config.try_exists().is_ok_and(|e| e)
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

impl Default for Config {
    fn default() -> Self {
        Self {
            history: directories().map(|dirs| dirs.data_dir().join("history.txt").into()),
            skin: Default::default(),
            skins: Default::default(),
        }
    }
}
