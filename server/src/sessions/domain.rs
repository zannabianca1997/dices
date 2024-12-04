pub mod models {
    use chrono::{DateTime, Utc};
    use derive_more::derive::{AsRef, Display, From, Into};
    use dices_ast::intrisics::NoInjectedIntrisics;
    use rand::rngs::SmallRng;
    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;
    use uuid::Uuid;

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

    type ServerRNG = SmallRng;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Session {
        pub id: Uuid,
        pub name: String,
        pub description: Option<String>,
        pub created_at: DateTime<Utc>,
        #[serde(skip)]
        pub image: Option<dices_engine::Engine<ServerRNG, NoInjectedIntrisics>>,
    }
}
