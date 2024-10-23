use axum::Router;
use derive_more::derive::{Constructor, Debug, Display, Error, From};
use dices_version::Version;
use sea_orm::{Database, DatabaseConnection, DbErr};
use serde::{Deserialize, Serialize};
use tokio::{net::TcpListener, signal};
use tower_http::trace::TraceLayer;
use tracing::{instrument, Instrument, Span};
use tracing_subscriber::field::display;
use utoipa::openapi::{Contact, Info, License, OpenApi, OpenApiBuilder};
use utoipa_swagger_ui::SwaggerUi;

mod sessions;
mod user;
mod version;
const DOMAINS: &[Domain] = &[version::DOMAIN, sessions::DOMAIN, user::DOMAIN];

pub const VERSION: Version = Version::new(
    env!("CARGO_PKG_VERSION_MAJOR"),
    env!("CARGO_PKG_VERSION_MINOR"),
    env!("CARGO_PKG_VERSION_PATCH"),
);

/// Config of the server
#[derive(Debug, Deserialize)]
pub struct Config {
    address: String,
    #[cfg_attr(not(debug_assertions), debug(skip))]
    database_url: String,
}
/// Default configs of the server
#[derive(Debug, Serialize)]
pub struct DefaultConfig {
    address: String,
}

impl Default for DefaultConfig {
    fn default() -> Self {
        Self {
            address: String::from("0.0.0.0:8080"),
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
    #[display("Fatal database error")]
    DbErr(DbErr),
}

/// Global state of the app
#[derive(Debug, Clone)]
struct AppState {
    database: DatabaseConnection,
}
impl AppState {
    #[instrument(name = "creating-appstate")]
    async fn new(config: &Config) -> Result<Self, DbErr> {
        tracing::info!("Connecting to the db");
        let database = Database::connect(&config.database_url)
            .instrument(tracing::info_span!("initial-db-connection"))
            .await?;
        Ok(Self { database })
    }
}

pub struct App {
    pub config: Config,
    router: Router<AppState>,
}

impl App {
    pub fn new(config: Config) -> Result<Self, BuildError> {
        let mut global_router = Router::new()
            .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi()));
        for Domain {
            name, version, api, ..
        } in DOMAINS
        {
            global_router = global_router.nest(&format!("/api/v{version}/{name}"), api())
        }
        global_router = global_router.layer(TraceLayer::new_for_http());
        Ok(Self {
            config,
            router: global_router,
        })
    }

    #[instrument(skip(self))]
    pub async fn serve(self) -> Result<(), FatalError> {
        let Self { config, router } = self;
        let addr = &config.address;
        // Warning if this is a debug build
        #[cfg(debug_assertions)]
        tracing::warn!("This is a debug build. As such, it is slower and unsecure. Use a release build in production");
        // Creating the global appstate
        tracing::debug!("Creating global app state");
        let router = router.with_state(AppState::new(&config).await?);
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

fn openapi() -> utoipa::openapi::OpenApi {
    let mut open_api = OpenApiBuilder::new()
        .info(
            Info::builder()
                .title("Dices Server")
                .description(Some("A server to run instances of `dices`"))
                .license(Some(License::new("MIT")))
                .contact(Some(
                    Contact::builder()
                        .name(Some("zannabianca1997"))
                        .email(Some("zannabianca199712@gmail.com"))
                        .url(Some("https://github.com/zannabianca1997/dices"))
                        .build(),
                ))
                .version(env!("CARGO_PKG_VERSION")),
        )
        .build();
    for Domain {
        name,
        version,
        api_docs,
        ..
    } in DOMAINS
    {
        let mut other = api_docs();
        other.paths.paths.iter_mut().for_each(|(_, path_item)| {
            let update_tags = |operation: Option<&mut utoipa::openapi::path::Operation>| {
                if let Some(operation) = operation {
                    let operation_tags = operation.tags.get_or_insert(Vec::new());
                    operation_tags.push(name.to_string());
                }
            };

            update_tags(path_item.get.as_mut());
            update_tags(path_item.put.as_mut());
            update_tags(path_item.post.as_mut());
            update_tags(path_item.delete.as_mut());
            update_tags(path_item.options.as_mut());
            update_tags(path_item.head.as_mut());
            update_tags(path_item.patch.as_mut());
            update_tags(path_item.trace.as_mut());
        });
        open_api = open_api.nest(format!("/api/v{version}/{name}"), other)
    }
    open_api
}

struct Domain {
    name: &'static str,
    version: u16,
    api: fn() -> Router<AppState>,
    api_docs: fn() -> OpenApi,
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
