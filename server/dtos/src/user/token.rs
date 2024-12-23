use axum::{async_trait, extract::FromRequestParts};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    typed_header::TypedHeaderRejection,
    TypedHeader,
};
use derive_more::derive::From;
use http::request::Parts;
use serde::Serialize;

use thiserror::Error;

use crate::errors::{ErrorCode, ErrorResponse, ServerError};

#[derive(Clone, Debug, From)]
pub struct UserToken(pub Authorization<Bearer>);

impl Serialize for UserToken {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.token().serialize(serializer)
    }
}
impl PartialEq for UserToken {
    fn eq(&self, other: &Self) -> bool {
        self.0.token() == other.0.token()
    }
}
impl Eq for UserToken {}

#[derive(Debug, Error, From)]
#[error("Cannot parse authorization header")]
pub struct AuthHeaderRejection(#[source] TypedHeaderRejection);

impl ServerError for AuthHeaderRejection {
    fn error_code(&self) -> ErrorCode {
        match self.0.reason() {
            axum_extra::typed_header::TypedHeaderRejectionReason::Missing => {
                ErrorCode::MissingAuthHeader
            }
            _ => ErrorCode::InvalidAuthHeader,
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for UserToken
where
    S: Send + Sync,
{
    type Rejection = ErrorResponse<AuthHeaderRejection>;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(header) = TypedHeader::from_request_parts(parts, state)
            .await
            .map_err(AuthHeaderRejection)?;
        Ok(UserToken(header))
    }
}
