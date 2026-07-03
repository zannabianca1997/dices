use std::path::PathBuf;

use dices_std::StdOptions;
use directories::ProjectDirs;
use figment::Figment;
use figment::providers::{Env, Format, Serialized, Toml};
use serde::{Deserialize, Serialize};

use crate::cli::CliConfig;
use crate::config::skin::Skin;
use history::HistoryConfig;

pub mod history;
pub mod skin;
pub mod theme;

fn directories() -> Option<ProjectDirs> {
    ProjectDirs::from("site.zannabianca1997.dices", "", "dices")
}

pub fn config_file() -> Option<PathBuf> {
    Some(directories()?.config_dir().join("Config.toml"))
}

pub fn themes_dir() -> Option<PathBuf> {
    Some(directories()?.config_dir().join("themes"))
}

pub fn banners_dir() -> Option<PathBuf> {
    Some(directories()?.config_dir().join("banners"))
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub history: HistoryConfig,
    pub skin: Skin,
    pub std: StdOptions,
}

impl Config {
    pub fn extract(cli: CliConfig, command_given: bool) -> Result<Self, figment::Error> {
        let mut defaults = Config::default();
        if command_given {
            defaults.skin.banners = false;
        }

        let mut figment = Figment::new().merge(Serialized::defaults(defaults));

        if !cli.no_default_config {
            if let Some(config) = config_file()
                && config.exists()
            {
                figment = figment.merge(Toml::file_exact(config));
            }
            figment = figment.merge(Toml::file("Dices.toml"));
        }

        if let Some(path) = cli.config.as_ref() {
            figment = figment.merge(Toml::file_exact(path));
        }

        figment = figment.merge(Env::prefixed("DICES_").split("_")).merge(cli);

        let config: Self = figment.extract()?;
        Ok(config)
    }
}
