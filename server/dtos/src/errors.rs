use std::{
    error::{Error, Report},
    fmt::Display,
};

use axum::{
    extract::rejection::{JsonRejection, PathRejection},
    response::IntoResponse,
    Json,
};
use derive_more::derive::From;
use either::Either::{Left, Right};
use http::StatusCode;
use schemars::{JsonSchema, JsonSchema_repr};
use serde::Serialize;
use serde_repr::Serialize_repr;
use serde_with::{
    chrono::{self, DateTime, FixedOffset},
    serde_as, DisplayFromStr,
};
use thiserror::Error;

/// Error code of the `dices` server
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize_repr, JsonSchema_repr)]
#[repr(u16)]
pub enum ErrorCode {
    // -- GENERAL --
    /// Internal server error
    InternalServerError = 0,
    /// A JSON sent was found to be invalid
    InvalidJson = 1,
    /// A path could not be parsed
    InvalidPath = 2,

    // -- AUTHENTICATION --
    /// The authentication header is missing
    MissingAuthHeader = 100,
    /// The authentication header cannot be
    InvalidAuthHeader = 101,
    /// The token is expired
    ExpiredToken = 102,
    /// The token is not well formed
    MalformedToken = 103,
    /// The used cannot access this path
    UnauthorizedId = 104,

    /// This user already exists
    UserAlreadyExists = 110,
    /// The use name cannot contain whitespaces
    WhitespacesInUsername = 111,
    /// The user do not exist
    UserDoNotExist = 112,
    /// The password provided is wrong
    WrongPassword = 113,
    /// The user was deleted
    UserDeleted = 114,
}
impl ErrorCode {
    pub fn status_code(&self) -> StatusCode {
        match self {
            ErrorCode::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::InvalidJson => StatusCode::BAD_REQUEST,
            ErrorCode::InvalidPath => StatusCode::NOT_FOUND,
            ErrorCode::InvalidAuthHeader | ErrorCode::MissingAuthHeader => StatusCode::UNAUTHORIZED,
            ErrorCode::ExpiredToken => StatusCode::FORBIDDEN,
            ErrorCode::MalformedToken => StatusCode::BAD_REQUEST,
            ErrorCode::UnauthorizedId => StatusCode::FORBIDDEN,
            ErrorCode::UserAlreadyExists => StatusCode::CONFLICT,
            ErrorCode::WhitespacesInUsername => StatusCode::BAD_REQUEST,
            ErrorCode::UserDoNotExist | ErrorCode::WrongPassword => StatusCode::UNAUTHORIZED,
            ErrorCode::UserDeleted => StatusCode::GONE,
        }
    }
}

pub trait ServerError: Error {
    fn error_code(&self) -> ErrorCode;
}

impl ServerError for JsonRejection {
    fn error_code(&self) -> ErrorCode {
        ErrorCode::InvalidJson
    }
}

impl ServerError for PathRejection {
    fn error_code(&self) -> ErrorCode {
        ErrorCode::InvalidPath
    }
}

#[serde_as]
#[derive(Serialize, JsonSchema)]
struct ErrorResponseDto<M: Display> {
    code: ErrorCode,
    #[serde_as(as = "DisplayFromStr")]
    msg: M,
    time: DateTime<FixedOffset>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, From)]
pub struct ErrorResponse<T>(pub T);

#[derive(Debug, Error)]
#[error("Internal server error")]
struct InternalServerError;

impl<T: ServerError> Serialize for ErrorResponse<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let Self(inner) = self;
        let code = inner.error_code();
        let msg = Report::new(if code != ErrorCode::InternalServerError {
            Left(inner)
        } else {
            tracing::error!("Internal server error: {}", Report::new(inner).pretty(true));
            Right(InternalServerError)
        })
        .pretty(true);

        ErrorResponseDto {
            code,
            msg,
            time: chrono::Local::now().fixed_offset(),
        }
        .serialize(serializer)
    }
}

impl<T: ServerError + JsonSchema> JsonSchema for ErrorResponse<T> {
    fn schema_name() -> String {
        ErrorResponseDto::<String>::schema_name()
    }

    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        ErrorResponseDto::<String>::json_schema(gen)
    }
}

impl<T: ServerError> IntoResponse for ErrorResponse<T> {
    fn into_response(self) -> axum::response::Response {
        (self.0.error_code().status_code(), Json(self)).into_response()
    }
}
