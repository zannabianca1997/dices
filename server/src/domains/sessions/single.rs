//! # `/sessions/{session}`: API regarding a single session
//!
//! This is the main api with which the user can communicate with a session

use axum::{
    debug_handler,
    extract::{FromRef, State},
};
use sea_orm::{
    ActiveModelTrait as _, ColumnTrait as _, DatabaseConnection, EntityTrait as _,
    IntoActiveModel as _, QueryFilter as _, Set,
};
use utoipa_axum::{router::OpenApiRouter, routes};

use dices_server_auth::{Autenticated, AuthKey};
use dices_server_dtos::session::{
    SessionCreateDto, SessionGetError, SessionPathData, SessionQueryDto, SessionUpdateDto,
    SessionUpdateError,
};
use dices_server_entities::{
    prelude::*, sea_orm_active_enums::UserRole, session::SessionId, session_user, user::UserId,
};

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
    let (session, session_user) = fetch_session_data(&db, user_id, session_id).await?;

    Ok(SessionQueryDto {
        session,
        session_user: session_user.into(),
    })
}

#[utoipa::path(
    put,
    path = "/",
    request_body=SessionCreateDto,
    responses(SessionQueryDto, SessionUpdateError),
    params(SessionPathData)
)]
#[debug_handler(state = crate::app::App)]
/// Edit info about the session
///
/// Edit the info about the session.
/// This only works if the current user is part of the session, and has role `Admin`.
async fn session_put(
    State(db): State<DatabaseConnection>,
    user_id: Autenticated<UserId>,
    SessionPathData { id: session_id }: SessionPathData,
    SessionCreateDto { name, description }: SessionCreateDto,
) -> Result<SessionQueryDto, SessionUpdateError> {
    // First, we fetch it, guaranteeing it exist and the user is a member
    let (session, session_user) = fetch_session_data(&db, user_id, session_id).await?;

    // Check the permissions
    if session_user.role < UserRole::Admin {
        return Err(SessionUpdateError::NotAdmin);
    }

    // Apply the update
    let mut session = session.into_active_model();
    session.name = Set(name);
    session.description = Set(description);
    let session = session.update(&db).await?;

    Ok(SessionQueryDto {
        session,
        session_user: session_user.into(),
    })
}

#[utoipa::path(
    patch,
    path = "/",
    request_body=SessionUpdateDto,
    responses(SessionQueryDto, SessionUpdateError),
    params(SessionPathData)
)]
#[debug_handler(state = crate::app::App)]
/// Patch info about the session
///
/// Patch the info about the session, allowing partial editing.
/// This only works if the current user is part of the session, and has role `Admin`.
async fn session_patch(
    State(db): State<DatabaseConnection>,
    user_id: Autenticated<UserId>,
    SessionPathData { id: session_id }: SessionPathData,
    SessionUpdateDto { name, description }: SessionUpdateDto,
) -> Result<SessionQueryDto, SessionUpdateError> {
    // First, we fetch it, guaranteeing it exist and the user is a member
    let (session, session_user) = fetch_session_data(&db, user_id, session_id).await?;

    // Check the permissions
    if session_user.role < UserRole::Admin {
        return Err(SessionUpdateError::NotAdmin);
    }

    // Apply the update
    let mut session = session.into_active_model();
    if let Some(name) = name {
        session.name = Set(name);
    }
    if let Some(description) = description {
        session.description = Set(description);
    }
    let session = session.update(&db).await?;

    Ok(SessionQueryDto {
        session,
        session_user: session_user.into(),
    })
}

#[utoipa::path(
    delete,
    path = "/",
    responses(
        (status= OK, description="Session successfully deleted"),SessionUpdateError),
    params(SessionPathData)
)]
#[debug_handler(state = crate::app::App)]
/// Delete the session
///
/// Delete the session
/// This only works if the current user is part of the session, and has role `Admin`.
async fn session_delete(
    State(db): State<DatabaseConnection>,
    user_id: Autenticated<UserId>,
    SessionPathData { id: session_id }: SessionPathData,
) -> Result<(), SessionUpdateError> {
    // First, we fetch it, guaranteeing it exist and the user is a member
    let (session, session_user) = fetch_session_data(&db, user_id, session_id).await?;

    // Check the permissions
    if session_user.role < UserRole::Admin {
        return Err(SessionUpdateError::NotAdmin);
    }

    // Apply the update
    session.into_active_model().delete(&db).await?;

    Ok(())
}

pub fn router<S: Clone + Send + Sync + 'static>() -> OpenApiRouter<S>
where
    DatabaseConnection: FromRef<S>,
    AuthKey: FromRef<S>,
{
    OpenApiRouter::default().routes(routes!(
        session_get,
        session_put,
        session_patch,
        session_delete
    ))
}

async fn fetch_session_data(
    db: &DatabaseConnection,
    user_id: Autenticated<UserId>,
    session_id: SessionId,
) -> Result<(dices_server_entities::session::Model, session_user::Model), SessionGetError> {
    let (session, session_user) = Session::find_by_id(session_id)
        .find_also_related(SessionUser)
        .filter(session_user::Column::User.eq(*user_id.inner()))
        .one(db)
        .await?
        .ok_or(SessionGetError::NotFound)?;

    let session_user = session_user.expect("The query should only return sessions with users");

    Ok((session, session_user))
}
