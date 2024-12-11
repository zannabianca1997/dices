use clap::Args;
use derive_more::derive::{Display, Error, From};
use serde::{Deserialize, Serialize};
use tracing::level_filters::LevelFilter;

#[derive(Debug, Display, Error, From)]
pub enum LoggingSetupError {
    #[display("Cannot parse log level")]
    InvalidLogLevel(tracing_subscriber::filter::LevelParseError),
}

pub(crate) fn setup_logging(
    LoggingConfig {
        pretty,
        level: level_filter,
    }: LoggingConfig,
) -> Result<(), LoggingSetupError> {
    let subscriber = tracing_subscriber::fmt().with_max_level(
        level_filter
            .parse::<LevelFilter>()
            .map_err(LoggingSetupError::InvalidLogLevel)?,
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
pub(super) struct LoggingConfigCli {
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
pub(super) struct LoggingConfig {
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
