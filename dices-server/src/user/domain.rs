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

        pub const fn as_bytes(&self) -> &uuid::Bytes {
            self.0.as_bytes()
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
            check_password(self.id, &self.password, password)
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
    use argon2::{
        password_hash::{Salt, SaltString},
        Argon2, PasswordHasher as _, PasswordVerifier as _,
    };

    use super::models::UserId;

    #[derive(Debug, Clone)]
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

    /// Create a password hash to store safely passwords in the database
    pub(super) fn hash_password(id: UserId, password: &str) -> PasswordHash {
        PasswordHash(
            Argon2::default()
                .hash_password(
                    password.as_bytes(),
                    &SaltString::encode_b64(id.as_bytes())
                        .expect("Uuids should be always able to be made into salts"),
                )
                .expect("Argon2 should be infallible")
                .to_string(),
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
}
