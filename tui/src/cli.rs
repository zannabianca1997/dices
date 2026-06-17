use std::{ffi::OsString, path::PathBuf};

use clap::{Args, Parser};

use crate::config::skin::{SelectedSkin, SkinValueParser};

/// dices TUI
///
/// A REPL to a local dices engine.
#[derive(Debug, Parser)]
#[clap(name = "dices", version)]
pub struct Cli {
    /// Set a seed for this session
    ///
    /// Initialize the random number generator with the given seed
    #[clap(long, short)]
    pub seed: Option<OsString>,
    #[clap(flatten)]
    pub config: CliConfig,
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
    /// ones, or in the config directory
    #[clap(long)]
    pub no_default_config: bool,
    /// Set a skin for this session
    ///
    /// Custom skin can be set in the config file under `skins`
    #[clap(long, env = "DICES_SKIN", value_parser = SkinValueParser, default_value_t)]
    pub skin: SelectedSkin,
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use crate::{cli::CliConfig, config::skin::SelectedSkin};

    use super::Cli;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }

    #[test]
    fn custom_skin_is_supported() {
        let Cli {
            config: CliConfig { skin, .. },
            ..
        } = Cli::parse_from(["dices", "--skin", "my-skin"]);
        assert!(matches!(skin, SelectedSkin::Other(s) if s == "my-skin"));
    }

    #[test]
    fn builtin_skins_are_supported() {
        let Cli {
            config: CliConfig { skin, .. },
            ..
        } = Cli::parse_from(["dices", "--skin", "ascii"]);
        assert!(matches!(skin, SelectedSkin::Ascii));
    }

    #[test]
    fn help_lists_skin_variants() {
        use clap::CommandFactory;
        let mut output = Vec::new();
        Cli::command().write_long_help(&mut output).unwrap();
        let help = String::from_utf8(output).unwrap();
        assert!(help.contains("- ascii"), "help does not mention ascii");
        assert!(help.contains("- fancy"), "help does not mention fancy");
        assert!(help.contains("- (other)"), "help does not mention (other)");
    }
}
