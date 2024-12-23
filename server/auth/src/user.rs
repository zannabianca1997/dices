use axum::{
    async_trait,
    extract::{rejection::PathRejection, FromRequestParts},
    http::request::Parts,
};
use derive_more::derive::From;
use thiserror::Error;

use dices_server_dtos::{
    errors::{ErrorCode, ErrorResponse, ServerError},
    user::UserPathData,
};
use dices_server_entities::user::UserId;

use crate::{
    claims::{UserClaims, UserClaimsRejection},
    Autenticated,
};

#[async_trait]
impl<S> FromRequestParts<S> for Autenticated<UserId>
where
    S: Send + Sync,
    UserClaims: FromRequestParts<S>,
{
    type Rejection = <UserClaims as FromRequestParts<S>>::Rejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        UserClaims::from_request_parts(parts, state)
            .await
            .map(|claims| Autenticated(claims.subject))
    }
}

#[derive(Debug, Error, From)]
pub enum AuthenticatedUserPathRejection {
    #[error("No user authentication provided")]
    Header(UserClaimsRejection),
    #[error("Cannot extract path data")]
    Path(PathRejection),
    #[error("Invalid user id")]
    UnauthorizedId,
}

impl ServerError for AuthenticatedUserPathRejection {
    fn error_code(&self) -> dices_server_dtos::errors::ErrorCode {
        match self {
            AuthenticatedUserPathRejection::Header(user_claims_rejection) => {
                user_claims_rejection.error_code()
            }
            AuthenticatedUserPathRejection::Path(path_rejection) => path_rejection.error_code(),
            AuthenticatedUserPathRejection::UnauthorizedId => ErrorCode::UnauthorizedId,
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Autenticated<UserPathData>
where
    S: Send + Sync,
    Autenticated<UserId>: FromRequestParts<S, Rejection = ErrorResponse<UserClaimsRejection>>,
    UserPathData: FromRequestParts<S, Rejection = ErrorResponse<PathRejection>>,
{
    type Rejection = ErrorResponse<AuthenticatedUserPathRejection>;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Autenticated(authenticated_id) =
            Autenticated::from_request_parts(parts, state)
                .await
                .map_err(|err| AuthenticatedUserPathRejection::Header(err.0))?;
        let path_data @ UserPathData { id } = UserPathData::from_request_parts(parts, state)
            .await
            .map_err(|err| AuthenticatedUserPathRejection::Path(err.0))?;

        if authenticated_id == id {
            Ok(Autenticated(path_data))
        } else {
            Err(AuthenticatedUserPathRejection::UnauthorizedId.into())
        }
    }
}
