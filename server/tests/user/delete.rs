use axum::http::StatusCode;
use dices_server_entities::user::UserId;
use serde_json::{from_value, json, Map, Value};
use test_log::test;
use uuid::Uuid;

use crate::infrastructure::Infrastructure;

#[test(tokio::test)]
async fn should_be_able_to_delete() {
    Infrastructure::with(|infrastructure| {
        Box::pin(async move {
            let (_, token) = infrastructure.signup("user", "password").await;

            let response = infrastructure
                .server()
                .delete("/user")
                .authorization_bearer(&token)
                .expect_success()
                .await;

            response.assert_status(StatusCode::OK);

            let check = infrastructure
                .server()
                .get("/user")
                .authorization_bearer(token)
                .expect_failure()
                .await;

            check.assert_status(StatusCode::GONE);
        })
    })
    .await;
}

#[test(tokio::test)]
async fn cannot_signin_back() {
    Infrastructure::with(|infrastructure| {
        Box::pin(async move {
            let (_, token) = infrastructure.signup("user", "password").await;

            infrastructure
                .server()
                .delete("/user")
                .authorization_bearer(token)
                .expect_success()
                .await;

            let check = infrastructure
                .server()
                .post("/auth/signin")
                .json(&json!({
                    "name": "user",
                    "password": "password"
                }))
                .expect_failure()
                .await;

            check.assert_status(StatusCode::UNAUTHORIZED);
        })
    })
    .await;
}

#[test(tokio::test)]
async fn can_signup_again() {
    Infrastructure::with(|infrastructure| {
        Box::pin(async move {
            let (old_id, token) = infrastructure.signup("user", "password").await;

            infrastructure
                .server()
                .delete("/user")
                .authorization_bearer(token)
                .expect_success()
                .await;

            let check = infrastructure
                .server()
                .post("/auth/signup")
                .json(&json!({
                    "name": "user",
                    "password": "password"
                }))
                .expect_success()
                .await;

            check.assert_status(StatusCode::CREATED);

            let new_id: UserId = from_value(
                check
                    .json::<Map<String, Value>>()
                    .get("id")
                    .unwrap()
                    .clone(),
            )
            .unwrap();

            assert_ne!(
                old_id, new_id,
                "The newly created user should have a different id"
            )
        })
    })
    .await;
}

#[test(tokio::test)]
async fn should_request_auth() {
    Infrastructure::with(|infrastructure| {
        Box::pin(async move {
            infrastructure.signup("user", "password").await;

            let response = infrastructure
                .server()
                .delete("/user")
                .expect_failure()
                .await;

            response.assert_status(StatusCode::UNAUTHORIZED);
        })
    })
    .await;
}
