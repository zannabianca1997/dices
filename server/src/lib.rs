use std::{io, path::PathBuf};

use clap::Parser;
use config::ConfigArgs;
use dices_version::Version;
use thiserror::Error;
use tokio::fs;
use tracing_config::config::ArcMutexGuard;

pub use dices_server_auth as auth;
pub use dices_server_dtos as dtos;
pub use dices_server_entities as entities;
pub use dices_server_migration as migration;

pub mod app;
mod banner;
pub mod config;
mod domains;
mod logging;

pub use app::{App, AppConfig, ServeConfig};
pub use banner::banner;
pub use config::Config;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
/// Run a server hosting sessions of `dices`
pub struct CliArgs {
    #[clap(long)]
    /// Emit the configs to a file and exit.
    ///
    /// If no other arguments is given, this will create an example config.
    pub example_config: Option<PathBuf>,
    #[clap(flatten)]
    pub config: ConfigArgs,
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

pub const VERSION: Version = Version::new(
    env!("CARGO_PKG_VERSION_MAJOR"),
    env!("CARGO_PKG_VERSION_MINOR"),
    env!("CARGO_PKG_VERSION_PATCH"),
);
