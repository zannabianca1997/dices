//! # `/sessions/{session}`: API regarding a single session
//!
//! This is the main api with which the user can communicate with a session

use axum::{
    debug_handler,
    extract::{FromRef, State},
};
use sea_orm::{ColumnTrait as _, DatabaseConnection, EntityTrait as _, QueryFilter as _};
use utoipa_axum::{router::OpenApiRouter, routes};

use dices_server_auth::{Autenticated, AuthKey};
use dices_server_dtos::session::{SessionGetError, SessionPathData, SessionQueryDto};
use dices_server_entities::{prelude::*, session_user, user::UserId};

#[utoipa::path(
    get,
    path = "/",
    responses(SessionQueryDto, SessionGetError),
    params(SessionPathData)
)]
#[debug_handler(state = crate::app::App)]
/// Get info about the session
///
/// Get the info about the session requested.
/// This only works if the current user is part of the session.
async fn session_get(
    State(db): State<DatabaseConnection>,
    user_id: Autenticated<UserId>,
    SessionPathData { id: session_id }: SessionPathData,
) -> Result<SessionQueryDto, SessionGetError> {
    let (session, session_user) = Session::find_by_id(session_id)
        .find_also_related(SessionUser)
        .filter(session_user::Column::User.eq(*user_id.inner()))
        .one(&db)
        .await?
        .ok_or(SessionGetError::NotFound)?;

    Ok(SessionQueryDto {
        session,
        session_user: session_user
            .expect("The query should find only sessions with a user")
            .into(),
    })
}

pub fn router<S: Clone + Send + Sync + 'static>() -> OpenApiRouter<S>
where
    DatabaseConnection: FromRef<S>,
    AuthKey: FromRef<S>,
{
    OpenApiRouter::default().routes(routes!(session_get))
}
