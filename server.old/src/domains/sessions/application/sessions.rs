use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sea_orm::DatabaseConnection;

use super::super::domain::models::SessionCreate;
use crate::{
    domains::{
        commons::ErrorResponse,
        sessions::domain::models::{Session, SessionId},
        user::AutenticatedUser,
    },
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
pub(crate) async fn post_new(
    State(database): State<DatabaseConnection>,
    user: AutenticatedUser,
    Json(session_create): Json<SessionCreate>,
) -> Result<(StatusCode, Json<Session>), ErrorResponse> {
    let session = Session::new(&database, session_create, user).await?;
    Ok((StatusCode::CREATED, Json(session)))
}

#[utoipa::path(
    get,
    path = "",
    description = "Get a list of sessions available",
    responses(
        (status=StatusCode::OK, description="The sessions found", body = [Session]),
        (status=StatusCode::UNAUTHORIZED, description="Sessions can be queried only by authenticated users", body = ErrorResponse)
    ),
    security(("UserJWT" = []))
)]
pub(crate) async fn get_all(
    State(database): State<DatabaseConnection>,
    requester: AutenticatedUser,
) -> Result<Json<Box<[Session]>>, ErrorResponse> {
    let sessions = Session::find_all(&database, requester)
        .await?
        .try_collect()?;
    Ok(Json(sessions))
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
pub(crate) async fn get(
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
    delete,
    path = "/{session-uuid}",
    description = "Delete a session",
    params(
        ("session-uuid" = SessionId, description="UUID of the session", format=Uuid),
    ),
    responses(
        (status=StatusCode::OK, description="The session was deleted"),
        (status=StatusCode::UNAUTHORIZED, description="Sessions can be deleted only by authenticated users", body = ErrorResponse),
        (status=StatusCode::FORBIDDEN, description="User has not the role needed to delete the session", body = ErrorResponse),
        (status=StatusCode::NOT_FOUND, description="Session does not exist", body = ErrorResponse)
    ),
    security(("UserJWT" = []))
)]
pub(crate) async fn delete(
    State(database): State<DatabaseConnection>,
    Path(session_uuid): Path<SessionId>,
    requester: AutenticatedUser,
) -> Result<(), ErrorResponse> {
    Ok(Session::delete(&database, session_uuid, requester).await?)
}
