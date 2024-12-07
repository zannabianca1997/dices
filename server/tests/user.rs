#![feature(duration_constructors)]

use axum::http::StatusCode;
use common::Infrastructure;
use dices_server::ErrorCodes;
use serde_json::{from_value, json, Map, Value};
use uuid::Uuid;

use test_log::test;

mod common;

#[test(tokio::test)]
async fn register() {
    Infrastructure::with(|infrastructure| {
        Box::pin(async move {
            let response = infrastructure
                .server()
                .post("/api/v1/user/register")
                .json(&json!({
                    "username": "Zanna",
                    "password": "password"
                }))
                .expect_success()
                .await;

            response.assert_status(StatusCode::CREATED);
            let response: Map<String, Value> = response.json();

            let Value::String(_) = response.get("token").expect("The token must be returned")
            else {
                panic!("The token must be a string")
            };

            let Value::Object(user) = response.get("user").expect("The user must be returned")
            else {
                panic!("The user must be a map")
            };
            let Value::String(username) =
                user.get("username").expect("The username must be returned")
            else {
                panic!("The username must be a string")
            };
            assert_eq!(username, "Zanna");
            let _: Uuid = from_value(user.get("id").expect("The id must be returned").clone())
                .expect("The id must be a valid uuid");
        })
    })
    .await;
}

#[test(tokio::test)]
async fn duplicated_register_fail() {
    Infrastructure::with(|infrastructure| {
        Box::pin(async move {
            infrastructure.register("Zanna", "password").await;

            infrastructure
                .server()
                .post("/api/v1/user/register")
                .json(&json!({
                    "username": "Zanna",
                    "password": "password"
                }))
                .expect_failure()
                .await;
        })
    })
    .await;
}

#[test(tokio::test)]
async fn login() {
    Infrastructure::with(|infrastructure| {
        Box::pin(async move {
            let id = infrastructure.register("Zanna", "password").await.0;

            let response = infrastructure
                .server()
                .post("/api/v1/user/login")
                .json(&json!({
                    "username": "Zanna",
                    "password": "password"
                }))
                .expect_success()
                .await;

            response.assert_status_ok();
            let response: Map<String, Value> = response.json();

            let Value::String(_) = response.get("token").expect("The token must be returned")
            else {
                panic!("The token must be a string")
            };

            let Value::Object(user) = response.get("user").expect("The user must be returned")
            else {
                panic!("The user must be a map")
            };
            let Value::String(username) =
                user.get("username").expect("The username must be returned")
            else {
                panic!("The username must be a string")
            };
            assert_eq!(username, "Zanna");
            let got_id: Uuid = from_value(user.get("id").expect("The id must be returned").clone())
                .expect("The id must be a valid uuid");
            assert_eq!(got_id, id, "The user id must match");
        })
    })
    .await;
}

#[test(tokio::test)]
async fn login_fail() {
    Infrastructure::with(|infrastructure| {
        Box::pin(async move {
            let response = infrastructure
                .server()
                .post("/api/v1/user/login")
                .json(&json!({
                    "username": "Zanna",
                    "password": "password"
                }))
                .expect_failure()
                .await;

            response.assert_status(StatusCode::UNAUTHORIZED);
            response.assert_json_contains(&json!({
                "code": ErrorCodes::UnknowUsername
            }));
        })
    })
    .await;
}

#[test(tokio::test)]
async fn user_info() {
    Infrastructure::with(|infrastructure| {
        Box::pin(async move {
            let (id, token) = infrastructure.register("Zanna", "password").await;

            let response = infrastructure
                .server()
                .get("/api/v1/user")
                .authorization_bearer(token)
                .expect_success()
                .await;

            response.assert_status_ok();
            response.assert_json_contains(&json!({
                "id": id,
                "username": "Zanna",
            }));
            assert!(response.json::<Value>().get("password").is_none());
        })
    })
    .await;
}

#[test(tokio::test)]
async fn user_info_fail() {
    Infrastructure::with(|infrastructure| {
        Box::pin(async move {
            infrastructure.register("Zanna", "password").await;

            let response = infrastructure
                .server()
                .get("/api/v1/user")
                .expect_failure()
                .await;

            response.assert_status(StatusCode::UNAUTHORIZED);
            response.assert_json_contains(&json!({
                "code": ErrorCodes::InvalidAuthHeader
            }));
        })
    })
    .await;
}
