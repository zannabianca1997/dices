use axum::{
    extract::{FromRequest, FromRequestParts},
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, FixedOffset};
use derive_more::derive::From;
use dices_server_entities::{
    sea_orm_active_enums::UserRole,
    session::{self, SessionId},
    session_user,
};
use http::StatusCode;
use sea_orm::DbErr;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::{IntoParams, IntoResponses, OpenApi, ToSchema};

use crate::paginated::{PageInfo, PaginatedDto};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, From, utoipa::ToSchema, utoipa::IntoResponses)]
#[response(status = CREATED)]
/// Details about the newly created session
pub struct SessionCreateResponseDto(pub SessionQueryDto);

impl IntoResponse for SessionCreateResponseDto {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::CREATED, self.0).into_response()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, From, utoipa::ToSchema, utoipa::IntoResponses)]
#[response(status = OK)]
/// Details about a session
pub struct SessionQueryDto {
    #[serde(flatten)]
    pub session: session::Model,
    #[serde(flatten)]
    pub session_user: SessionUserAdditionalData,
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    Serialize,
    From,
    utoipa::ToSchema,
    utoipa::IntoResponses,
    Deserialize,
)]
#[response(status = OK)]
/// Short details about a session
pub struct SessionShortQueryDto {
    /// The session ID
    pub id: SessionId,
    /// The session name
    pub name: String,
    /// Last time the user sent a command to this session, or the time they where added
    pub last_interaction: DateTime<FixedOffset>,
}

impl From<SessionQueryDto> for SessionShortQueryDto {
    fn from(
        SessionQueryDto {
            session: session::Model { id, name, .. },
            session_user:
                SessionUserAdditionalData {
                    added_at,
                    last_access,
                    ..
                },
        }: SessionQueryDto,
    ) -> Self {
        Self {
            id,
            name,
            last_interaction: last_access.unwrap_or(added_at),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, IntoResponses, Error)]
pub enum SessionListGetError {
    #[error("Internal server error")]
    #[response(status=INTERNAL_SERVER_ERROR)]
    /// Internal server error
    InternalServerError,
}
impl IntoResponse for SessionListGetError {
    fn into_response(self) -> axum::response::Response {
        match self {
            SessionListGetError::InternalServerError => {
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
impl From<DbErr> for SessionListGetError {
    fn from(value: DbErr) -> Self {
        crate::internal_server_error(&value);
        Self::InternalServerError
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, From, utoipa::ToSchema)]
/// Additional data from the SessionUser
pub struct SessionUserAdditionalData {
    /// Role of the user in this session
    pub role: UserRole,
    /// Time the user was added to this session
    pub added_at: DateTime<FixedOffset>,
    /// Last time the user sent a command to this session
    pub last_access: Option<DateTime<FixedOffset>>,
}

impl From<session_user::Model> for SessionUserAdditionalData {
    fn from(
        session_user::Model {
            role,
            added_at,
            last_access,
            ..
        }: session_user::Model,
    ) -> Self {
        SessionUserAdditionalData {
            role,
            added_at,
            last_access,
        }
    }
}

impl IntoResponse for SessionQueryDto {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, FromRequest, ToSchema)]
#[from_request(via(axum::Json))]
pub struct SessionCreateDto {
    /// Name of the session
    ///
    /// Can contain spaces, but not newlines
    pub name: String,
    /// Description of the session
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, IntoResponses, Error)]
pub enum SessionCreateError {
    #[error("Name contains newlines")]
    #[response(status=BAD_REQUEST)]
    /// Name contains newlines
    NameContainsNewline,
    #[error("Internal server error")]
    #[response(status=INTERNAL_SERVER_ERROR)]
    /// Internal server error
    InternalServerError,
}

impl IntoResponse for SessionCreateError {
    fn into_response(self) -> axum::response::Response {
        match self {
            SessionCreateError::NameContainsNewline => StatusCode::BAD_REQUEST.into_response(),
            SessionCreateError::InternalServerError => {
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

impl From<DbErr> for SessionCreateError {
    fn from(value: DbErr) -> Self {
        crate::internal_server_error(&value);
        Self::InternalServerError
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, FromRequest, ToSchema)]
#[from_request(via(axum::Json))]
pub struct SessionUpdateDto {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default, with = "::serde_with::rust::double_option")]
    pub description: Option<Option<String>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, FromRequestParts, IntoParams)]
#[from_request(via(axum::extract::Path))]
pub struct SessionPathData {
    #[serde(rename = "session", alias = "session-id")]
    pub id: SessionId,
}

#[derive(Debug, Error, From, IntoResponses)]
pub enum SessionGetError {
    #[error("Session does not exist")]
    #[response(status=NOT_FOUND)]
    /// The session does not exist
    NotFound,
    #[error("Internal Server Error")]
    #[response(status=INTERNAL_SERVER_ERROR)]
    /// Internal server error
    InternalServerError,
}

impl IntoResponse for SessionGetError {
    fn into_response(self) -> axum::response::Response {
        match self {
            SessionGetError::NotFound => StatusCode::NOT_FOUND.into_response(),
            SessionGetError::InternalServerError => {
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

impl From<DbErr> for SessionGetError {
    fn from(value: DbErr) -> Self {
        crate::internal_server_error(&value);
        Self::InternalServerError
    }
}

#[derive(Debug, Error, From, IntoResponses)]
pub enum SessionUpdateError {
    #[error("Session does not exist")]
    #[response(status=NOT_FOUND)]
    /// The session does not exist
    NotFound,
    #[error("An admin user is needed to edit the session")]
    #[response(status=FORBIDDEN)]
    /// An admin user is needed to edit the session
    NotAdmin,
    #[error("Internal Server Error")]
    #[response(status=INTERNAL_SERVER_ERROR)]
    /// Internal server error
    InternalServerError,
}

impl IntoResponse for SessionUpdateError {
    fn into_response(self) -> axum::response::Response {
        match self {
            SessionUpdateError::NotFound => StatusCode::NOT_FOUND.into_response(),
            SessionUpdateError::InternalServerError => {
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            SessionUpdateError::NotAdmin => StatusCode::FORBIDDEN.into_response(),
        }
    }
}

impl From<SessionGetError> for SessionUpdateError {
    fn from(value: SessionGetError) -> Self {
        match value {
            SessionGetError::NotFound => Self::NotFound,
            SessionGetError::InternalServerError => Self::InternalServerError,
        }
    }
}
impl From<DbErr> for SessionUpdateError {
    fn from(value: DbErr) -> Self {
        crate::internal_server_error(&value);
        Self::InternalServerError
    }
}

#[derive(OpenApi)]
#[openapi(components(schemas(
    dices_server_entities::session::Model,
    SessionUserAdditionalData,
    SessionQueryDto,
    SessionShortQueryDto,
    SessionCreateDto,
    SessionUpdateDto,
    PageInfo,
    PaginatedDto<SessionShortQueryDto>
)))]
pub struct ApiComponents;
