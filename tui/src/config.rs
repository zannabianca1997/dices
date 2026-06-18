use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

use directories::ProjectDirs;
use figment::Figment;
use figment::providers::{Env, Format, Serialized, Toml};
use serde::{Deserialize, Serialize};

use crate::cli::CliConfig;
use crate::config::skin::Skin;
use history::HistoryConfig;

pub mod history;
pub mod skin;

fn directories() -> Option<ProjectDirs> {
    ProjectDirs::from("site.zannabianca1997.dices", "", "dices")
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// History file
    pub history: HistoryConfig,
    /// Graphical skin
    pub skin: Skin,
}

impl Config {
    pub fn extract(
        CliConfig {
            config,
            no_default_config,
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

        let config: Self = figment.extract()?;
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
            history: Default::default(),
            skin: Skin::default(),
        }
    }
}
