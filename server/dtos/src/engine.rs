use axum::{
    async_trait,
    extract::{rejection::StringRejection, FromRequest, Request},
    response::IntoResponse,
    Json,
};
use derive_more::derive::{From, Into};
use dices_ast::{expression::ParseError, Expression};
use dices_server_entities::log;
use dices_server_intrisics::ServerIntrisics;
use http::StatusCode;
use nunny::NonEmpty;
use sea_orm::DbErr;
use serde::Serialize;
use thiserror::Error;
use tokio::task::JoinError;
use utoipa::{IntoResponses, ToSchema};

use crate::session::SessionGetError;

#[derive(Debug, Clone, Into)]
pub struct Command {
    pub source: String,
    pub value: Box<NonEmpty<[Expression<ServerIntrisics>]>>,
}

#[derive(Debug, Error, From, IntoResponses, Clone)]
pub enum CommandRejection {
    #[error("Session does not exist")]
    #[response(status=NOT_FOUND)]
    /// The session does not exist
    NotFound,
    #[error("Internal Server Error")]
    #[response(status=INTERNAL_SERVER_ERROR)]
    /// Internal Server Error
    InternalServerError,
    #[error("Command body cannot be parsed")]
    #[response(status=BAD_REQUEST)]
    /// Command body cannot be parsed
    CommandBodyCannotBeDeserialized,
    #[error("To issue commands one must be a player")]
    #[response(status=FORBIDDEN)]
    /// To issue commands one must be a player
    NotAPlayer,
}

impl IntoResponse for CommandRejection {
    fn into_response(self) -> axum::response::Response {
        match self {
            CommandRejection::InternalServerError => {
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            CommandRejection::CommandBodyCannotBeDeserialized => {
                StatusCode::BAD_REQUEST.into_response()
            }
            CommandRejection::NotAPlayer => StatusCode::FORBIDDEN.into_response(),
            CommandRejection::NotFound => StatusCode::NOT_FOUND.into_response(),
        }
    }
}

impl From<StringRejection> for CommandRejection {
    fn from(value: StringRejection) -> Self {
        match value {
            StringRejection::InvalidUtf8(_) => Self::CommandBodyCannotBeDeserialized,
            _ => {
                crate::internal_server_error(&value);
                Self::InternalServerError
            }
        }
    }
}
impl From<DbErr> for CommandRejection {
    fn from(value: DbErr) -> Self {
        crate::internal_server_error(&value);
        Self::InternalServerError
    }
}
impl From<JoinError> for CommandRejection {
    fn from(value: JoinError) -> Self {
        crate::internal_server_error(&value);
        Self::InternalServerError
    }
}
impl From<ParseError> for CommandRejection {
    fn from(_value: ParseError) -> Self {
        Self::CommandBodyCannotBeDeserialized
    }
}
impl From<SessionGetError> for CommandRejection {
    fn from(value: SessionGetError) -> Self {
        match value {
            SessionGetError::NotFound => Self::NotFound,
            SessionGetError::InternalServerError => Self::InternalServerError,
        }
    }
}

#[async_trait]
impl<S> FromRequest<S> for Command
where
    S: Send + Sync,
{
    type Rejection = CommandRejection;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let source = String::from_request(request, state).await?;
        let value = dices_ast::expression::parse_file(source.trim())?;
        Ok(Self { source, value })
    }
}

#[derive(Debug, Clone, Serialize, ToSchema, IntoResponses)]
#[response(status=OK)]
pub struct CommandResult(pub Box<[log::Model]>);

impl IntoResponse for CommandResult {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}
