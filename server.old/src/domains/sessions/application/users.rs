use axum::{
    extract::{Path, State},
    Json,
};
use sea_orm::DatabaseConnection;

use crate::domains::{
    commons::ErrorResponse,
    sessions::domain::models::{Session, SessionId, SessionUser},
    user::AutenticatedUser,
};

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
pub async fn get_all(
    State(database): State<DatabaseConnection>,
    Path(session_uuid): Path<SessionId>,
    requester: AutenticatedUser,
) -> Result<Json<Box<[SessionUser]>>, ErrorResponse> {
    let users = Session::users(&database, session_uuid, requester)
        .await?
        .try_collect()?;
    Ok(Json(users))
}
