use chrono::{DateTime, Utc};
use derive_more::derive::{AsRef, Debug, Display, From, Into};
use jwt::claims::SecondsSinceEpoch;
use sea_orm::{ConnectionTrait, DbErr};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    app::AuthKey,
    domains::{
        commons::{ErrorCodes, ErrorResponse},
        user::domain::security,
    },
};

use super::security::{check_password, hash_password, AutenticatedUser, PasswordHash};

#[derive(Debug, Deserialize, ToSchema)]
/// Info to login in the service
pub struct LoginRequest {
    /// The username
    pub username: String,
    /// The password
    pub password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
/// Info to register into the service
pub struct RegisterRequest {
    /// The username
    pub username: String,
    /// The password
    pub password: String,
}

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
/// An ID uniquely identifying a user
pub struct UserId(Uuid);

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

impl UserId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub const fn as_bytes(&self) -> &uuid::Bytes {
        self.0.as_bytes()
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
/// Info about a user
pub struct User {
    /// The unique user ID
    pub id: UserId,

    /// The username
    pub username: String,

    /// The hashed password
    #[serde(skip)]
    pub password: PasswordHash,

    /// The datetime at which the user was created
    pub created_at: DateTime<Utc>,
    /// The last access of the user
    pub last_access: DateTime<Utc>,
}
impl User {
    pub async fn new(
        db: &impl ConnectionTrait,
        RegisterRequest { username, password }: RegisterRequest,
    ) -> Result<(Self, AutenticatedUser), RegistrationError> {
        let username = username.trim().to_owned();
        if username.is_empty() {
            return Err(RegistrationError::BlankUsername);
        }
        if username.contains(char::is_whitespace) {
            return Err(RegistrationError::SpacesInUsername(username));
        }
        if Self::exist_by_name(db, &username).await? {
            return Err(RegistrationError::UsernameTaken(username));
        }

        let id = UserId::new();
        let (password, authenticated) = hash_password(id, &password);
        let created_at = Utc::now();
        let user = Self {
            id,
            username,
            password,
            created_at,
            last_access: created_at,
        };

        Ok(user
            .clone()
            .save(db)
            .await
            .map(|()| (user, authenticated))?)
    }

    pub(crate) async fn login(
        db: &impl ConnectionTrait,
        LoginRequest { username, password }: LoginRequest,
    ) -> Result<(Self, AutenticatedUser), LoginError> {
        let username = username.trim().to_owned();
        let Some(user) = Self::find_by_name(db, &username).await? else {
            return Err(LoginError::UnknowUsername(username));
        };
        let Some(authenticated) = check_password(user.id, &user.password, &password) else {
            return Err(LoginError::WrongPassword);
        };
        // access done!
        let user = user.update_last_access(db).await?;

        Ok((user, authenticated))
    }
}

#[derive(Debug, From)]
pub enum RegistrationError {
    BlankUsername,
    #[from(skip)]
    SpacesInUsername(String),
    DbErr(DbErr),
    #[from(skip)]
    UsernameTaken(String),
}
impl From<RegistrationError> for ErrorResponse {
    fn from(value: RegistrationError) -> Self {
        match value {
            RegistrationError::BlankUsername => ErrorResponse::builder()
                .code(ErrorCodes::BlankUsername)
                .msg("The username cannot be blank")
                .build(),
            RegistrationError::SpacesInUsername(username) => ErrorResponse::builder()
                .code(ErrorCodes::UsernameHasSpaces)
                .msg(format!("The username `{username}` contains blank spaces"))
                .add("username", username)
                .build(),
            RegistrationError::DbErr(db_err) => ErrorResponse::internal_server_error(db_err),
            RegistrationError::UsernameTaken(username) => ErrorResponse::builder()
                .code(ErrorCodes::UsernameTaken)
                .msg(format!("The username `{username}` is already taken"))
                .add("username", username)
                .build(),
        }
    }
}

#[derive(Debug, From)]
pub enum LoginError {
    #[from(skip)]
    UnknowUsername(String),
    WrongPassword,
    DbErr(DbErr),
}
impl From<LoginError> for ErrorResponse {
    fn from(value: LoginError) -> Self {
        match value {
            LoginError::DbErr(db_err) => ErrorResponse::internal_server_error(db_err),
            LoginError::UnknowUsername(username) => ErrorResponse::builder()
                .code(ErrorCodes::UnknowUsername)
                .msg(format!("No user named `{username}`"))
                .add("username", username)
                .build(),
            LoginError::WrongPassword => ErrorResponse::builder()
                .code(ErrorCodes::WrongPassword)
                .msg("Wrong password")
                .build(),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
/// Successfull signin/registration
pub struct SignInResponse {
    /// The signed in user
    user: User,
    /// The token used to access the API
    ///
    /// It has a limited duration, and need to be periodically refreshed
    token: String,
}
impl SignInResponse {
    pub fn new(user: User, auth: AutenticatedUser, auth_key: AuthKey) -> Self {
        assert_eq!(user.id, auth.user_id());
        let token = security::generate_token(auth, auth_key);
        Self { user, token }
    }
}

#[derive(Debug, Serialize, ToSchema)]
/// Successfull refreshed token
pub struct RefreshResponse {
    /// The token used to access the API
    ///
    /// It has a limited duration, and need to be periodically refreshed
    pub token: String,
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
