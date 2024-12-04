#![feature(error_reporter)]

use std::{error::Report, io, path::PathBuf};

use clap::{Args, Parser};
use derive_more::derive::{Display, Error, From};
use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};
use tracing::level_filters::LevelFilter;

use dices_server::{App, DefaultConfig};

#[derive(Debug, Parser)]
struct Cli {
    /// File for the default options for the server
    #[clap(long = "setup", short = 'S')]
    file_setup: Option<PathBuf>,
    /// Enviroment variable file
    #[clap(long)]
    dot_env: Option<PathBuf>,

    #[clap(flatten)]
    cli_setup: ConfigCli,
}
/// Config of the server for the cli
#[derive(Debug, Serialize, Args)]
struct ConfigCli {
    /// The address to listen to
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(long, short)]
    address: Option<String>,

    /// The database URL
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(long, short)]
    database_url: Option<String>,

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
    #[display("Error in loading .env file")]
    DotEnv(dotenvy::Error),
}

fn main_impl() -> Result<(), MainError> {
    // Load all configs
    let config = make_figment(Cli::parse())?;
    // Setup logging
    setup_logging(config.extract_inner("logging")?)?;
    // Configure app
    let app = App::new(config.extract()?)?;
    // Build runtime
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(MainError::Runtime)?;
    // Serve the app
    runtime.block_on(async {
        let app = app.build().await?;
        app.await.map_err(MainError::Runtime)?;
        Ok::<(), MainError>(())
    })?;
    // Graceful exit
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
            .map_err(MainError::InvalidLogLevel)?,
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
    Cli {
        file_setup,
        cli_setup,
        dot_env,
    }: Cli,
) -> Result<Figment, MainError> {
    // Load .env
    match dot_env {
        Some(dot_env) => match dotenvy::from_path(dot_env) {
            Ok(()) => (),
            Err(err) => return Err(MainError::DotEnv(err)),
        },
        None => match dotenvy::dotenv() {
            Ok(_) => (),
            Err(err) if err.not_found() => (),
            Err(err) => return Err(MainError::DotEnv(err)),
        },
    }
    let mut config = Figment::new()
        .merge(Serialized::defaults(DefaultConfig::default()))
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
    config = config
        .merge(Env::raw())
        .merge(Serialized::defaults(cli_setup));
    Ok(config)
}

fn main() -> Result<(), Report<MainError>> {
    main_impl().map_err(|err| Report::new(err).pretty(true))
}
