use std::path::PathBuf;

use clap::Args;
use figment::{
    providers::{Env, Format, Serialized, Toml},
    value::Tag,
    Figment, Metadata, Provider,
};
use serde::{de::Error, Deserialize, Serialize};
use tracing_config::config::model::TracingConfig;

use crate::{app::ServeConfig, logging::default_tracing_config};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(flatten)]
    /// General app configs
    pub serve: ServeConfig,
    /// Configs about loggings
    pub logging: TracingConfig,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            serve: Default::default(),
            logging: default_tracing_config(),
        }
    }
}

#[derive(Debug, Args)]
pub struct ConfigArgs {
    #[clap(short, long)]
    /// Config file to load
    config_file: Option<PathBuf>,
    #[clap(long)]
    /// Do not search for the default config file
    no_default_config_file: bool,
    #[clap(long)]
    /// Do not load enviroment variables
    no_env: bool,
    #[clap(long)]
    /// Do not use defaults, load every config from other sources
    no_defaults: bool,

    #[clap(short = 'C')]
    /// Command line configuration
    ///
    /// Formatted as `conf.name=value`. Will overwrite any other source.
    configs: Vec<String>,
}
impl Provider for ConfigArgs {
    fn metadata(&self) -> figment::Metadata {
        Metadata::named("command line arguments")
    }

    fn data(
        &self,
    ) -> Result<figment::value::Map<figment::Profile, figment::value::Dict>, figment::Error> {
        let mut data = figment::value::Dict::new();

        for item in &self.configs {
            let (n, v) = item.split_once('=').ok_or_else(|| {
                figment::Error::custom(format!(
                    "Invalid configuration {item}: expected string of the type `conf.name=value`"
                ))
            })?;
            let mut data = &mut data;
            let mut components = n.split('.');
            let name = components.next_back().unwrap();
            for component in components {
                let figment::value::Value::Dict(_, new_data) =
                    data.entry(component.to_string()).or_insert_with(|| {
                        figment::value::Value::Dict(Tag::Default, Default::default())
                    })
                else {
                    unreachable!()
                };
                data = new_data;
            }

            data.insert(name.to_owned(), v.parse().unwrap());
        }

        Ok(figment::value::Map::from_iter([(
            figment::Profile::Global,
            data,
        )]))
    }
}

/// Build the figment from the multiple configuration sources
fn figment(config_args: ConfigArgs) -> Figment {
    // First, the defaults values
    let mut figment = if !config_args.no_defaults {
        Figment::from(Serialized::defaults(Config::default()))
    } else {
        Figment::new()
    };
    // Then the default config file
    if !config_args.no_default_config_file {
        figment = figment.merge(Toml::file("DicesServer.toml"));
    }
    // Then the one provided by the user
    if let Some(config_file) = &config_args.config_file {
        figment = figment.merge(Toml::file_exact(config_file));
    }
    // Then, the enviroment variables and the arguments
    if !config_args.no_env {
        match dotenv::dotenv() {
            Ok(_) => (),
            Err(err) if err.not_found() => (),
            Err(err) => eprintln!("Cannot open `.env` to load enviroment variable: {err}"),
        };
        figment = figment.merge(Env::raw().split("__").global());
    }
    // Finally the cli arguments
    figment.merge(config_args)
}

pub fn configure(config_args: ConfigArgs) -> figment::Result<Config> {
    figment(config_args).extract()
}
