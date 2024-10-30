use std::{borrow::Cow, error::Error, fmt::Display};

use axum::{http::StatusCode, response::IntoResponse, Json};
use derive_more::derive::{Constructor, Display, Error, From};
use dices_ast::value;
use dices_server_migration::schema;
use serde::Serialize;
use serde_json::to_value;
use serde_repr::{Deserialize_repr, Serialize_repr};
use utoipa::{ToResponse, ToSchema};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize_repr, Deserialize_repr, ToSchema,
)]
#[repr(u8)]
pub enum ErrorCodes {
    /*
     * Genera codes
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
    /// A human readable message about the error
    pub msg: Cow<'static, str>,
    /// Additional info on the error
    #[serde(flatten)]
    pub additional: serde_json::Map<String, serde_json::Value>,
}
impl ErrorResponse {
    pub fn internal_server_error(err: impl Error) -> Self {
        // log the error
        tracing::error!("Internal server error: {}", err);
        // return a opaque message
        Self {
            code: ErrorCodes::InternalServerError,
            http_code: None,
            msg: "Internal server error".into(),
            additional: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ErrorResponseBuilder {
    pub code: Option<ErrorCodes>,
    pub http_code: Option<StatusCode>,
    pub msg: Option<Cow<'static, str>>,
    pub additional: serde_json::Map<String, serde_json::Value>,
}
impl ErrorResponseBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn code(self, code: ErrorCodes) -> Self {
        Self {
            code: Some(code),
            ..self
        }
    }
    pub fn http_code(self, http_code: StatusCode) -> Self {
        Self {
            http_code: Some(http_code),
            ..self
        }
    }
    pub fn msg<M: Into<Cow<'static, str>>>(self, msg: M) -> Self {
        Self {
            msg: Some(msg.into()),
            ..self
        }
    }
    pub fn add<K: Into<String>, V: Serialize>(mut self, key: K, value: V) -> Self {
        self.additional.insert(
            key.into(),
            to_value(value).expect("Additional values serialization should be infallible"),
        );
        self
    }
    pub fn build(self) -> ErrorResponse {
        let Self {
            code: Some(code),
            http_code,
            msg: Some(msg),
            additional,
        } = self
        else {
            if self.code.is_none() {
                panic!("The error code must be set!")
            }
            if self.msg.is_none() {
                panic!("The message must be set")
            }
            unreachable!()
        };
        ErrorResponse {
            code,
            http_code,
            msg,
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
