use axum::http::StatusCode;
use dices_server_dtos::errors::ErrorCode;
use serde_json::{from_value, json, Map, Value};
use test_log::test;
use uuid::Uuid;

use crate::infrastructure::Infrastructure;

#[test(tokio::test)]
async fn user_should_be_able_to_signin() {
    Infrastructure::with(|infrastructure| {
        Box::pin(async move {
            infrastructure.signup("user", "password").await;

            let response = infrastructure
                .server()
                .post("/user/signin")
                .json(&json!({
                    "name": "user",
                    "password": "password"
                }))
                .expect_success()
                .await;

            response.assert_status(StatusCode::OK);

            response.assert_json_contains(&json!({
                "name": "user"
            }));

            let response: Map<String, Value> = response.json();
            let Value::String(_) = response.get("token").expect("The token must be returned")
            else {
                panic!("The token must be a string")
            };
            let _: Uuid = from_value(response.get("id").expect("The id must be returned").clone())
                .expect("The id must be a valid uuid");

            assert_eq!(response.get("password"), None, "The password was returned")
        })
    })
    .await;
}

#[test(tokio::test)]
async fn wrong_password_should_be_refused() {
    Infrastructure::with(|infrastructure| {
        Box::pin(async move {
            infrastructure.signup("user", "password").await;

            let response = infrastructure
                .server()
                .post("/user/signin")
                .json(&json!({
                    "name": "user",
                    "password": "wrong_password"
                }))
                .expect_failure()
                .await;

            response.assert_status(StatusCode::UNAUTHORIZED);

            response.assert_json_contains(&json!({
                "code": ErrorCode::WrongPassword
            }));
        })
    })
    .await;
}
