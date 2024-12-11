use std::path::PathBuf;

use clap::{Args, Parser};
use serde::Serialize;

#[derive(Debug, Parser)]
pub struct Cli {
    /// File for the default options for the server
    #[clap(long = "setup", short = 'S')]
    pub(super) file_setup: Option<PathBuf>,
    /// Enviroment variable file
    #[clap(long)]
    pub(super) dot_env: Option<PathBuf>,

    #[clap(flatten)]
    pub(super) cli_setup: ConfigCli,
}
/// Config of the server for the cli
#[derive(Debug, Serialize, Args)]
pub(super) struct ConfigCli {
    /// The address to listen to
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(long, short)]
    pub(super) address: Option<String>,

    /// The database URL
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(long, short)]
    pub(super) database_url: Option<String>,

    /// The logging setups
    #[clap(flatten)]
    pub(super) logging: super::logging::LoggingConfigCli,
}
