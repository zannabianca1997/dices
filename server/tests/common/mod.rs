use std::time::Duration;

use axum_test::TestServer;
use dices_server::{App, Config};
use serde_json::{from_value, json, Value};
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{runners::AsyncRunner, ContainerAsync, ImageExt},
};
use tokio::sync::oneshot::{self, Sender};
use tracing::instrument;
use uuid::Uuid;

#[must_use]
pub struct Infrastructure {
    pub db: ContainerAsync<Postgres>,
    shutdown: Sender<()>,
    pub server: TestServer,
}
#[allow(unused)]
impl Infrastructure {
    pub async fn up() -> Self {
        let db = db().await;
        let connection_string = format!(
            "postgres://dices_server_test:dices_server_test@{}:{}/dices_server_test",
            db.get_host().await.unwrap(),
            db.get_host_port_ipv4(5432).await.unwrap()
        );
        let (server, shutdown) = server(connection_string).await;
        Self {
            db,
            server,
            shutdown,
        }
    }

    pub async fn down(mut self) {
        let Self {
            db,
            server,
            shutdown,
        } = self;
        // sending shutdown signal
        shutdown.send(()).unwrap();
        // poll the server until is closed
        while server.is_running() {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        // drop the server
        drop(server);
        // stop the database
        db.stop().await.expect("Error in stopping postgres");
        // remove the database container
        db.rm().await.expect("Error in removing the test container");
    }

    pub async fn register(&self, username: &str, password: &str) -> (Uuid, String) {
        let registration = self
            .server
            .post("/api/v1/user/register")
            .json(&json!({
                "username": username,
                "password": password
            }))
            .expect_success()
            .await
            .json::<Value>();
        (
            from_value(registration["user"]["id"].clone()).unwrap(),
            from_value(registration["token"].clone()).unwrap(),
        )
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
async fn server(connection_string: String) -> (TestServer, Sender<()>) {
    tracing::info!("Creating test app");

    let (sender, receiver) = oneshot::channel();

    let app = App::new(Config::new(
        "127.0.0.1:0".to_owned(),
        connection_string,
        Duration::from_hours(1),
    ))
    .expect("The app should be configurable")
    .build_with_shutdown(async { receiver.await.expect("Error in receiving shutdown signal") })
    .await
    .expect("The app should be buildable");

    (
        TestServer::new(app).expect("Cannot create the test server"),
        sender,
    )
}
