use axum::extract::FromRef;
use dices_server_auth::AuthKey;
use sea_orm::DatabaseConnection;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;

mod auth;
mod sessions;
mod user;
mod version;

#[derive(OpenApi)]
struct Api;

pub(super) fn router<S: Clone + Send + Sync + 'static>() -> OpenApiRouter<S>
where
    DatabaseConnection: FromRef<S>,
    AuthKey: FromRef<S>,
{
    OpenApiRouter::with_openapi(Api::openapi())
        .nest("/user", user::router())
        .nest("/auth", auth::router())
        .nest("/sessions", sessions::router())
        .nest("/version", version::router())
}

/// Tag all the paths of an api
fn tag_api(openapi: &mut utoipa::openapi::OpenApi, tag: &str) {
    for path in openapi.paths.paths.values_mut() {
        for op in [
            &mut path.get,
            &mut path.head,
            &mut path.trace,
            &mut path.put,
            &mut path.post,
            &mut path.patch,
            &mut path.delete,
        ]
        .into_iter()
        .flatten()
        {
            op.tags.get_or_insert_default().push(tag.to_string());
        }
    }
}
