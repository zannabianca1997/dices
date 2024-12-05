use chrono::{DateTime, Utc};
use derive_more::derive::{AsRef, Display, From, Into};
use dices_ast::intrisics::NoInjectedIntrisics;
use rand_xoshiro::Xoshiro256PlusPlus;
use sea_orm::{ConnectionTrait, DbErr};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    domains::{commons::ErrorResponse, user::AutenticatedUser},
    ErrorCodes,
};

#[derive(
    Debug,
    Display,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    AsRef,
    From,
    Into,
    ToSchema,
)]
#[repr(transparent)]
/// An ID uniquely identifying a sessions
pub struct SessionId(Uuid);

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub const fn as_bytes(&self) -> &uuid::Bytes {
        self.0.as_bytes()
    }
}

#[derive(Debug, From)]
pub enum SessionCreateError {
    BlankName,
    DbErr(DbErr),
}
impl From<SessionCreateError> for ErrorResponse {
    fn from(value: SessionCreateError) -> Self {
        match value {
            SessionCreateError::BlankName => ErrorResponse::builder()
                .code(ErrorCodes::BlankSessionName)
                .msg("The session name cannot be blank")
                .build(),
            SessionCreateError::DbErr(db_err) => ErrorResponse::internal_server_error(db_err),
        }
    }
}

type ServerRNG = Xoshiro256PlusPlus;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Session {
    pub id: SessionId,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    #[serde(skip)]
    pub image: Option<dices_engine::Engine<ServerRNG, NoInjectedIntrisics>>,
}

impl Session {
    pub async fn new(
        db: &impl ConnectionTrait,
        SessionCreate { name, description }: SessionCreate,
        _: AutenticatedUser,
    ) -> Result<Self, SessionCreateError> {
        let name = name.trim().to_owned();
        if name.is_empty() {
            return Err(SessionCreateError::BlankName);
        }

        let id = SessionId::new();
        let created_at = Utc::now();
        let session = Self {
            id,
            name,
            created_at,
            description,
            image: None,
        };

        Ok(session.clone().save(db).await.map(|()| session)?)
    }
}

/// Data needed to create a session
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct SessionCreate {
    /// The name of the session
    pub name: String,
    /// Oprional description of the session
    pub description: Option<String>,
}
