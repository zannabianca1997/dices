use chrono::{DateTime, Utc};
use derive_more::derive::{AsRef, Display, From, Into};
use derive_more::Error;
use sea_orm::{ConnectionTrait, DbErr, EnumIter, TransactionTrait};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domains::sessions::infrastructure::{
    create, create_session_user, destroy, fetch_users, find_all, find_by_id, find_session_user,
};
use crate::{
    domains::{
        commons::ErrorResponse,
        user::{AutenticatedUser, UserId},
    },
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

#[derive(Debug, Display, From, Error)]
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

#[derive(Debug, Display, From, Error)]
pub enum UserAddError {
    #[display("Users with role {creator:?} cannot add users with role {added:?}")]
    ProhibitedNewUserRole {
        creator: UserRole,
        added: UserRole,
    },
    DbErr(DbErr),
}
impl From<UserAddError> for ErrorResponse {
    fn from(value: UserAddError) -> Self {
        match value {
            UserAddError::ProhibitedNewUserRole { creator, added } => ErrorResponse::builder()
                .code(ErrorCodes::CannotAddUserWithHigherRole)
                .msg(format!(
                    "Users with role {creator} cannot add a user with role {added}"
                ))
                .add("creator_role", creator)
                .add("created_role", added)
                .build(),
            UserAddError::DbErr(db_err) => ErrorResponse::internal_server_error(db_err),
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Session {
    pub id: SessionId,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}
impl Session {
    pub async fn new(
        db: &(impl ConnectionTrait + TransactionTrait),
        SessionCreate { name, description }: SessionCreate,
        creator: AutenticatedUser,
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
        };

        create(session.clone(), db, creator.user_id()).await?;

        Ok(session)
    }

    pub(crate) async fn users(
        db: &(impl ConnectionTrait + TransactionTrait),
        session_uuid: SessionId,
        requester: AutenticatedUser,
    ) -> Result<impl Iterator<Item = Result<SessionUser, UsersGetNextError>>, UsersGetError> {
        let db = db.begin().await?;

        let (session, user) = find_by_id(&db, session_uuid, requester.user_id())
            .await?
            .ok_or_else(|| UsersGetError::SessionNotFound(session_uuid))?;

        if !(user.role.can(Permission::GetUsers)) {
            return Err(UsersGetError::CannotSeeUserList(user.role));
        }

        let users = fetch_users(&db, &session).await?;

        db.commit().await?;

        Ok(users)
    }

    pub(crate) async fn find_by_id(
        db: &impl ConnectionTrait,
        id: SessionId,
        requester: AutenticatedUser,
    ) -> Result<Option<Session>, DbErr> {
        Ok(find_by_id(db, id, requester.user_id())
            .await?
            .and_then(|(s, p)| p.role.can(Permission::GetData).then_some(s)))
    }

    pub(crate) async fn find_all(
        db: &(impl ConnectionTrait + TransactionTrait),
        requester: AutenticatedUser,
    ) -> Result<impl Iterator<Item = Result<Session, SessionsGetNextError>>, DbErr> {
        Ok(find_all(db, requester.user_id()).await?.filter_map(|r| {
            r.map(|(s, p)| p.role.can(Permission::GetData).then_some(s))
                .transpose()
        }))
    }

    pub(crate) async fn delete(
        db: &(impl ConnectionTrait + TransactionTrait),
        session_uuid: SessionId,
        requester: AutenticatedUser,
    ) -> Result<(), DeleteSessionError> {
        db.transaction(|db| {
            Box::pin(async move {
                let user = find_session_user(db, session_uuid, requester.user_id())
                    .await?
                    .ok_or_else(|| DeleteSessionError::SessionNotFound(session_uuid))?;

                if !(user.role.can(Permission::Delete)) {
                    return Err(DeleteSessionError::RoleCannotDelete(user.role));
                }

                destroy(db, session_uuid).await?;
                Ok(())
            })
        })
        .await
        .map_err(|err| match err {
            sea_orm::TransactionError::Connection(db_err) => db_err.into(),
            sea_orm::TransactionError::Transaction(err) => err,
        })
    }
}

#[derive(Debug, Display, From, Error)]
pub enum DeleteSessionError {
    SessionNotFound(#[error(not(source))] SessionId),
    DbErr(DbErr),
    RoleCannotDelete(#[error(not(source))] UserRole),
}
impl From<DeleteSessionError> for ErrorResponse {
    fn from(value: DeleteSessionError) -> Self {
        match value {
            DeleteSessionError::SessionNotFound(session_uuid) => ErrorResponse::builder()
                .code(ErrorCodes::SessionNotFound)
                .msg(format!("The session {session_uuid} does not exist"))
                .add("uuid", session_uuid)
                .build(),
            DeleteSessionError::DbErr(db_err) => db_err.into(),
            DeleteSessionError::RoleCannotDelete(user_role) => ErrorResponse::builder()
                .code(ErrorCodes::CannotDeleteSession)
                .msg(format!(
                    "Users with role {user_role} cannot delete sessions"
                ))
                .add("user_role", user_role)
                .build(),
        }
    }
}

#[derive(Debug, Display, From, Error)]
pub enum UsersGetError {
    SessionNotFound(#[error(not(source))] SessionId),
    CannotSeeUserList(#[error(not(source))] UserRole),
    DbErr(DbErr),
}
impl From<UsersGetError> for ErrorResponse {
    fn from(value: UsersGetError) -> Self {
        match value {
            UsersGetError::SessionNotFound(session_uuid) => ErrorResponse::builder()
                .code(ErrorCodes::SessionNotFound)
                .msg(format!("The session {session_uuid} does not exist"))
                .add("uuid", session_uuid)
                .build(),
            UsersGetError::CannotSeeUserList(role) => ErrorResponse::builder()
                .code(ErrorCodes::CannotSeeUserList)
                .msg(format!(
                    "The user with role {role} cannot see the user list"
                ))
                .build(),
            UsersGetError::DbErr(err) => err.into(),
        }
    }
}

#[derive(Debug, Display, From, Error)]
pub enum UsersGetNextError {}
impl From<UsersGetNextError> for ErrorResponse {
    fn from(value: UsersGetNextError) -> Self {
        match value {}
    }
}

#[derive(Debug, Display, From, Error)]
pub enum SessionsGetNextError {
    DbErr(DbErr),
}
impl From<SessionsGetNextError> for ErrorResponse {
    fn from(value: SessionsGetNextError) -> Self {
        match value {
            SessionsGetNextError::DbErr(db_err) => db_err.into(),
        }
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

/// Data needed to add a user to a session
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct JoinSession {
    /// Role this user will take in this session
    pub role: UserRole,
}

/// Role a user can have in a session
#[derive(
    Debug, Clone, Copy, Deserialize, Serialize, ToSchema, Eq, PartialEq, EnumIter, Display,
)]
pub enum UserRole {
    /// User that can play and modify users in the session
    Admin,
    /// Player
    Player,
    /// User that can only watch
    Observer,
}

/// Category of actions that a user might want to do with a session he's in
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(
    test,
    derive(strum::EnumDiscriminants),
    strum_discriminants(derive(EnumIter))
)]
pub enum Permission {
    /// Obtain general data
    GetData,
    /// Set general data
    SetData,
    /// Get the history
    GetHistory,
    /// Delete the session
    Delete,
    /// Get the user list
    GetUsers,
    /// Add a user with the given role
    AddUser { role: UserRole },
    /// Remove a user with the given role
    RemoveUser { role: UserRole },
    /// Remove itself from the session
    RemoveSelf,
    /// Set the role of a user
    SetRole {
        /// Initial role of the user
        from: UserRole,
        /// New role of the user
        to: UserRole,
    },
    /// Change own role
    SetSelfRole { to: UserRole },
    /// Send a command
    SendCommand,
}

impl UserRole {
    pub fn can(self, permission: Permission) -> bool {
        use Permission::*;
        use UserRole::*;

        match (self, permission) {
            // Anyone can exit
            (_, RemoveSelf) => true,
            // Those operations are really no-op, and can be permitted
            (_, SetRole { from, to }) if from == to => true,
            (_, SetSelfRole { to }) if self == to => true,

            // Admins cannot remove other admins
            (Admin, RemoveUser { role: Admin } | SetRole { from: Admin, to: _ }) => false,
            // Otherwise, they are omnipotent
            (Admin, _) => true,

            // Users can read session data, users and history.
            // They can send command and make themselves observers
            (
                Player,
                GetData | GetUsers | GetHistory | SendCommand | SetSelfRole { to: Observer },
            ) => true,

            // Observers can see the session data and history
            (Observer, GetData | GetHistory) => true,

            // All other actions are prohibited
            _ => false,
        }
    }
}

#[cfg(test)]
mod permissions_constraints;

/// Relationship between a user and a session
#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
pub struct SessionUser {
    /// The session
    pub session: SessionId,
    /// The user
    pub user: UserId,
    /// The role the user has in the session
    pub role: UserRole,
    /// When the user was added
    pub added_at: DateTime<Utc>,
    /// Last time the user interacted with the session
    pub last_access: Option<DateTime<Utc>>,
}

impl SessionUser {
    pub async fn add_new(
        &self,
        db: &impl ConnectionTrait,
        user: UserId,
        JoinSession { role }: JoinSession,
    ) -> Result<Self, UserAddError> {
        if !self.role.can(Permission::AddUser { role }) {
            return Err(UserAddError::ProhibitedNewUserRole {
                creator: self.role,
                added: role,
            });
        }

        let new_user = Self {
            session: self.session,
            user,
            role,
            added_at: Utc::now(),
            last_access: None,
        };
        create_session_user(new_user, db).await?;
        Ok(new_user)
    }
}
