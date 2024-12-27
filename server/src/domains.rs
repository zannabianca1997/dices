use axum::extract::FromRef;
use dices_server_auth::AuthKey;
use sea_orm::DatabaseConnection;
use utoipa_axum::router::OpenApiRouter;

mod auth;
mod user;
mod version;

pub(super) fn router<S: Clone + Send + Sync + 'static>() -> OpenApiRouter<S>
where
    DatabaseConnection: FromRef<S>,
    AuthKey: FromRef<S>,
{
    OpenApiRouter::default()
        .nest("/user", user::router())
        .nest("/auth", auth::router())
        .nest("/version", version::router())
}
