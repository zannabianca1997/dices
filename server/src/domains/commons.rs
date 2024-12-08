use std::{borrow::Cow, error::Error, fmt::Display};

use axum::{http::StatusCode, response::IntoResponse, Json};
use sea_orm::DbErr;
use serde::Serialize;
use serde_json::to_value;
use serde_repr::{Deserialize_repr, Serialize_repr};
use utoipa::ToSchema;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize_repr, Deserialize_repr, ToSchema,
)]
#[repr(u8)]
pub enum ErrorCodes {
    /*
     * General codes
     */
    InternalServerError = 0,
    /*
     * Users
     */
    BlankUsername = 100,
    UsernameHasSpaces = 101,
    UsernameTaken = 102,
    UnknowUsername = 103,
    WrongPassword = 104,
    InvalidAuthHeader = 105,
    InvalidToken = 106,
    TokenExpired = 107,
    UserDeleted = 108,
    /*
     * Sessions
     */
    SessionNotFound = 200,
    UserNotMemberOfSession = 201,
    BlankSessionName = 210,
    CannotAddUserWithHigherRole = 220,
    CannotSeeUserList = 221,
}
impl ErrorCodes {
    fn http(&self) -> StatusCode {
        match self {
            ErrorCodes::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCodes::BlankUsername | ErrorCodes::UsernameHasSpaces => StatusCode::BAD_REQUEST,
            ErrorCodes::UsernameTaken => StatusCode::CONFLICT,
            ErrorCodes::UnknowUsername
            | ErrorCodes::WrongPassword
            | ErrorCodes::InvalidAuthHeader
            | ErrorCodes::InvalidToken => StatusCode::UNAUTHORIZED,
            ErrorCodes::TokenExpired => StatusCode::FORBIDDEN,
            ErrorCodes::UserDeleted => StatusCode::FORBIDDEN,
            ErrorCodes::BlankSessionName => StatusCode::BAD_REQUEST,
            ErrorCodes::SessionNotFound => StatusCode::NOT_FOUND,
            ErrorCodes::CannotAddUserWithHigherRole
            | ErrorCodes::UserNotMemberOfSession
            | ErrorCodes::CannotSeeUserList => StatusCode::FORBIDDEN,
        }
    }
}

/// A general error returned from the api
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ErrorResponse {
    /// The error code
    pub code: ErrorCodes,
    /// Status code to overwrite the default one for this error
    #[serde(skip)]
    pub http_code: Option<StatusCode>,
    /// A human-readable message about the error
    pub msg: Cow<'static, str>,
    /// Additional info on the error
    #[serde(flatten)]
    pub additional: serde_json::Map<String, serde_json::Value>,
}
impl ErrorResponse {
    pub fn internal_server_error(err: impl Error) -> Self {
        // log the error
        tracing::error!("Internal server error: {}", err);
        // return an opaque message
        Self::builder()
            .code(ErrorCodes::InternalServerError)
            .msg("Internal server error")
            .build()
    }

    pub(crate) fn builder() -> ErrorResponseBuilder<(), ()> {
        ErrorResponseBuilder::new()
    }
}
impl From<DbErr> for ErrorResponse {
    fn from(value: DbErr) -> Self {
        Self::internal_server_error(value)
    }
}
impl From<!> for ErrorResponse {
    fn from(value: !) -> Self {
        value
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ErrorResponseBuilder<C, M> {
    pub(crate) code: C,
    pub(crate) http_code: Option<StatusCode>,
    pub(crate) msg: M,
    pub(crate) additional: serde_json::Map<String, serde_json::Value>,
}
impl ErrorResponseBuilder<(), ()> {
    fn new() -> Self {
        Self {
            code: (),
            http_code: None,
            msg: (),
            additional: serde_json::Map::new(),
        }
    }
}
impl<C, M> ErrorResponseBuilder<C, M> {
    pub(crate) fn code<C2: Into<ErrorCodes>>(self, code: C2) -> ErrorResponseBuilder<C2, M> {
        let Self {
            code: _,
            http_code,
            msg,
            additional,
        } = self;
        ErrorResponseBuilder {
            code,
            http_code,
            msg,
            additional,
        }
    }
    pub(crate) fn http_code(self, http_code: impl Into<StatusCode>) -> Self {
        Self {
            http_code: Some(http_code.into()),
            ..self
        }
    }
    pub(crate) fn msg<M2: Into<Cow<'static, str>>>(self, msg: M2) -> ErrorResponseBuilder<C, M2> {
        let Self {
            code,
            http_code,
            msg: _,
            additional,
        } = self;
        ErrorResponseBuilder {
            code,
            http_code,
            msg,
            additional,
        }
    }
    pub(crate) fn add(mut self, key: impl Display, value: impl Serialize) -> Self {
        self.additional.insert(
            key.to_string(),
            to_value(value).expect("Additional values serialization should be infallible"),
        );
        self
    }
}
impl<C: Into<ErrorCodes>, M: Into<Cow<'static, str>>> ErrorResponseBuilder<C, M> {
    pub(crate) fn build(self) -> ErrorResponse {
        let Self {
            code,
            http_code,
            msg,
            additional,
        } = self;
        ErrorResponse {
            code: code.into(),
            http_code,
            msg: msg.into(),
            additional,
        }
    }
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> axum::response::Response {
        let status = self.http_code.unwrap_or(self.code.http());
        (status, Json(self)).into_response()
    }
}
