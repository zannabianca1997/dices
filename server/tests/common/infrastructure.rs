use std::borrow::Borrow;
use std::future::Future;
use std::panic::{resume_unwind, AssertUnwindSafe};
use std::pin::Pin;

use axum_test::TestServer;
use dices_server_dtos::user::token::UserToken;
use dices_server_entities::session::SessionId;
use dices_server_entities::user::UserId;
use futures::FutureExt;
use serde_json::json;
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{runners::AsyncRunner, ContainerAsync, ImageExt},
};
use tracing::instrument;

use dices_server::app::ConnectOptions;
use dices_server::{App, AppConfig};

pub struct Infrastructure {
    db: ContainerAsync<Postgres>,
    server: TestServer,
}
#[allow(unused)]
impl Infrastructure {
    /// Run a piece of code with the infrastructure running
    ///
    /// This is in a callback so the teardown code runs automatically at the end
    pub async fn with<F, T>(callback: F) -> T
    where
        F: for<'c> FnOnce(&'c Self) -> Pin<Box<dyn Future<Output = T> + 'c>>,
    {
        let infrastructure = Self::up().await;

        let res = AssertUnwindSafe(callback(&infrastructure))
            .catch_unwind()
            .await;

        Self::down(infrastructure).await;

        match res {
            Ok(t) => t,
            Err(e) => resume_unwind(e),
        }
    }

    /// Pull up the infrastructure
    async fn up() -> Self {
        let db = db().await;
        let connection_string = format!(
            "postgres://dices_server_test:dices_server_test@{}:{}/dices_server_test",
            db.get_host().await.unwrap(),
            db.get_host_port_ipv4(5432).await.unwrap()
        );
        let server = server(connection_string).await;
        Self { db, server }
    }

    /// Close the infrastructure
    async fn down(infrastructure: Infrastructure) {
        let Self { db, server } = infrastructure;
        // drop the server
        drop(server);
        // stop the database
        db.stop().await.expect("Error in stopping postgres");
        // remove the database container
        db.rm().await.expect("Error in removing the test container");
    }

    pub const fn server(&self) -> &TestServer {
        &self.server
    }

    /// Signup a user
    pub async fn signup(
        &self,
        name: impl AsRef<str>,
        password: impl AsRef<str>,
    ) -> (UserId, UserToken) {
        let mut response: serde_json::Value = self
            .server()
            .post("/auth/signup")
            .json(&json!({
                "name": name.as_ref(),
                "password": password.as_ref()
            }))
            .expect_success()
            .await
            .json();
        (
            serde_json::from_value(response["id"].take()).unwrap(),
            serde_json::from_value(response["token"].take()).unwrap(),
        )
    }

    /// Create a session
    pub async fn new_session(
        &self,
        token: impl Borrow<UserToken>,
        name: impl AsRef<str>,
        description: Option<impl AsRef<str>>,
    ) -> SessionId {
        let mut response: serde_json::Value = self
            .server()
            .post("/sessions")
            .authorization_bearer(token.borrow())
            .json(&json!({
                "name": name.as_ref(),
                "description": description.as_ref().map(AsRef::<str>::as_ref)
            }))
            .expect_success()
            .await
            .json();

        serde_json::from_value(response["id"].take()).unwrap()
    }
}

#[instrument]
async fn db() -> ContainerAsync<Postgres> {
    tracing::info!("Creating test database");
    Postgres::default()
        .with_db_name("dices_server_test")
        .with_user("dices_server_test")
        .with_password("dices_server_test")
        .with_tag("17.0-alpine3.20")
        .start()
        .await
        .expect("Cannot start postgred db")
}

#[instrument]
async fn server(connection_string: String) -> TestServer {
    tracing::info!("Creating test app");

    let app = App::build(AppConfig {
        db: ConnectOptions::Url(connection_string),
        ..Default::default()
    })
    .await
    .expect("The app should be buildable")
    .service();

    TestServer::new(app).expect("Cannot create the test server")
}
