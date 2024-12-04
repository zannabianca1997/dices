use std::io;

pub use cli::Cli;
use derive_more::derive::{Display, Error, From};
use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};
use serde::Serialize;

use crate::{App, DefaultConfig};

mod cli;
mod logging;

#[derive(Debug, Display, Error, From)]
pub enum MainError {
    #[display("Error in configuring server")]
    Setup(figment::Error),
    #[display("Error in configuring server logging")]
    LoggingSetup(logging::LoggingSetupError),
    #[display("Error while initializing tokio runtime")]
    #[from(skip)]
    Runtime(io::Error),
    #[display("Fatal error while building the app")]
    Build(crate::BuildError),
    #[display("Fatal error during running")]
    Fatal(crate::FatalError),
    #[display("Error in loading .env file")]
    DotEnv(dotenvy::Error),
}

pub fn main(cli: Cli) -> Result<(), MainError> {
    // Load all configs
    let config = build_config_figment(cli)?;
    // Setup logging
    logging::setup_logging(config.extract_inner("logging")?)?;
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

fn build_config_figment(
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
                logging: logging::LoggingConfig,
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