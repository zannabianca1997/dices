use axum::{
    async_trait,
    extract::{rejection::PathRejection, FromRequestParts},
    http::{request::Parts, StatusCode},
    response::IntoResponse,
    Json,
};
use derive_more::derive::From;
use serde::Serialize;
use thiserror::Error;

use dices_server_dtos::user::UserPathData;
use dices_server_entities::user::UserId;
use utoipa::{openapi::SecurityRequirement, Modify};

use crate::{
    claims::{UserClaims, UserClaimsRejection},
    Autenticated,
};

#[async_trait]
impl<S> FromRequestParts<S> for Autenticated<UserId>
where
    S: Send + Sync,
    UserClaims: FromRequestParts<S, Rejection = UserClaimsRejection>,
{
    type Rejection = UserClaimsRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        UserClaims::from_request_parts(parts, state)
            .await
            .map(|claims| Autenticated(claims.subject))
    }
}

#[derive(Debug, Error, Serialize)]
#[error("Invalid user id")]
pub struct UnauthorizedId {
    pub authenticated: UserId,
    pub requested: UserId,
}

impl IntoResponse for UnauthorizedId {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::FORBIDDEN, Json(self)).into_response()
    }
}

#[derive(Debug, Error, From)]
pub enum AuthenticatedUserPathRejection {
    #[error("No user authentication provided")]
    Header(UserClaimsRejection),
    #[error("Cannot extract path data")]
    Path(PathRejection),
    #[error(transparent)]
    UnauthorizedId(UnauthorizedId),
}
impl IntoResponse for AuthenticatedUserPathRejection {
    fn into_response(self) -> axum::response::Response {
        match self {
            AuthenticatedUserPathRejection::Header(user_claims_rejection) => {
                user_claims_rejection.into_response()
            }
            AuthenticatedUserPathRejection::Path(path_rejection) => path_rejection.into_response(),
            AuthenticatedUserPathRejection::UnauthorizedId(unauthorized_id) => {
                unauthorized_id.into_response()
            }
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Autenticated<UserPathData>
where
    S: Send + Sync,
    Autenticated<UserId>: FromRequestParts<S, Rejection = UserClaimsRejection>,
    UserPathData: FromRequestParts<S, Rejection = PathRejection>,
{
    type Rejection = AuthenticatedUserPathRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Autenticated(authenticated_id) = Autenticated::from_request_parts(parts, state)
            .await
            .map_err(AuthenticatedUserPathRejection::Header)?;
        let path_data @ UserPathData { id } = UserPathData::from_request_parts(parts, state)
            .await
            .map_err(AuthenticatedUserPathRejection::Path)?;

        if authenticated_id == id {
            Ok(Autenticated(path_data))
        } else {
            Err(UnauthorizedId {
                authenticated: authenticated_id,
                requested: id,
            }
            .into())
        }
    }
}

pub struct RequireUserToken;

impl Modify for RequireUserToken {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        for path in openapi.paths.paths.values_mut() {
            for op in [
                &mut path.get,
                &mut path.head,
                &mut path.trace,
                &mut path.put,
                &mut path.post,
                &mut path.patch,
                &mut path.delete,
            ]
            .into_iter()
            .flatten()
            {
                op.security
                    .get_or_insert_default()
                    .push(SecurityRequirement::new::<_, _, &str>("user_token", []));
            }
        }
    }
}
