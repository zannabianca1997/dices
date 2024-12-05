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

#[test(tokio::test)]
async fn version_engine() {
    let infrastructure = common::Infrastructure::up().await;

    let response = infrastructure
        .server
        .get("/api/v1/version/engine")
        .expect_success()
        .await;

    response.assert_status_ok();
    response.assert_json(&dices_engine::VERSION);

    infrastructure.down().await;
}

#[test(tokio::test)]
async fn version_ast() {
    let infrastructure = common::Infrastructure::up().await;

    let response = infrastructure
        .server
        .get("/api/v1/version/ast")
        .expect_success()
        .await;

    response.assert_status_ok();
    response.assert_json(&dices_ast::VERSION);

    infrastructure.down().await;
}
