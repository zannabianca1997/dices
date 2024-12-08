#![feature(duration_constructors)]

use axum::http::StatusCode;
use common::Infrastructure;
use fnv::FnvHashSet;
use serde_json::{from_value, json, Map, Value};
use uuid::Uuid;

use test_log::test;

mod common;

#[test(tokio::test)]
async fn create() {
    Infrastructure::with(|infrastructure| {
        Box::pin(async move {
            let token = infrastructure.register("Zanna", "password").await.1;

            let response = infrastructure
                .server()
                .post("/api/v1/sessions")
                .authorization_bearer(token)
                .json(&json!({
                    "name": "Test Session"
                }))
                .expect_success()
                .await;

            response.assert_status(StatusCode::CREATED);
            let response: Map<String, Value> = response.json();

            let Value::String(name) = response.get("name").expect("The name must be returned")
            else {
                panic!("The name must be a string")
            };
            assert_eq!(name, "Test Session");
            assert!(response.get("description").is_none_or(Value::is_null));
            let _: Uuid = from_value(response.get("id").expect("The id must be returned").clone())
                .expect("The id must be a valid uuid");
        })
    })
    .await;
}

#[test(tokio::test)]
async fn create_no_user_fail() {
    Infrastructure::with(|infrastructure| {
        Box::pin(async move {
            infrastructure
                .server()
                .post("/api/v1/sessions")
                .json(&json!({
                    "name": "Test Session"
                }))
                .expect_failure()
                .await;
        })
    })
    .await;
}

#[test(tokio::test)]
async fn create_with_description() {
    Infrastructure::with(|infrastructure| {
        Box::pin(async move {
            let token = infrastructure.register("Zanna", "password").await.1;

            let response = infrastructure
                .server()
                .post("/api/v1/sessions")
                .authorization_bearer(token)
                .json(&json!({
                    "name": "Test Session",
                    "description": "My interesting description"
                }))
                .expect_success()
                .await;

            response.assert_status(StatusCode::CREATED);
            let response: Map<String, Value> = response.json();

            let Value::String(name) = response.get("name").expect("The name must be returned")
            else {
                panic!("The name must be a string")
            };
            assert_eq!(name, "Test Session");
            let Value::String(description) = response
                .get("description")
                .expect("The description must be returned")
            else {
                panic!("The description must be a string")
            };
            assert_eq!(description, "My interesting description");
            let _: Uuid = from_value(response.get("id").expect("The id must be returned").clone())
                .expect("The id must be a valid uuid");
        })
    })
    .await;
}

#[test(tokio::test)]
async fn get_sessions_list() {
    Infrastructure::with(|infrastructure| {
        Box::pin(async move {
            let token = infrastructure.register("Zanna", "password").await.1;
            let sessions_uuid = [
                infrastructure
                    .create_session(&token, "Test Session 1")
                    .await,
                infrastructure
                    .create_session(&token, "Test Session 2")
                    .await,
                infrastructure
                    .create_session(&token, "Test Session 3")
                    .await,
            ]
            .into_iter()
            .collect::<FnvHashSet<_>>();

            let response = infrastructure
                .server()
                .get(&format!("/api/v1/sessions"))
                .authorization_bearer(token)
                .expect_success()
                .await;

            response.assert_status_ok();

            let Value::Array(sessions) = response.json() else {
                panic!("Sessions should be an array");
            };

            let fetched_sessions_uuid = sessions
                .into_iter()
                .map(|session| {
                    from_value::<Uuid>(
                        session
                            .get("id")
                            .expect("The session uuid should be returned in the session list")
                            .clone(),
                    )
                    .expect("The session uuid should be a uuid")
                })
                .collect::<FnvHashSet<_>>();

            assert_eq!(
                sessions_uuid, fetched_sessions_uuid,
                "The list of sessions got does not match the list of session inserted"
            );
        })
    })
    .await;
}

#[test(tokio::test)]
async fn get_sessions_list_exclude_extraneus_sessions() {
    Infrastructure::with(|infrastructure| {
        Box::pin(async move {
            let token = infrastructure.register("Zanna", "password").await.1;
            let sessions_uuid = [
                infrastructure
                    .create_session(&token, "Test Session 1")
                    .await,
                infrastructure
                    .create_session(&token, "Test Session 2")
                    .await,
                infrastructure
                    .create_session(&token, "Test Session 3")
                    .await,
            ]
            .into_iter()
            .collect::<FnvHashSet<_>>();

            // Inserting other sessions
            {
                let token = infrastructure
                    .register("EvilZanna", "evil4password")
                    .await
                    .1;

                infrastructure
                    .create_session(&token, "Test Session 4")
                    .await;
                infrastructure
                    .create_session(&token, "Test Session 5")
                    .await;
                infrastructure
                    .create_session(&token, "Test Session 6")
                    .await;
            };

            let response = infrastructure
                .server()
                .get(&format!("/api/v1/sessions"))
                .authorization_bearer(token)
                .expect_success()
                .await;

            response.assert_status_ok();

            let Value::Array(sessions) = response.json() else {
                panic!("Sessions should be an array");
            };

            let fetched_sessions_uuid = sessions
                .into_iter()
                .map(|session| {
                    from_value::<Uuid>(
                        session
                            .get("id")
                            .expect("The session uuid should be returned in the session list")
                            .clone(),
                    )
                    .expect("The session uuid should be a uuid")
                })
                .collect::<FnvHashSet<_>>();

            assert_eq!(
                sessions_uuid, fetched_sessions_uuid,
                "The list of sessions got does not match the list of session inserted"
            );
        })
    })
    .await;
}

#[test(tokio::test)]
async fn get_session() {
    Infrastructure::with(|infrastructure| {
        Box::pin(async move {
            let token = infrastructure.register("Zanna", "password").await.1;
            let session_uuid = infrastructure.create_session(&token, "Test Session").await;

            let response = infrastructure
                .server()
                .get(&format!("/api/v1/sessions/{session_uuid}"))
                .authorization_bearer(token)
                .expect_success()
                .await;

            response.assert_status_ok();
            response.assert_json_contains(&json!({
                "name": "Test Session"
            }));
        })
    })
    .await;
}

#[test(tokio::test)]
async fn get_session_with_other_user_fail() {
    Infrastructure::with(|infrastructure| {
        Box::pin(async move {
            let token = infrastructure.register("Zanna", "password").await.1;
            let session_uuid = infrastructure.create_session(&token, "Test Session").await;
            let other_token = infrastructure.register("Other", "pasword").await.1;

            let response = infrastructure
                .server()
                .get(&format!("/api/v1/sessions/{session_uuid}"))
                .authorization_bearer(other_token)
                .expect_failure()
                .await;

            response.assert_status_not_found();
        })
    })
    .await;
}
