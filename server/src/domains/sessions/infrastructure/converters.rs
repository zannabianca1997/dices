use bincode::error::DecodeError;
use chrono::DateTime;

use crate::{
    domains::sessions::domain::models::{Session, SessionId, SessionUser, UserRole},
    entities,
};

impl TryFrom<entities::session::Model> for Session {
    type Error = DecodeError;

    fn try_from(
        entities::session::Model {
            id,
            name,
            description,
            created_at,
            image,
        }: entities::session::Model,
    ) -> Result<Self, Self::Error> {
        Ok(Session {
            id: SessionId::from(id),
            name,
            description,
            created_at: created_at.to_utc(),
            image: image
                .map(|bytes| bincode::decode_from_slice(&bytes, bincode::config::standard()))
                .transpose()?
                .map(|(i, _)| i),
        })
    }
}

impl From<Session> for entities::session::Model {
    fn from(
        Session {
            id,
            name,
            description,
            created_at,
            image,
        }: Session,
    ) -> Self {
        Self {
            id: id.into(),
            name,
            description,
            created_at: created_at.fixed_offset(),
            image: image
                .map(|image| bincode::encode_to_vec(image, bincode::config::standard()))
                .transpose()
                .expect("The engine should be always encodable"),
        }
    }
}

impl From<entities::sea_orm_active_enums::UserRole> for UserRole {
    fn from(value: entities::sea_orm_active_enums::UserRole) -> Self {
        use entities::sea_orm_active_enums::UserRole as DbUserRole;
        match value {
            DbUserRole::Admin => UserRole::Admin,
            DbUserRole::Observer => UserRole::Observer,
            DbUserRole::Player => UserRole::Player,
        }
    }
}

impl From<UserRole> for entities::sea_orm_active_enums::UserRole {
    fn from(value: UserRole) -> Self {
        use entities::sea_orm_active_enums::UserRole as DbUserRole;
        match value {
            UserRole::Admin => DbUserRole::Admin,
            UserRole::Observer => DbUserRole::Observer,
            UserRole::Player => DbUserRole::Player,
        }
    }
}

impl From<entities::session_user::Model> for SessionUser {
    fn from(
        entities::session_user::Model {
            session,
            user,
            role,
            added_at,
            last_access,
        }: entities::session_user::Model,
    ) -> Self {
        SessionUser {
            session: session.into(),
            user: user.into(),
            role: role.into(),
            added_at: added_at.to_utc(),
            last_access: last_access.as_ref().map(DateTime::to_utc),
        }
    }
}

impl From<SessionUser> for entities::session_user::Model {
    fn from(
        SessionUser {
            session,
            user,
            role,
            added_at,
            last_access,
        }: SessionUser,
    ) -> Self {
        entities::session_user::Model {
            session: session.into(),
            user: user.into(),
            role: role.into(),
            added_at: added_at.fixed_offset(),
            last_access: last_access.as_ref().map(DateTime::fixed_offset),
        }
    }
}
