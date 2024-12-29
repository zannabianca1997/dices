use axum::{
    extract::{FromRequest, FromRequestParts},
    response::IntoResponse,
    Json,
};
use derive_more::derive::From;
use http::StatusCode;
use sea_orm::DbErr;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::{IntoResponses, OpenApi};

use dices_server_entities::user::{self, UserId};

pub mod token;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, From, utoipa::ToSchema, utoipa::IntoResponses)]
#[response(status = OK)]
/// Details about a user
pub struct UserQueryDto(pub user::Model);

impl IntoResponse for UserQueryDto {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, FromRequest)]
#[from_request(via(axum::Json))]
#[derive(utoipa::ToSchema)]
pub struct UserSignupDto(pub UserSigninDto);

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, FromRequest)]
#[from_request(via(axum::Json))]
#[derive(utoipa::ToSchema)]
pub struct UserSigninDto {
    pub name: String,
    pub password: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, utoipa::ToSchema, utoipa::IntoResponses)]
#[response(status=CREATED)]
/// Data about the user that signed up
pub struct UserSignupResponseDto(pub UserSigninResponseDto);

impl IntoResponse for UserSignupResponseDto {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::CREATED, self.0).into_response()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, utoipa::ToSchema, utoipa::IntoResponses)]
#[response(status=OK)]
/// Result of a successfull login
pub struct UserSigninResponseDto {
    /// The token needed to access the API
    pub token: token::UserToken,
    #[serde(flatten)]
    pub user: dices_server_entities::user::Model,
}

impl IntoResponse for UserSigninResponseDto {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

#[derive(Debug, Error, utoipa::IntoResponses)]
pub enum SignupError {
    #[error("A user with this name already exists")]
    #[response(status=CONFLICT)]
    /// A user with that name already exists
    UserAlreadyExist,
    #[error("The user name cannot contain whitespaces")]
    #[response(status=BAD_REQUEST)]
    /// The user name cannot contain whitespaces
    WhitespacesInUsername,
    #[error("Database error")]
    #[response(status=INTERNAL_SERVER_ERROR)]
    /// Internal server error
    InternalServerError,
}
impl IntoResponse for SignupError {
    fn into_response(self) -> axum::response::Response {
        match self {
            SignupError::InternalServerError => {
                crate::internal_server_error(&self);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            SignupError::UserAlreadyExist => StatusCode::CONFLICT.into_response(),
            SignupError::WhitespacesInUsername => StatusCode::BAD_REQUEST.into_response(),
        }
    }
}

impl From<DbErr> for SignupError {
    fn from(value: DbErr) -> Self {
        crate::internal_server_error(&value);
        Self::InternalServerError
    }
}

#[derive(Debug, Error, Serialize, IntoResponses)]
pub enum SigninError {
    #[error("This user does not exist")]
    #[response(status=NOT_FOUND)]
    /// This user does not exist
    UserDoNotExist,
    #[error("Wrong password")]
    #[response(status=UNAUTHORIZED)]
    /// Wrong password provided
    WrongPassword,
    #[error("Internal server error")]
    #[response(status=INTERNAL_SERVER_ERROR)]
    /// Internal server error
    InternalServerError,
}

impl IntoResponse for SigninError {
    fn into_response(self) -> axum::response::Response {
        match self {
            SigninError::UserDoNotExist | SigninError::WrongPassword => {
                StatusCode::UNAUTHORIZED.into_response()
            }
            SigninError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

impl From<DbErr> for SigninError {
    fn from(value: DbErr) -> Self {
        crate::internal_server_error(&value);
        Self::InternalServerError
    }
}

impl From<argon2::password_hash::Error> for SigninError {
    fn from(value: argon2::password_hash::Error) -> Self {
        crate::internal_server_error(&value);
        Self::InternalServerError
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, FromRequest)]
#[from_request(via(axum::Json))]
#[derive(utoipa::ToSchema)]
pub struct UserUpdateDto {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, FromRequestParts)]
#[from_request(via(axum::extract::Path))]
pub struct UserPathData {
    #[serde(rename = "user", alias = "user-id")]
    pub id: UserId,
}

#[derive(Debug, Error, From, IntoResponses)]
pub enum UserGetError {
    #[error("User was deleted")]
    #[response(status=GONE)]
    /// The user was deleted
    Deleted,
    #[error("Internal Server Error")]
    #[response(status=INTERNAL_SERVER_ERROR)]
    /// Internal server error
    InternalServerError,
}

impl IntoResponse for UserGetError {
    fn into_response(self) -> axum::response::Response {
        match self {
            UserGetError::Deleted => StatusCode::GONE.into_response(),
            UserGetError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

impl From<DbErr> for UserGetError {
    fn from(value: DbErr) -> Self {
        crate::internal_server_error(&value);
        Self::InternalServerError
    }
}

#[derive(OpenApi)]
#[openapi(components(schemas(
    dices_server_entities::user::Model,
    UserQueryDto,
    UserSigninDto,
    UserSignupDto,
    UserSigninResponseDto,
    UserSignupResponseDto,
    UserUpdateDto
)))]
pub struct ApiComponents;
