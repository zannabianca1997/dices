use std::{future::Future, io};

use axum::{extract::FromRef, routing::IntoMakeService, Router};
use futures::{join, try_join, FutureExt, TryFutureExt};
use sea_orm::{Database, DbErr};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use dices_server_auth::{AuthConfig, AuthKey};
use dices_server_migration::{sea_orm::DatabaseConnection, Migrator};

mod connection;
pub use connection::ConnectOptions;
use tokio::{
    net::{unix::SocketAddr, TcpListener, ToSocketAddrs},
    signal,
};
use tower_http::trace::TraceLayer;

#[derive(Debug, Deserialize, Default, Serialize)]
/// Configuration of the app
pub struct AppConfig {
    /// Config of the security
    pub auth: AuthConfig,

    /// Config of the database connection
    pub conn: ConnectOptions,
}
#[derive(Debug, Deserialize, Default, Serialize)]
/// Configuration of the app for serving it
pub struct ServeConfig {
    /// Config of the app
    #[serde(flatten)]
    pub app: AppConfig,

    /// Socket address
    pub socket: SocketConfig,
}

#[derive(Debug, Deserialize, Serialize)]
/// Address to bind the app to
#[serde(untagged)]
pub enum SocketConfig {
    Compact(String),
    Large {
        /// The ip
        ip: String,
        // The port
        port: u16,
    },
}
impl Default for SocketConfig {
    fn default() -> Self {
        Self::Large {
            ip: "127.0.0.1".to_owned(),
            port: 8080,
        }
    }
}

#[derive(Debug, Error)]
pub enum BuildError {
    #[error("Error in connecting to the database")]
    DbErr(
        #[source]
        #[from]
        DbErr,
    ),
}
#[derive(Debug, Error)]
pub enum FatalError {
    #[error("Error during input/output")]
    Io(
        #[from]
        #[source]
        io::Error,
    ),
}

#[derive(Debug, Clone, FromRef)]
pub struct App {
    auth_key: AuthKey,
    database_connection: DatabaseConnection,
}

impl App {
    /// Build the app as a router
    pub async fn build(AppConfig { auth, conn }: AppConfig) -> Result<Self, BuildError> {
        let auth_key = AuthKey::new(auth);

        tracing::info!("Connecting to the database");
        let database_connection = Database::connect(conn).await?;

        Ok(Self {
            auth_key,
            database_connection,
        })
    }

    /// Serve the app with the given graceful shutdown
    pub(crate) async fn serve_with_graceful_shutdown(
        self,
        socket: SocketConfig,
        shutdown: impl Future<Output = ()> + Send + 'static,
    ) -> Result<(), FatalError> {
        let service = router().with_state(self).layer(TraceLayer::new_for_http());

        let tcp_listener = match socket {
            SocketConfig::Compact(socket) => TcpListener::bind(socket).await?,
            SocketConfig::Large { ip, port } => TcpListener::bind((ip, port)).await?,
        };

        axum::serve(tcp_listener, service)
            .with_graceful_shutdown(shutdown)
            .await?;

        Ok(())
    }
    /// Serve the app until `Ctrl-C` or terminate signal
    pub(crate) async fn serve(self, socket: SocketConfig) -> Result<(), FatalError> {
        self.serve_with_graceful_shutdown(socket, shutdown_signal())
            .await
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    };
    tracing::info!("Received shutdown signal")
}

fn router() -> Router<App> {
    Router::new().nest("/user", super::user::router())
}
