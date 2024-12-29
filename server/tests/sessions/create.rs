use axum::http::StatusCode;
use dices_server_entities::{sea_orm_active_enums::UserRole, session::SessionId};
use serde_json::{from_value, json, Map, Value};
use test_log::test;

use crate::infrastructure::Infrastructure;

#[test(tokio::test)]
async fn should_be_able_to_create_session() {
    Infrastructure::with(|infrastructure| {
        Box::pin(async move {
            let (_, token) = infrastructure.signup("user", "password").await;

            let response = infrastructure
                .server()
                .post("/sessions")
                .authorization_bearer(token)
                .json(&json!({
                    "name": "session",
                }))
                .expect_success()
                .await;

            response.assert_status(StatusCode::CREATED);

            response.assert_json_contains(&json!({
                "name": "session",
                "description": null,
                "role": UserRole::Admin
            }));

            let response: Map<String, Value> = response.json();
            let _: SessionId =
                from_value(response.get("id").expect("The id must be returned").clone())
                    .expect("The id must be a valid uuid");
        })
    })
    .await;
}

#[test(tokio::test)]
async fn should_request_auth() {
    Infrastructure::with(|infrastructure| {
        Box::pin(async move {
            let response = infrastructure
                .server()
                .post("/sessions")
                .json(&json!({
                    "name": "session",
                }))
                .expect_failure()
                .await;

            response.assert_status(StatusCode::UNAUTHORIZED);
        })
    })
    .await;
}

#[test(tokio::test)]
async fn should_be_able_to_create_session_with_description() {
    Infrastructure::with(|infrastructure| {
        Box::pin(async move {
            let (_, token) = infrastructure.signup("user", "password").await;

            let response = infrastructure
                .server()
                .post("/sessions")
                .authorization_bearer(token)
                .json(&json!({
                    "name": "session",
                    "description": "Some description"
                }))
                .expect_success()
                .await;

            response.assert_status(StatusCode::CREATED);

            response.assert_json_contains(&json!({
                "name": "session",
                "description": "Some description",
                "role": UserRole::Admin
            }));

            let response: Map<String, Value> = response.json();
            let _: SessionId =
                from_value(response.get("id").expect("The id must be returned").clone())
                    .expect("The id must be a valid uuid");
        })
    })
    .await;
}

#[test(tokio::test)]
async fn should_be_able_to_create_session_with_same_name() {
    Infrastructure::with(|infrastructure| {
        Box::pin(async move {
            let (_, token) = infrastructure.signup("user", "password").await;
            let first_id = infrastructure
                .new_session(&token, "session", None::<&str>)
                .await;

            let response = infrastructure
                .server()
                .post("/sessions")
                .authorization_bearer(token)
                .json(&json!({
                    "name": "session",
                }))
                .expect_success()
                .await;

            let response: Map<String, Value> = response.json();
            let other_id: SessionId =
                from_value(response.get("id").expect("The id must be returned").clone())
                    .expect("The id must be a valid uuid");

            assert_ne!(first_id, other_id);
        })
    })
    .await;
}
