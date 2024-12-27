use axum::http::StatusCode;
use serde_json::{json, Map, Value};
use test_log::test;

use crate::infrastructure::Infrastructure;

#[test(tokio::test)]
async fn should_return_infos() {
    Infrastructure::with(|infrastructure| {
        Box::pin(async move {
            let (id, token) = infrastructure.signup("user", "password").await;

            let response = infrastructure
                .server()
                .get("/user")
                .authorization_bearer(token)
                .expect_success()
                .await;

            response.assert_status(StatusCode::OK);

            response.assert_json_contains(&json!({
                "name": "user",
                "id": id,
            }));

            let response = response.json::<Map<String, Value>>();

            assert_eq!(response.get("password"), None, "The password was returned");

            assert!(response.get("created_at").is_some());
            assert!(response.get("last_access").is_some());
        })
    })
    .await;
}

#[test(tokio::test)]
async fn should_request_auth() {
    Infrastructure::with(|infrastructure| {
        Box::pin(async move {
            infrastructure.signup("user", "password").await;

            let response = infrastructure.server().get("/user").expect_failure().await;

            response.assert_status(StatusCode::UNAUTHORIZED);
        })
    })
    .await;
}
