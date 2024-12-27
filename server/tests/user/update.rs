use axum::http::StatusCode;
use serde_json::{json, Map, Value};
use test_log::test;

use crate::infrastructure::Infrastructure;

#[test(tokio::test)]
async fn should_update_all() {
    Infrastructure::with(|infrastructure| {
        Box::pin(async move {
            let (id, token) = infrastructure.signup("user", "password").await;

            let response = infrastructure
                .server()
                .put("/user")
                .authorization_bearer(token)
                .json(&json!({
                    "name": "user2",
                    "password": "password2"
                }))
                .expect_success()
                .await;

            response.assert_status(StatusCode::OK);

            response.assert_json_contains(&json!({
                "name": "user2",
                "id": id,
            }));

            let response = response.json::<Map<String, Value>>();

            assert_eq!(response.get("password"), None, "The password was returned");

            assert!(response.get("created_at").is_some());
            assert!(response.get("last_access").is_some());

            // Check that we can enter with the new password
            infrastructure
                .server()
                .post("/user/signin")
                .json(&json!({
                    "name": "user2",
                    "password": "password2"
                }))
                .expect_success()
                .await
                .assert_json_contains(&json!({
                    "id": id,
                }));
        })
    })
    .await;
}

#[test(tokio::test)]
async fn should_update_name() {
    Infrastructure::with(|infrastructure| {
        Box::pin(async move {
            let (id, token) = infrastructure.signup("user", "password").await;

            let response = infrastructure
                .server()
                .patch("/user")
                .authorization_bearer(token)
                .json(&json!({
                    "name": "user2",
                }))
                .expect_success()
                .await;

            response.assert_status(StatusCode::OK);

            response.assert_json_contains(&json!({
                "name": "user2",
                "id": id,
            }));

            let response = response.json::<Map<String, Value>>();

            assert_eq!(response.get("password"), None, "The password was returned");

            assert!(response.get("created_at").is_some());
            assert!(response.get("last_access").is_some());

            // Check that we can enter with the new password
            infrastructure
                .server()
                .post("/user/signin")
                .json(&json!({
                    "name": "user2",
                    "password": "password"
                }))
                .expect_success()
                .await
                .assert_json_contains(&json!({
                    "id": id,
                }));
        })
    })
    .await;
}

#[test(tokio::test)]
async fn should_update_password() {
    Infrastructure::with(|infrastructure| {
        Box::pin(async move {
            let (id, token) = infrastructure.signup("user", "password").await;

            let response = infrastructure
                .server()
                .patch("/user")
                .authorization_bearer(token)
                .json(&json!({
                    "password": "password2"
                }))
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

            // Check that we can enter with the new password
            infrastructure
                .server()
                .post("/user/signin")
                .json(&json!({
                    "name": "user",
                    "password": "password2"
                }))
                .expect_success()
                .await
                .assert_json_contains(&json!({
                    "id": id,
                }));
        })
    })
    .await;
}

#[test(tokio::test)]
async fn put_should_request_auth() {
    Infrastructure::with(|infrastructure| {
        Box::pin(async move {
            infrastructure.signup("user", "password").await;

            let response = infrastructure.server().put("/user").expect_failure().await;

            response.assert_status(StatusCode::UNAUTHORIZED);
        })
    })
    .await;
}

#[test(tokio::test)]
async fn patch_should_request_auth() {
    Infrastructure::with(|infrastructure| {
        Box::pin(async move {
            infrastructure.signup("user", "password").await;

            let response = infrastructure
                .server()
                .patch("/user")
                .expect_failure()
                .await;

            response.assert_status(StatusCode::UNAUTHORIZED);
        })
    })
    .await;
}
