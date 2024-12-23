use axum::{
    extract::{
        rejection::{JsonRejection, PathRejection},
        FromRequest, FromRequestParts,
    },
    response::IntoResponse,
    Json,
};
use derive_more::derive::From;
use dices_server_entities::user::{self, UserId};
use http::StatusCode;
use sea_orm::DbErr;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::errors::{ErrorCode, ErrorResponse, ServerError};

pub mod token;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, From)]
pub struct UserQueryDto(pub user::Model);

impl IntoResponse for UserQueryDto {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, FromRequest)]
#[from_request(via(axum::Json), rejection(ErrorResponse<JsonRejection>))]
pub struct UserSignupDto(pub UserSigninDto);

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, FromRequest)]
#[from_request(via(axum::Json), rejection(ErrorResponse<JsonRejection>))]
pub struct UserSigninDto {
    pub name: String,
    pub password: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct UserSignupResponseDto(pub UserLoginResponseDto);

impl IntoResponse for UserSignupResponseDto {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::CREATED, self.0).into_response()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct UserLoginResponseDto {
    /// The token needed to access the API
    pub token: token::UserToken,
    #[serde(flatten)]
    pub user: dices_server_entities::user::Model,
}

impl IntoResponse for UserLoginResponseDto {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

#[derive(Debug, Error)]
pub enum SignupError {
    #[error("A user with this name already exists")]
    UserAlreadyExist,
    #[error("The user name cannot contain whitespaces")]
    WhitespacesInUsername,
    #[error("Database error")]
    DbErr(#[source] DbErr),
}
impl ServerError for SignupError {
    fn error_code(&self) -> crate::errors::ErrorCode {
        match self {
            SignupError::UserAlreadyExist => ErrorCode::UserAlreadyExists,
            SignupError::WhitespacesInUsername => ErrorCode::WhitespacesInUsername,
            SignupError::DbErr(_) => ErrorCode::InternalServerError,
        }
    }
}
impl IntoResponse for SignupError {
    fn into_response(self) -> axum::response::Response {
        ErrorResponse(self).into_response()
    }
}
#[derive(Debug, Error)]
pub enum SigninError {
    #[error("This user does not exist")]
    UserDoNotExist,
    #[error("Wrong password")]
    WrongPassword,
    #[error("Database error")]
    DbErr(#[source] DbErr),
    #[error("Error in checking password")]
    CheckPasswordError(#[source] argon2::password_hash::Error),
}
impl ServerError for SigninError {
    fn error_code(&self) -> crate::errors::ErrorCode {
        match self {
            SigninError::DbErr(_) | SigninError::CheckPasswordError(_) => {
                ErrorCode::InternalServerError
            }
            SigninError::UserDoNotExist => ErrorCode::UserDoNotExist,
            SigninError::WrongPassword => ErrorCode::WrongPassword,
        }
    }
}
impl IntoResponse for SigninError {
    fn into_response(self) -> axum::response::Response {
        ErrorResponse(self).into_response()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, FromRequest)]
#[from_request(via(axum::Json), rejection(ErrorResponse<JsonRejection>))]
pub struct UserUpdateDto {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, FromRequestParts)]
#[from_request(via(axum::extract::Path), rejection(ErrorResponse<PathRejection>))]
pub struct UserPathData {
    #[serde(rename = "user", alias = "user-id")]
    pub id: UserId,
}

#[derive(Debug, Error, From)]
pub enum UserGetError {
    #[error("User was deleted")]
    Deleted,
    #[error("Database error")]
    DbErr(#[source] DbErr),
}
impl ServerError for UserGetError {
    fn error_code(&self) -> crate::errors::ErrorCode {
        match self {
            UserGetError::DbErr(_) => ErrorCode::InternalServerError,
            UserGetError::Deleted => ErrorCode::UserDeleted,
        }
    }
}
impl IntoResponse for UserGetError {
    fn into_response(self) -> axum::response::Response {
        ErrorResponse(self).into_response()
    }
}
