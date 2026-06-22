use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

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

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    /// History file
    pub history: HistoryConfig,
    /// Graphical skin
    pub skin: Skin,
    /// Options for the standard library
    pub std: StdOptions,
}

impl Config {
    pub fn extract(cli: CliConfig, command_given: bool) -> Result<Self, figment::Error> {
        let mut defaults = Config::default();
        if command_given {
            defaults.skin.banners = false;
        }

        let mut figment = Figment::new().merge(Serialized::defaults(defaults));

        // Write down the default theme
        if let Some(theme_dirs) = themes_dir() {
            let _ = theme::write_themes_if_not_exists(&theme_dirs);
        }

        if !cli.no_default_config {
            if let Some(config) = config_file()
                && write_config_file_if_not_exists(&config).is_ok()
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

fn write_config_file_if_not_exists(config: &Path) -> io::Result<()> {
    fs::create_dir_all(config.parent().unwrap())?;
    let mut file = match File::create_new(config) {
        Ok(file) => file,
        Err(err) if err.kind() == io::ErrorKind::AlreadyExists => return Ok(()),
        Err(err) => return Err(err),
    };
    let config = r#"# Dices config file
# This can be overridden with a `Dices.toml` in the current
# or parent directory, or with env variables

[history]
# file = "alternate/history/file" # Database for the history
# capacity = 1000                 # History capacity

[skin]
# theme     = "CatppuccinMocha"  # Default theme (from the `themes` directory)
# banners   = true               # Show banners
# graphical = true               # Use unicode characters
# color     = true               # Colorize output
"#;

    file.write_all(config.as_bytes())
}
