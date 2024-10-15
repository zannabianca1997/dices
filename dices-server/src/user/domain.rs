pub mod models {
    use chrono::{DateTime, Utc};
    use derive_more::derive::{AsRef, Debug, Display};
    use jwt::{claims::SecondsSinceEpoch, token::Signed, Claims, Header, RegisteredClaims, Token};
    use serde::{Deserialize, Serialize};
    use utoipa::{
        openapi::{schema, License},
        ToSchema,
    };
    use uuid::Uuid;

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

    #[derive(Debug, Deserialize, ToSchema)]
    /// Request a new pair of token/refresh token
    pub struct RefreshRequest {
        /// The refresh token
        pub refresh_token: String,
    }

    #[derive(Debug, Display, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, AsRef)]
    #[repr(transparent)]
    /// An ID uniquely identifying a user
    pub struct UserId(Uuid);

    impl UserId {
        pub fn new() -> Self {
            Self(Uuid::new_v4())
        }
    }

    #[derive(Debug, Clone, Serialize, ToSchema)]
    /// Info about a user
    pub struct User {
        /// The unique user ID
        #[serde(skip)]
        pub id: UserId,

        /// The username
        pub username: String,

        /// The hashed password
        #[serde(skip)]
        password: PasswordHash,

        /// The datetime at which the user was created
        #[schema(read_only)]
        pub created: DateTime<Utc>,
        /// The last access of the user
        #[schema(read_only)]
        pub last_access: DateTime<Utc>,
    }
    impl User {
        pub fn new(RegisterRequest { username, password }: RegisterRequest) -> Self {
            let id = UserId::new();
            let password = hash_password(id, &password);
            let created = Utc::now();
            Self {
                id,
                username,
                password,
                created,
                last_access: created,
            }
        }

        pub fn authenticate(&self, password: &str) -> Option<AutenticatedUser> {
            check_password(self.id, self.password, password)
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
        /// The single use refresh token
        ///
        /// It has a long duration, and can be used to request a new valid token pair
        refresh_token: String,
    }

    #[derive(Debug, Serialize, ToSchema)]
    /// Successfull refreshed token
    pub struct RefreshResponse {
        /// The token used to access the API
        ///
        /// It has a limited duration, and need to be periodically refreshed
        token: String,
        /// The single use refresh token
        ///
        /// It has a long duration, and can be used to request a new valid token pair
        refresh_token: String,
    }

    /// Claims for a user token
    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    pub struct UserClaims {
        #[serde(rename = "iss")]
        pub issuer: &'static str,
        #[serde(rename = "sub")]
        pub subject: UserId,
        #[serde(rename = "exp")]
        pub expiration: SecondsSinceEpoch,
        #[serde(rename = "iat")]
        pub issued_at: SecondsSinceEpoch,
    }
}

mod security {
    use std::mem::size_of;

    use argon2::{Argon2, Params};

    use super::models::UserId;

    #[derive(Debug, Clone, Copy)]
    #[repr(transparent)]
    pub struct PasswordHash([u8; 32]);

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

    /// Generate the password hasher
    ///
    /// The params should be in the future readed from the db
    fn hasher() -> Argon2<'static> {
        Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x10,
            Params::new(19 * 1024, 2, 1, Some(size_of::<PasswordHash>()))
                .expect("The params should be valid"),
        )
    }

    /// Create a password hash to store safely passwords in the database
    pub(super) fn hash_password(id: UserId, password: &str) -> PasswordHash {
        let mut hash = PasswordHash([0; size_of::<PasswordHash>()]);
        hasher()
            .hash_password_into(password.as_bytes(), id.as_ref().as_bytes(), &mut hash.0)
            .expect("The hashing procedure should be infallible");
        hash
    }

    /// Check if a password matches the one in the db
    pub(super) fn check_password(
        id: UserId,
        stored: PasswordHash,
        provided: &str,
    ) -> Option<AutenticatedUser> {
        let hashed = hash_password(id, provided);
        (hashed.0 == stored.0).then(|| AutenticatedUser { user_id: id })
    }
}
