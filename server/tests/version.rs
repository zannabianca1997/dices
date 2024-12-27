#[path = "common/infrastructure.rs"]
mod infrastructure;

use crate::infrastructure::Infrastructure;
use test_log::test;

#[test(tokio::test)]
async fn version_server_should_be_returned() {
    Infrastructure::with(|infrastructure| {
        Box::pin(async move {
            let response = infrastructure
                .server()
                .get("/version/server")
                .expect_success()
                .await;

            response.assert_status_ok();
            response.assert_json(&dices_server::VERSION);
        })
    })
    .await;
}

#[test(tokio::test)]
async fn version_engine_should_be_returned() {
    Infrastructure::with(|infrastructure| {
        Box::pin(async move {
            let response = infrastructure
                .server()
                .get("/version/engine")
                .expect_success()
                .await;

            response.assert_status_ok();
            response.assert_json(&dices_engine::VERSION);
        })
    })
    .await;
}

#[test(tokio::test)]
async fn version_ast_should_be_returned() {
    Infrastructure::with(|infrastructure| {
        Box::pin(async move {
            let response = infrastructure
                .server()
                .get("/version/ast")
                .expect_success()
                .await;

            response.assert_status_ok();
            response.assert_json(&dices_ast::VERSION);
        })
    })
    .await;
}
