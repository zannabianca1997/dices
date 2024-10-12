use axum::Router;
use derive_more::derive::{Constructor, Display, Error, From};
use dices_ast::version::Version;
use serde::{Deserialize, Serialize};
use tokio::{net::TcpListener, signal};
use tower_http::trace::TraceLayer;
use tracing::{info_span, instrument, Instrument};

mod common;

pub const VERSION: Version = Version::new(
    env!("CARGO_PKG_VERSION_MAJOR"),
    env!("CARGO_PKG_VERSION_MINOR"),
    env!("CARGO_PKG_VERSION_PATCH"),
);

/// Config of the server
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    addr: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            addr: String::from("0.0.0.0:8080"),
        }
    }
}

/// An unrecoverable error while building the server
#[derive(Debug, Display, Error, From)]
pub enum BuildError {}

/// An unrecoverable server error
#[derive(Debug, Display, Error, From)]
pub enum FatalError {
    #[display("Cannot bind to the address `{addr}`")]
    BindAddress {
        source: std::io::Error,
        addr: String,
    },
    #[display("Fatal io error while serving")]
    IOError(std::io::Error),
}

/// Global state of the app
#[derive(Debug, Constructor, Serialize, Deserialize, Clone)]
struct AppState {}

pub struct App {
    pub config: Config,
    router: Router<AppState>,
}

impl App {
    pub fn new(config: Config) -> Result<Self, BuildError> {
        Ok(Self {
            config,
            router: Router::new()
                .merge(common::router())
                .layer(TraceLayer::new_for_http()),
        })
    }

    #[instrument(skip(self))]
    pub async fn serve(self) -> Result<(), FatalError> {
        let Self {
            config: Config { addr },
            router,
        } = self;
        // Creating the global appstate
        tracing::debug!("Creating global app state.");
        let router = router.with_state(AppState::new());
        // Bindind to the port
        tracing::debug!(addr, "Creating the listener");
        let listener =
            TcpListener::bind(&addr)
                .await
                .map_err(|source| FatalError::BindAddress {
                    source,
                    addr: addr.clone(),
                })?;
        // Serve the app
        tracing::info!(addr, "Start serving the app");
        axum::serve(listener, router)
            .with_graceful_shutdown(shutdown_signal())
            .await?;
        // Gracefully exited
        tracing::info!("Exiting...");
        Ok(())
    }
}

#[instrument]
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
