// Logic to parse user claims from the authentication header

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use argon2::{password_hash::SaltString, Argon2, PasswordHasher, PasswordVerifier};
use axum::{
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
    response::IntoResponse,
    Json,
};
use axum_extra::headers::Authorization;
use chrono::{DateTime, FixedOffset, Local};
use derive_more::derive::From;
use dices_server_dtos::user::token::{AuthHeaderRejection, UserToken};
use dices_server_entities::user::{PasswordHash, UserId};
use jwt::{claims::SecondsSinceEpoch, SignWithKey as _, VerifyWithKey as _};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::{ToResponse, ToSchema};

use crate::{auth_key::AuthKey, Autenticated};

#[derive(Debug, Error, From, Serialize, ToResponse, ToSchema)]
#[error("The token was received at {received_at}, but it expired at {expiration}")]
pub struct ExpiredToken {
    pub expiration: DateTime<FixedOffset>,
    pub received_at: DateTime<FixedOffset>,
}

impl IntoResponse for ExpiredToken {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::UNAUTHORIZED, Json(self)).into_response()
    }
}

#[derive(Debug, Error, From, Serialize)]
pub enum InvalidTokenError {
    #[error(transparent)]
    Expired(ExpiredToken),
    #[error("The token is malformed")]
    Malformed,
}

impl IntoResponse for InvalidTokenError {
    fn into_response(self) -> axum::response::Response {
        match self {
            InvalidTokenError::Expired(expired_token) => expired_token.into_response(),
            InvalidTokenError::Malformed => StatusCode::BAD_REQUEST.into_response(),
        }
    }
}

/// Claims for a user token
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UserClaims {
    #[serde(rename = "sub")]
    pub subject: UserId,
    #[serde(rename = "exp")]
    pub expiration: SecondsSinceEpoch,
    #[serde(rename = "iat")]
    pub issued_at: SecondsSinceEpoch,
}

#[derive(Debug, Error, From)]
pub enum UserClaimsRejection {
    #[error(transparent)]
    TokenRejection(AuthHeaderRejection),
    #[error(transparent)]
    InvalidToken(InvalidTokenError),
}

impl IntoResponse for UserClaimsRejection {
    fn into_response(self) -> axum::response::Response {
        match self {
            UserClaimsRejection::TokenRejection(auth_header_rejection) => {
                auth_header_rejection.into_response()
            }
            UserClaimsRejection::InvalidToken(invalid_token_error) => {
                invalid_token_error.into_response()
            }
        }
    }
}

impl<S> FromRequestParts<S> for UserClaims
where
    S: Send + Sync,
    AuthKey: FromRef<S>,
{
    type Rejection = UserClaimsRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = UserToken::from_request_parts(parts, state)
            .await
            .map_err(UserClaimsRejection::TokenRejection)?;
        let auth_key = AuthKey::from_ref(state);
        parse_token(auth_header.0.token(), auth_key).map_err(UserClaimsRejection::InvalidToken)
    }
}

fn parse_token(token: &str, auth_key: AuthKey) -> Result<UserClaims, InvalidTokenError> {
    let received_at = SystemTime::now();
    match token.verify_with_key(&auth_key) {
        Ok(UserClaims { expiration, .. })
            if SystemTime::UNIX_EPOCH + Duration::from_secs(expiration) < received_at =>
        {
            Err(InvalidTokenError::Expired(ExpiredToken {
                expiration: DateTime::<Local>::from(
                    SystemTime::UNIX_EPOCH + Duration::from_secs(expiration),
                )
                .into(),
                received_at: DateTime::<Local>::from(received_at).into(),
            }))
        }
        Err(err) => Err({
            tracing::debug!("Token {token} was refused: {err}");
            InvalidTokenError::Malformed
        }),
        Ok(claims) => Ok(claims),
    }
}

/// Create a password hash to store safely passwords in the database
pub fn hash_password(id: UserId, password: &str) -> (Autenticated<UserId>, PasswordHash) {
    (
        Autenticated(id), // we can give this, as the hash was here
        Argon2::default()
            .hash_password(
                password.as_bytes(),
                &SaltString::encode_b64(id.as_bytes())
                    .expect("Uuids should be always able to be made into salts"),
            )
            .expect("Argon2 should be infallible")
            .to_string()
            .into(),
    )
}

pub fn new_token(id: Autenticated<UserId>, auth_key: AuthKey) -> UserToken {
    let issued_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let expiration = (SystemTime::now() + auth_key.token_validity())
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let claims = UserClaims {
        subject: id.into_inner(),
        expiration,
        issued_at,
    };

    let token = claims
        .sign_with_key(&auth_key)
        .expect("The signing process should be infallible");

    UserToken(Authorization::bearer(&token).expect("The token should be a valid bearer token"))
}

pub use argon2::password_hash::Error as CheckPasswordError;

pub fn check_password(
    id: UserId,
    hash: &PasswordHash,
    provided: String,
) -> Result<Autenticated<UserId>, CheckPasswordError> {
    Argon2::default()
        .verify_password(
            provided.as_bytes(),
            &argon2::PasswordHash::new(hash.as_str())?,
        )
        .map(|()| Autenticated(id))
}
