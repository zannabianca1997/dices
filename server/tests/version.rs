#![feature(duration_constructors)]

mod common;

use test_log::test;

#[test(tokio::test)]
async fn version_server() {
    let infrastructure = common::Infrastructure::up().await;

    let response = infrastructure
        .server
        .get("/api/v1/version")
        .expect_success()
        .await;

    response.assert_status_ok();
    response.assert_json(&dices_server::VERSION);

    infrastructure.down().await;
}
