use std::{sync::Arc, time::Duration};

use axum::{extract::FromRef, serve::WithGracefulShutdown, Router};
use derive_more::derive::{Constructor, Debug, Display, Error, From};
use dices_server_migration::MigratorTrait;
use hmac::{Hmac, Mac};
use jwt::{SigningAlgorithm, VerifyingAlgorithm};
use rand::{thread_rng, Rng};
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use tokio::{net::TcpListener, signal};
use tower_http::trace::TraceLayer;
use tracing::{instrument, Instrument};
use utoipa::openapi::{
    security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Contact, Info, License, OpenApiBuilder,
};
use utoipa_swagger_ui::SwaggerUi;

use crate::domains;

/// Config of the server
#[derive(Debug, Deserialize, Constructor)]
pub struct Config {
    address: String,
    #[cfg_attr(not(debug_assertions), debug(skip))]
    database_url: String,
    token_validity: Duration,
}
/// Default configs of the server
#[derive(Debug, Serialize)]
pub struct DefaultConfig {
    address: String,
    token_validity: Duration,
}

impl Default for DefaultConfig {
    fn default() -> Self {
        Self {
            address: String::from("0.0.0.0:8080"),
            token_validity: Duration::from_hours(1),
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

#[derive(Debug, Clone)]
pub(crate) struct AuthKey {
    key: Arc<Hmac<Sha256>>,
    token_validity: Duration,
}
impl AuthKey {
    fn generate(token_validity: Duration) -> Self {
        let secret: [u8; 32] = thread_rng().gen();
        Self {
            key: Arc::new(Hmac::new_from_slice(&secret).unwrap()),
            token_validity,
        }
    }

    pub(crate) fn token_validity(&self) -> Duration {
        self.token_validity
    }
}
impl SigningAlgorithm for AuthKey {
    fn algorithm_type(&self) -> jwt::AlgorithmType {
        SigningAlgorithm::algorithm_type(&*self.key)
    }

    fn sign(&self, header: &str, claims: &str) -> Result<String, jwt::Error> {
        self.key.sign(header, claims)
    }
}
impl VerifyingAlgorithm for AuthKey {
    fn algorithm_type(&self) -> jwt::AlgorithmType {
        VerifyingAlgorithm::algorithm_type(&*self.key)
    }

    fn verify_bytes(
        &self,
        header: &str,
        claims: &str,
        signature: &[u8],
    ) -> Result<bool, jwt::Error> {
        self.key.verify_bytes(header, claims, signature)
    }

    fn verify(&self, header: &str, claims: &str, signature: &str) -> Result<bool, jwt::Error> {
        VerifyingAlgorithm::verify(&*self.key, header, claims, signature)
    }
}

/// Global state of the app
#[derive(Debug, Clone, FromRef)]
pub(crate) struct AppState {
    database: DatabaseConnection,
    auth_key: AuthKey,
}
impl AppState {
    #[instrument(name = "initialize-appstate")]
    async fn init(config: &Config) -> Result<Self, DbErr> {
        tracing::info!("Connecting to the database");
        let options = ConnectOptions::new(&config.database_url)
            .sqlx_logging(cfg!(debug_assertions)) // do not show sql in release
            .sqlx_logging_level(tracing::log::LevelFilter::Trace) // log sql on `trace` level
            .to_owned();
        let database = Database::connect(options)
            .instrument(tracing::info_span!("initial-db-connection"))
            .await?;
        tracing::info!("Applying eventual migrations to the database");
        dices_server_migration::Migrator::up(&database, None)
            .instrument(tracing::info_span!("apply-pending-migrations"))
            .await?;
        tracing::info!("Generating auth key");
        let auth_key = AuthKey::generate(config.token_validity);
        Ok(Self { database, auth_key })
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
        for domains::Domain {
            name, version, api, ..
        } in domains::DOMAINS
        {
            global_router = global_router.nest(&format!("/api/v{version}/{name}"), api())
        }
        global_router = global_router.layer(TraceLayer::new_for_http());
        Ok(Self {
            config,
            router: global_router,
        })
    }

    pub async fn build(
        self,
    ) -> Result<
        WithGracefulShutdown<Router, Router, impl std::future::Future<Output = ()>>,
        FatalError,
    > {
        self.build_with_shutdown(shutdown_signal()).await
    }

    #[instrument(skip(self, shutdown))]
    pub async fn build_with_shutdown<S: std::future::Future<Output = ()> + Send + 'static>(
        self,
        shutdown: S,
    ) -> Result<WithGracefulShutdown<Router, Router, S>, FatalError> {
        let addr = self.config.address.clone();
        // Create the router
        let Self { config, router } = self;
        // Warning if this is a debug build
        #[cfg(debug_assertions)]
        tracing::warn!("This is a debug build. As such, it is slower and unsecure. Use a release build in production");
        // Creating the global appstate
        tracing::debug!("Creating global app state");
        let app_state = AppState::init(&config).await?;
        // Create the router
        let router = router.with_state(app_state);
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
        Ok(axum::serve(listener, router).with_graceful_shutdown(shutdown))
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
    let components = open_api.components.get_or_insert_default();
    components.add_security_scheme(
    "UserJWT",
    SecurityScheme::Http(
        HttpBuilder::new()
            .scheme(HttpAuthScheme::Bearer)
            .bearer_format("JWT").description(Some("A JWT token representing a successfull signin. Can be obtained from `/user/login` or `/user/register`. It has a limited duration, and mustbe periodically refreshed at `/user/refresh`"))
            .build(),
    ),
);
    for domains::Domain {
        name,
        version,
        api_docs,
        ..
    } in domains::DOMAINS
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
