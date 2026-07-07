use std::path::PathBuf;
use std::sync::Arc;

use dices_std::StdOptions;
use directories::ProjectDirs;
use figment::Figment;
use figment::providers::{Env, Format, Serialized, Toml};
use serde::{Deserialize, Serialize};

use crate::cli::CliConfig;
use crate::config::man::ManConfig;
use crate::config::skin::Skin;
use history::HistoryConfig;

pub mod history;
pub mod man;
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
    pub skin: Arc<Skin>,
    pub std: StdOptions,
    pub man: ManConfig,
}

impl Config {
    pub fn extract(cli: CliConfig, closes_soon: bool) -> Result<Self, figment::Error> {
        let mut figment = Figment::new().merge(Serialized::defaults(Config::default()));

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

        figment = figment.merge(Env::prefixed("DICES_").split("_"));

        // If cli will run a single command, disable banners and links
        if closes_soon {
            figment = figment.merge(("skin", figment::util::nest("banners", false.into())));
            figment = figment.merge(("man", figment::util::nest("links", false.into())));
        }

        figment = figment.merge(cli);

        let config: Self = figment.extract()?;
        Ok(config)
    }
}
