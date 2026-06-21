use std::{ffi::OsString, path::PathBuf};

use clap::{Args, Parser};
use figment::{Figment, Provider};
use itertools::Itertools;

use crate::config::themes_dir;

/// dices TUI
///
/// A REPL to a local dices engine.
#[derive(Debug, Parser)]
#[clap(name = "dices", version)]
pub struct Cli {
    /// Configuration of the tui
    #[clap(flatten)]
    pub config: CliConfig,

    /// Set a seed for this session
    ///
    /// Initialize the random number generator with the given seed
    #[clap(long, short)]
    pub seed: Option<OsString>,

    /// Do not close after command execution.
    #[clap(long, short, requires = "command")]
    pub interactive: bool,

    #[clap(
        short = 'C',
        long,
        num_args = ..,
        trailing_var_arg = true,
        allow_hyphen_values = true
    )]
    /// Command to run
    ///
    /// If given, this command will be executed and then the tui will exit if
    /// `interactive` is not specified. Banners will be off by default.
    pub command: Option<Vec<String>>,
}

#[derive(Debug, Args)]
pub struct CliConfig {
    /// Configuration file
    ///
    /// Merged into the default config file. Config resolution is, in order: Cli
    /// arguments -> Env vars -> This file, if specified -> `Dices.toml` in
    /// the current or parent directories -> `Config.toml` in the project config
    /// directory -> Program defaults
    #[clap(long, env = "DICES_CONFIG")]
    pub config: Option<PathBuf>,

    /// Skip default configuration files
    ///
    /// Avoid searching for `Dices.toml` in the current directory or parent
    /// ones, or in the config directory, and use only defaults, env vars and
    /// cli config.
    #[clap(long)]
    pub no_default_config: bool,

    /// Override default theme
    #[clap(long, short, long_help = theme_long_help(), env = "DICES_THEME")]
    pub theme: Option<String>,

    /// Show opening and closing banners
    #[clap(long = "banners", overrides_with = "_no_banners")]
    pub _banners: bool,

    /// Hide opening and closing banners
    #[clap(long = "no-banners", overrides_with = "_banners")]
    pub _no_banners: bool,

    /// Use non-ascii characters
    #[clap(long = "graphical", overrides_with = "_no_graphical")]
    pub _graphical: bool,

    /// Use only ascii characters
    #[clap(long = "no-graphical", overrides_with = "_graphical")]
    pub _no_graphical: bool,

    /// Colorize the output
    #[clap(short = 'c', long = "color", overrides_with = "_no_color")]
    pub _color: bool,

    /// Do not colorize the output
    #[clap(long = "no-color", overrides_with = "_color")]
    pub _no_color: bool,
}

fn theme_long_help() -> String {
    let available = themes_dir()
        .and_then(|dir| dir.read_dir().ok())
        .into_iter()
        .flat_map(|dir| {
            dir.filter_map(|item| {
                item.ok().and_then(|item| {
                    let file_name = item.file_name();
                    file_name
                        .to_string_lossy()
                        .strip_suffix(".toml")
                        .map(|n| format!("\"{n}\""))
                })
            })
        })
        .format(", ");

    let dir = themes_dir()
        .map(|theme_dir| theme_dir.display().to_string())
        .unwrap_or_else(|| String::from("project config directory"));

    format!(
        "Override default theme

Available themes are {available}.

To edit or add themes change the appropriate files inside {dir}"
    )
}

impl Provider for CliConfig {
    fn metadata(&self) -> figment::Metadata {
        figment::Metadata::named("Cli arguments")
    }

    fn data(
        &self,
    ) -> Result<figment::value::Map<figment::Profile, figment::value::Dict>, figment::Error> {
        let mut data = Figment::new();

        if let Some(theme) = &self.theme {
            data = data.merge(("skin", figment::util::map!["theme" => theme]));
        }
        if let Some(banners) = self.banners() {
            data = data.merge(("skin", figment::util::map!["banners" => banners]));
        }
        if let Some(graphical) = self.graphical() {
            data = data.merge(("skin", figment::util::map!["graphical" => graphical]));
        }
        if let Some(color) = self.color() {
            data = data.merge(("skin", figment::util::map!["color" => color]));
        }

        data.data()
    }
}

impl CliConfig {
    fn banners(&self) -> Option<bool> {
        if self._banners {
            Some(true)
        } else if self._no_banners {
            Some(false)
        } else {
            None
        }
    }

    fn graphical(&self) -> Option<bool> {
        if self._graphical {
            Some(true)
        } else if self._no_graphical {
            Some(false)
        } else {
            None
        }
    }

    fn color(&self) -> Option<bool> {
        if self._color {
            Some(true)
        } else if self._no_color {
            Some(false)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use figment::Provider;

    fn parse(args: &[&str]) -> Cli {
        let mut full = vec!["dices"];
        full.extend(args);
        Cli::parse_from(full)
    }

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }

    #[test]
    fn default_no_skin_args() {
        let cli = parse(&[]);
        assert_eq!(cli.config.theme, None);
        assert_eq!(cli.config.banners(), None);
        assert_eq!(cli.config.graphical(), None);
        assert_eq!(cli.config.color(), None);
    }

    #[test]
    fn seed_flag() {
        let cli = parse(&["--seed", "abc123"]);
        assert_eq!(cli.seed.unwrap(), "abc123");
    }

    #[test]
    fn theme_flag() {
        let cli = parse(&["--theme", "dark"]);
        assert_eq!(cli.config.theme.as_deref(), Some("dark"));
    }

    #[test]
    fn banners_flag() {
        let cli = parse(&["--banners"]);
        assert_eq!(cli.config.banners(), Some(true));
        assert_eq!(cli.config._no_banners, false);
    }

    #[test]
    fn no_banners_flag() {
        let cli = parse(&["--no-banners"]);
        assert_eq!(cli.config.banners(), Some(false));
        assert_eq!(cli.config._banners, false);
    }

    #[test]
    fn banners_overrides_no_banners() {
        let cli = parse(&["--no-banners", "--banners"]);
        assert_eq!(cli.config.banners(), Some(true));
    }

    #[test]
    fn no_banners_overrides_banners() {
        let cli = parse(&["--banners", "--no-banners"]);
        assert_eq!(cli.config.banners(), Some(false));
    }

    #[test]
    fn graphical_flag() {
        let cli = parse(&["--graphical"]);
        assert_eq!(cli.config.graphical(), Some(true));
    }

    #[test]
    fn no_graphical_flag() {
        let cli = parse(&["--no-graphical"]);
        assert_eq!(cli.config.graphical(), Some(false));
    }

    #[test]
    fn color_flag() {
        let cli = parse(&["--color"]);
        assert_eq!(cli.config.color(), Some(true));
    }

    #[test]
    fn no_color_flag() {
        let cli = parse(&["--no-color"]);
        assert_eq!(cli.config.color(), Some(false));
    }

    #[test]
    fn all_skin_flags_on() {
        let cli = parse(&["--banners", "--graphical", "--color"]);
        assert_eq!(cli.config.banners(), Some(true));
        assert_eq!(cli.config.graphical(), Some(true));
        assert_eq!(cli.config.color(), Some(true));
    }

    #[test]
    fn all_skin_flags_off() {
        let cli = parse(&["--no-banners", "--no-graphical", "--no-color"]);
        assert_eq!(cli.config.banners(), Some(false));
        assert_eq!(cli.config.graphical(), Some(false));
        assert_eq!(cli.config.color(), Some(false));
    }

    fn skin_from_data(
        data: &figment::value::Map<figment::Profile, figment::value::Dict>,
    ) -> &figment::value::Dict {
        data.get(&figment::Profile::Global)
            .and_then(|d| d.get("skin"))
            .and_then(|v| v.as_dict())
            .unwrap()
    }

    #[test]
    fn provider_empty_when_no_skin_args() {
        let cli = parse(&[]);
        let data = cli.config.data().unwrap();
        let global = data.get(&figment::Profile::Global);
        assert!(global.is_none() || global.unwrap().is_empty());
    }

    #[test]
    fn provider_includes_theme() {
        let cli = parse(&["--theme", "custom"]);
        let data = cli.config.data().unwrap();
        let skin = skin_from_data(&data);
        assert_eq!(skin.get("theme").unwrap().as_str(), Some("custom"));
    }

    #[test]
    fn provider_includes_banners() {
        let cli = parse(&["--no-banners"]);
        let data = cli.config.data().unwrap();
        let skin = skin_from_data(&data);
        assert_eq!(skin.get("banners").unwrap().to_bool(), Some(false));
    }

    #[test]
    fn provider_includes_graphical() {
        let cli = parse(&["--graphical"]);
        let data = cli.config.data().unwrap();
        let skin = skin_from_data(&data);
        assert_eq!(skin.get("graphical").unwrap().to_bool(), Some(true));
    }

    #[test]
    fn provider_includes_color() {
        let cli = parse(&["--no-color"]);
        let data = cli.config.data().unwrap();
        let skin = skin_from_data(&data);
        assert_eq!(skin.get("color").unwrap().to_bool(), Some(false));
    }

    #[test]
    fn provider_includes_all_skin_flags() {
        let cli = parse(&["--banners", "--no-graphical", "--color", "--theme", "foo"]);
        let data = cli.config.data().unwrap();
        let skin = skin_from_data(&data);
        assert_eq!(skin.get("banners").unwrap().to_bool(), Some(true));
        assert_eq!(skin.get("graphical").unwrap().to_bool(), Some(false));
        assert_eq!(skin.get("color").unwrap().to_bool(), Some(true));
        assert_eq!(skin.get("theme").unwrap().as_str(), Some("foo"));
    }

    #[test]
    fn config_file_flag() {
        let cli = parse(&["--config", "/some/path/config.toml"]);
        assert_eq!(
            cli.config.config.unwrap(),
            PathBuf::from("/some/path/config.toml")
        );
    }

    #[test]
    fn no_default_config_flag() {
        let cli = parse(&["--no-default-config"]);
        assert!(cli.config.no_default_config);
    }
}
