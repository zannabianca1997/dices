#![feature(duration_constructors)]

mod common;

use crate::common::Infrastructure;
use test_log::test;

#[test(tokio::test)]
async fn version_server() {
    Infrastructure::with(|infrastructure| {
        Box::pin(async move {
            let response = infrastructure
                .server()
                .get("/api/v1/version")
                .expect_success()
                .await;

            response.assert_status_ok();
            response.assert_json(&dices_server::VERSION);
        })
    })
    .await;
}

#[test(tokio::test)]
async fn version_engine() {
    Infrastructure::with(|infrastructure| {
        Box::pin(async move {
            let response = infrastructure
                .server()
                .get("/api/v1/version/engine")
                .expect_success()
                .await;

            response.assert_status_ok();
            response.assert_json(&dices_engine::VERSION);
        })
    })
    .await;
}

#[test(tokio::test)]
async fn version_ast() {
    Infrastructure::with(|infrastructure| {
        Box::pin(async move {
            let response = infrastructure
                .server()
                .get("/api/v1/version/ast")
                .expect_success()
                .await;

            response.assert_status_ok();
            response.assert_json(&dices_ast::VERSION);
        })
    })
    .await;
}
