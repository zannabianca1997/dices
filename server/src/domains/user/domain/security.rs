use std::time::{Duration, SystemTime, UNIX_EPOCH};

use argon2::{password_hash::SaltString, Argon2, PasswordHasher as _, PasswordVerifier as _};
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use derive_more::derive::{From, Into};

use crate::{
    app::AuthKey,
    domains::commons::{ErrorCodes, ErrorResponse, ErrorResponseBuilder},
};

use super::models::{UserClaims, UserId};

#[derive(Debug, Clone, From, Into)]
#[repr(transparent)]
pub struct PasswordHash(String);

/// Represent a successfull autenticated user
///
/// This type cannot be built outside of the `security` module
#[derive(Debug, Clone, Copy)]
pub struct AutenticatedUser {
    user_id: UserId,
}

impl AutenticatedUser {
    pub fn id(&self) -> UserId {
        self.user_id
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AutenticatedUser
where
    S: Send + Sync,
    AuthKey: FromRef<S>,
{
    type Rejection = ErrorResponse;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Search for the api key header
        let Ok(auth_header) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state).await
        else {
            return Err(ErrorResponseBuilder::new()
                .code(ErrorCodes::InvalidAuthHeader)
                .msg("The `Authorization` header is either missing or invalid")
                .build());
        };
        // Extract the signing key
        let auth_key = AuthKey::from_ref(state);
        // Check the token
        check_token(auth_header.token(), auth_key).map_err(Into::into)
    }
}

/// Create a password hash to store safely passwords in the database
pub(super) fn hash_password(id: UserId, password: &str) -> (PasswordHash, AutenticatedUser) {
    (
        PasswordHash(
            Argon2::default()
                .hash_password(
                    password.as_bytes(),
                    &SaltString::encode_b64(id.as_bytes())
                        .expect("Uuids should be always able to be made into salts"),
                )
                .expect("Argon2 should be infallible")
                .to_string(),
        ),
        AutenticatedUser { user_id: id }, // we can give this, as it was just built
    )
}

/// Check if a password matches the one in the db
pub(super) fn check_password(
    id: UserId,
    stored: &PasswordHash,
    provided: &str,
) -> Option<AutenticatedUser> {
    argon2::PasswordHash::new(&stored.0)
        .and_then(|hash| {
            Argon2::default()
                .verify_password(provided.as_bytes(), &hash)
                .map(|_| Some(AutenticatedUser { user_id: id }))
                .or_else(|err| match err {
                    argon2::password_hash::Error::Password => Ok(None),
                    _ => Err(err),
                })
        })
        .unwrap_or_else(|err| {
            tracing::error!("Error during password checking of user {id}: {err}");
            None
        })
}

use jwt::{SignWithKey, VerifyWithKey};

/// Generate a new auth token
pub(crate) fn generate_token(auth: AutenticatedUser, auth_key: AuthKey) -> String {
    let issued_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let expiration = (SystemTime::now() + auth_key.token_validity())
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let token_claims = UserClaims {
        subject: auth.id(),
        expiration,
        issued_at,
    };
    token_claims
        .sign_with_key(&auth_key)
        .expect("The signing process should be infallible")
}

fn check_token(token: &str, auth_key: AuthKey) -> Result<AutenticatedUser, ErrorResponse> {
    match token.verify_with_key(&auth_key) {
        Ok(UserClaims { expiration, .. })
            if SystemTime::UNIX_EPOCH + Duration::from_secs(expiration) < SystemTime::now() =>
        {
            Err({
                ErrorResponseBuilder::new()
                    .code(ErrorCodes::TokenExpired)
                    .msg("The provided bearer token is expired")
                    .add("provided_token", token)
                    .add(
                        "expiration",
                        SystemTime::UNIX_EPOCH + Duration::from_secs(expiration),
                    )
                    .add("received_at", SystemTime::now())
                    .build()
            })
        }
        Err(err) => Err({
            tracing::debug!("Token {token} was refused for a serialization error: {err}");
            ErrorResponseBuilder::new()
                .code(ErrorCodes::InvalidToken)
                .msg("The provided bearer token is invalid")
                .add("provided_token", token)
                .build()
        }),
        Ok(UserClaims {
            subject: user_id, ..
        }) => Ok(AutenticatedUser { user_id }),
    }
}
