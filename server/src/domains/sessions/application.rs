use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use sea_orm::DatabaseConnection;
use utoipa::OpenApi;

use crate::{
    app::AppState,
    domains::{commons::ErrorResponse, user::AutenticatedUser},
};

use super::domain::models::{Session, SessionCreate};

#[utoipa::path(
    post,
    path = "",
    description = "Create a session",
    responses(
        (status=StatusCode::CREATED, description="The session was created", body = Session),
        (status=StatusCode::BAD_REQUEST, description="The creation data were wrong or incomplete", body = ErrorResponse),
        (status=StatusCode::UNAUTHORIZED, description="A session can be created only by authenticated users", body = ErrorResponse)
    ),
    security(("UserJWT" = []))
)]
async fn create(
    State(database): State<DatabaseConnection>,
    user: AutenticatedUser,
    Json(session_create): Json<SessionCreate>,
) -> Result<(StatusCode, Json<Session>), ErrorResponse> {
    let session = Session::new(&database, session_create, user).await?;
    Ok((StatusCode::CREATED, Json(session)))
}

pub(super) fn router() -> Router<AppState> {
    Router::new().route("/", post(create))
}

#[derive(OpenApi)]
#[openapi(paths(create), components(schemas(Session, SessionCreate)))]
pub(super) struct ApiDocs;
