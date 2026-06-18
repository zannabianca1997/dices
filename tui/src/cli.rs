use std::{ffi::OsString, path::PathBuf};

use clap::{Args, Parser};

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
}

#[cfg(test)]
mod tests {
    use super::Cli;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
