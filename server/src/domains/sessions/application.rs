use axum::extract::Path;
use axum::routing::get;
use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use sea_orm::DatabaseConnection;
use utoipa::OpenApi;

use super::domain::models::{Session, SessionCreate, SessionId, SessionUser};
use crate::{
    app::AppState,
    domains::{commons::ErrorResponse, user::AutenticatedUser},
    ErrorCodes,
};

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

#[utoipa::path(
    get,
    path = "/{session-uuid}",
    description = "Get data about session",
    params(
        ("session-uuid" = SessionId, description="UUID of the session", format=Uuid),
    ),
    responses(
        (status=StatusCode::OK, description="The session found", body = Session),
        (status=StatusCode::UNAUTHORIZED, description="Sessions can be queried only by authenticated users", body = ErrorResponse),
        (status=StatusCode::NOT_FOUND, description="Session does not exist", body = ErrorResponse)
    ),
    security(("UserJWT" = []))
)]
async fn query(
    State(database): State<DatabaseConnection>,
    Path(session_uuid): Path<SessionId>,
    requester: AutenticatedUser,
) -> Result<Json<Session>, ErrorResponse> {
    Session::find_by_id(&database, session_uuid, requester)
        .await?
        .ok_or_else(|| {
            ErrorResponse::builder()
                .code(ErrorCodes::SessionNotFound)
                .msg(format!("The session {session_uuid} does not exist"))
                .add("uuid", session_uuid)
                .build()
        })
        .map(Json)
}
#[utoipa::path(
    get,
    path = "/{session-uuid}/users",
    description = "Get the list of users in this session",
    params(
        ("session-uuid" = SessionId, description="UUID of the session", format=Uuid),
    ),
    responses(
        (status=StatusCode::OK, description="The users found", body = [SessionUser]),
        (status=StatusCode::UNAUTHORIZED, description="Sessions can be queried only by authenticated users", body = ErrorResponse),
        (status=StatusCode::NOT_FOUND, description="Session does not exist", body = ErrorResponse)
    ),
    security(("UserJWT" = []))
)]
async fn query_users(
    State(database): State<DatabaseConnection>,
    Path(session_uuid): Path<SessionId>,
    requester: AutenticatedUser,
) -> Result<Json<Box<[SessionUser]>>, ErrorResponse> {
    let users = query(State(database.clone()), Path(session_uuid), requester)
        .await?
        .0
        .users(&database, requester)
        .await?
        .try_collect()?;
    Ok(Json(users))
}

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create))
        .route("/:session-uuid", get(query))
        .route("/:session-uuid/users", get(query_users))
}

#[derive(OpenApi)]
#[openapi(
    paths(create, query, query_users),
    components(schemas(Session, SessionCreate, SessionUser))
)]
pub(super) struct ApiDocs;
