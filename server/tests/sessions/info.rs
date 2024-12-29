use dices_server_entities::{sea_orm_active_enums::UserRole, session::SessionId};
use serde_json::json;
use test_log::test;

use crate::infrastructure::Infrastructure;

#[test(tokio::test)]
async fn should_get_infos() {
    Infrastructure::with(|infrastructure| {
        Box::pin(async move {
            let (_, token) = infrastructure.signup("user", "password").await;
            let id = infrastructure
                .new_session(&token, "session", None::<&str>)
                .await;

            let response = infrastructure
                .server()
                .get(&format!("/sessions/{id}"))
                .authorization_bearer(token)
                .expect_success()
                .await;

            response.assert_status_ok();

            response.assert_json_contains(&json!({
                "name": "session",
                "role": UserRole::Admin
            }));
        })
    })
    .await;
}

#[test(tokio::test)]
async fn should_not_get_info_with_wrong_uuid() {
    Infrastructure::with(|infrastructure| {
        Box::pin(async move {
            let (_, token) = infrastructure.signup("user", "password").await;
            infrastructure
                .new_session(&token, "session", None::<&str>)
                .await;

            let id = SessionId::gen();

            infrastructure
                .server()
                .get(&format!("/sessions/{id}"))
                .authorization_bearer(token)
                .expect_failure()
                .await
                .assert_status_not_found();
        })
    })
    .await;
}

#[test(tokio::test)]
async fn should_request_auth() {
    Infrastructure::with(|infrastructure| {
        Box::pin(async move {
            let (_, token) = infrastructure.signup("user", "password").await;
            let id = infrastructure
                .new_session(&token, "session", None::<&str>)
                .await;

            infrastructure
                .server()
                .get(&format!("/sessions/{id}"))
                .expect_failure()
                .await
                .assert_status_unauthorized();
        })
    })
    .await;
}

#[test(tokio::test)]
async fn should_require_to_be_members() {
    Infrastructure::with(|infrastructure| {
        Box::pin(async move {
            let (_, token) = infrastructure.signup("user", "password").await;
            let id = infrastructure
                .new_session(&token, "session", None::<&str>)
                .await;

            let (_, other_token) = infrastructure.signup("other_user", "other_password").await;

            infrastructure
                .server()
                .get(&format!("/sessions/{id}"))
                .authorization_bearer(other_token)
                .expect_failure()
                .await
                .assert_status_not_found();
        })
    })
    .await;
}
