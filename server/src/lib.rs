use std::{io, path::PathBuf};

use app::ServeConfig;
use clap::Parser;
use config::{Config, ConfigArgs};
use thiserror::Error;
use tokio::fs;
use tracing_config::config::ArcMutexGuard;

mod app;
mod config;
mod logging;
mod user;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
/// Run a server hosting sessions of `dices`
pub struct CliArgs {
    #[clap(long)]
    /// Emit the configs to a file and exit.
    ///
    /// If no other arguments is given, this will create an example config.
    example_config: Option<PathBuf>,
    #[clap(flatten)]
    config: ConfigArgs,
}

#[derive(Debug, Error)]
pub enum MainError {
    #[error("Error in configuring the server")]
    Config(
        #[source]
        #[from]
        figment::Error,
    ),
    #[error("Error in configuring logging")]
    Logging(
        #[source]
        #[from]
        tracing_config::TracingConfigError,
    ),
    #[error("Error in writing example configuration file")]
    WriteExampleConf(#[source] io::Error),
    #[error("Error in building app")]
    Build(#[from] app::BuildError),
    #[error("Error in running app")]
    Fatal(#[from] app::FatalError),
}

/// Run the server.
///
/// # Panics
/// This function may panic if tracing is used after it has runned
pub async fn main(
    CliArgs {
        config: config_args,
        example_config,
    }: CliArgs,
) -> Result<ArcMutexGuard, MainError> {
    // Read configuration
    let config = config::configure(config_args)?;
    // Setup logging
    let log_guard = logging::init(&config.logging)?;
    // Print example config
    if let Some(example_config) = example_config {
        tracing::info!("Printing configs to {}", example_config.display());
        let toml = toml::to_string_pretty(&config)
            .expect("The serialization of the configs should be infallible");
        fs::write(example_config, toml)
            .await
            .map_err(MainError::WriteExampleConf)?;
        return Ok(log_guard);
    }

    // Run the app
    let Config {
        serve:
            ServeConfig {
                app: app_config,
                socket,
                banner,
            },
        logging: _,
    } = config;

    if banner {
        banner::banner();
    }

    app::App::build(app_config).await?.serve(socket).await?;

    Ok(log_guard)
}

pub mod banner;
