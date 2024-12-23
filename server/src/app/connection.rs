use std::{
    borrow::Cow,
    net::{IpAddr, Ipv4Addr},
    time::Duration,
};

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none, DisplayFromStr, DurationSeconds};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ConnectOptions {
    Url(String),
    Large(ConnectOptionsLarge),
}

impl Default for ConnectOptions {
    fn default() -> Self {
        Self::Large(ConnectOptionsLarge::default())
    }
}

impl From<ConnectOptions> for sea_orm::ConnectOptions {
    fn from(value: ConnectOptions) -> Self {
        match value {
            ConnectOptions::Url(url) => sea_orm::ConnectOptions::new(url),
            ConnectOptions::Large(connect_options_large) => connect_options_large.into(),
        }
    }
}

fn default_values() -> String {
    String::from("dices_server")
}
fn default_ip() -> IpAddr {
    IpAddr::V4(Ipv4Addr::LOCALHOST)
}
fn default_port() -> u16 {
    5432
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UrlOrParts {
    Url {
        /// The URI of the database
        url: String,
    },
    Parts {
        #[serde(default = "default_values")]
        username: String,
        #[serde(default = "default_values")]
        password: String,
        #[serde(default = "default_ip")]
        ip: IpAddr,
        #[serde(default = "default_port")]
        port: u16,
        #[serde(default = "default_values")]
        database: String,
    },
}
impl Default for UrlOrParts {
    fn default() -> Self {
        Self::Parts {
            username: default_values(),
            password: default_values(),
            ip: default_ip(),
            port: default_port(),
            database: default_values(),
        }
    }
}
impl UrlOrParts {
    fn url(self) -> String {
        match self {
            UrlOrParts::Url { url } => url,
            UrlOrParts::Parts {
                username,
                password,
                ip,
                port,
                database,
            } => format!("postgres:{username}:{password}@{ip}:{port}/{database}"),
        }
    }
}

#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectOptionsLarge {
    #[serde(flatten)]
    pub url: UrlOrParts,
    /// Maximum number of connections for a pool
    pub max_connections: Option<u32>,
    /// Minimum number of connections for a pool
    pub min_connections: Option<u32>,
    /// The connection timeout for a packet connection
    pub connect_timeout: Option<Duration>,
    /// Maximum idle time for a particular connection to prevent
    /// network resource exhaustion
    #[serde_as(as = "Option<DurationSeconds>")]
    pub idle_timeout: Option<Duration>,
    /// Set the maximum amount of time to spend waiting for acquiring a connection
    #[serde_as(as = "Option<DurationSeconds>")]
    pub acquire_timeout: Option<Duration>,
    /// Set the maximum lifetime of individual connections
    #[serde_as(as = "Option<DurationSeconds>")]
    pub max_lifetime: Option<Duration>,
    /// Enable SQLx statement logging
    pub sqlx_logging: bool,
    /// SQLx statement logging level (ignored if `sqlx_logging` is false)
    #[serde_as(as = "DisplayFromStr")]
    pub sqlx_logging_level: log::LevelFilter,
    /// SQLx slow statements logging level (ignored if `sqlx_logging` is false)
    #[serde_as(as = "DisplayFromStr")]
    pub sqlx_slow_statements_logging_level: log::LevelFilter,
    /// SQLx slow statements duration threshold (ignored if `sqlx_logging` is false)
    #[serde_as(as = "DurationSeconds")]
    pub sqlx_slow_statements_logging_threshold: Duration,
    /// set sqlcipher key
    pub sqlcipher_key: Option<Cow<'static, str>>,
    /// Schema search path
    pub schema_search_path: Option<String>,
    pub test_before_acquire: bool,
    /// Only establish connections to the DB as needed. If set to `true`, the db connection will
    /// be created using SQLx's [connect_lazy](https://docs.rs/sqlx/latest/sqlx/struct.Pool.html#method.connect_lazy)
    /// method.
    pub connect_lazy: bool,
}
impl From<ConnectOptionsLarge> for sea_orm::ConnectOptions {
    fn from(
        ConnectOptionsLarge {
            url,
            max_connections,
            min_connections,
            connect_timeout,
            idle_timeout,
            acquire_timeout,
            max_lifetime,
            sqlx_logging,
            sqlx_logging_level,
            sqlx_slow_statements_logging_level,
            sqlx_slow_statements_logging_threshold,
            sqlcipher_key,
            schema_search_path,
            test_before_acquire,
            connect_lazy,
        }: ConnectOptionsLarge,
    ) -> Self {
        let mut opt = sea_orm::ConnectOptions::new(url.url());
        if let Some(max_connections) = max_connections {
            opt.max_connections(max_connections);
        }
        if let Some(min_connections) = min_connections {
            opt.min_connections(min_connections);
        }
        if let Some(connect_timeout) = connect_timeout {
            opt.connect_timeout(connect_timeout);
        }
        if let Some(idle_timeout) = idle_timeout {
            opt.idle_timeout(idle_timeout);
        }
        if let Some(acquire_timeout) = acquire_timeout {
            opt.acquire_timeout(acquire_timeout);
        }
        if let Some(max_lifetime) = max_lifetime {
            opt.max_lifetime(max_lifetime);
        }
        if let Some(min_connections) = min_connections {
            opt.min_connections(min_connections);
        }
        opt.sqlx_logging(sqlx_logging)
            .sqlx_logging_level(sqlx_logging_level)
            .sqlx_slow_statements_logging_settings(
                sqlx_slow_statements_logging_level,
                sqlx_slow_statements_logging_threshold,
            );
        if let Some(sqlcipher_key) = sqlcipher_key {
            opt.sqlcipher_key(sqlcipher_key);
        }
        if let Some(schema_search_path) = schema_search_path {
            opt.set_schema_search_path(schema_search_path);
        }
        opt.test_before_acquire(test_before_acquire)
            .connect_lazy(connect_lazy);
        opt
    }
}

impl Default for ConnectOptionsLarge {
    fn default() -> Self {
        Self {
            url: Default::default(),
            max_connections: None,
            min_connections: None,
            connect_timeout: None,
            idle_timeout: None,
            acquire_timeout: None,
            max_lifetime: None,
            sqlx_logging: true,
            sqlx_logging_level: log::LevelFilter::Info,
            sqlx_slow_statements_logging_level: log::LevelFilter::Off,
            sqlx_slow_statements_logging_threshold: Duration::from_secs(1),
            sqlcipher_key: None,
            schema_search_path: None,
            test_before_acquire: true,
            connect_lazy: false,
        }
    }
}
