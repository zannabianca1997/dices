#![feature(error_reporter)]

use std::{error::Report, io, path::PathBuf};

use clap::{Args, Parser};
use derive_more::derive::{Display, Error, From};
use dices_server::{App, Config};
use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment, Provider,
};
use serde::{Deserialize, Serialize};
use tracing::{instrument, level_filters::LevelFilter};

#[derive(Debug, Parser)]
struct CLI {
    /// File for the default options for the server
    #[clap(long = "setup", short = 'S')]
    file_setup: Option<PathBuf>,

    #[clap(flatten)]
    cli_setup: ConfigCli,
}
/// Config of the server for the cli
#[derive(Debug, Serialize, Args)]
pub struct ConfigCli {
    /// The address to listen to
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(long, short)]
    addr: Option<String>,

    /// The logging setups
    #[clap(flatten)]
    logging: LoggingConfigCli,
}

#[derive(Debug, Display, Error, From)]
enum MainError {
    #[display("Error in configuring server")]
    Setup(figment::Error),
    #[display("Error while initializing tokio runtime")]
    #[from(skip)]
    Runtime(io::Error),
    #[display("Fatal error while building the app")]
    Build(dices_server::BuildError),
    #[display("Fatal error during running")]
    Fatal(dices_server::FatalError),
    #[display("Cannot parse log level")]
    InvalidLogLevel(tracing_subscriber::filter::LevelParseError),
}

fn main() -> Result<(), Report<MainError>> {
    main_impl().map_err(|err| Report::new(err).pretty(true))
}

fn main_impl() -> Result<(), MainError> {
    let config = make_figment(CLI::parse());

    setup_logging(config.extract_inner("logging")?)?;

    let app = App::new(config.extract()?)?;

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(MainError::Runtime)?
        .block_on(app.serve())?;

    Ok(())
}

fn setup_logging(
    LoggingConfig {
        pretty,
        level: level_filter,
    }: LoggingConfig,
) -> Result<(), MainError> {
    let subscriber = tracing_subscriber::fmt().with_max_level(
        level_filter
            .parse::<LevelFilter>()
            .map_err(|err| MainError::InvalidLogLevel(err))?,
    );
    if pretty {
        tracing::subscriber::set_global_default(subscriber.pretty().finish())
    } else {
        tracing::subscriber::set_global_default(subscriber.finish())
    }
    .expect("The global subscriber should not have been set.");
    Ok(())
}

#[derive(Debug, Serialize, Args)]
struct LoggingConfigCli {
    /// If logging to console use pretty format
    #[serde(skip_serializing_if = "Option::is_none", rename = "pretty")]
    #[clap(long)]
    logging_pretty: Option<bool>,
    /// The minimum log level
    #[serde(skip_serializing_if = "Option::is_none", rename = "level")]
    #[clap(long)]
    logging_level: Option<String>,
}
#[derive(Debug, Deserialize, Serialize)]
struct LoggingConfig {
    pretty: bool,
    level: String,
}
impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            pretty: false,
            level: if cfg!(debug_assertions) {
                LevelFilter::DEBUG
            } else {
                LevelFilter::INFO
            }
            .to_string(),
        }
    }
}

fn make_figment(
    CLI {
        file_setup,
        cli_setup,
    }: CLI,
) -> Figment {
    let mut config = Figment::new()
        .merge(Serialized::defaults(Config::default()))
        .merge(Serialized::defaults({
            #[derive(Serialize, Default)]
            struct Nested {
                logging: LoggingConfig,
            }
            Nested::default()
        }))
        .merge(Toml::file("./DicesServer.toml"));
    if let Some(file_setup) = file_setup {
        config = config.merge(Toml::file_exact(file_setup))
    }
    config
        .merge(Env::prefixed("DICES_SERVER_"))
        .merge(Serialized::defaults(cli_setup))
}
