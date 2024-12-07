use super::domain::models::{
    Session, SessionId, SessionUser, UserRole, UsersGetError, UsersGetNextError,
};
use crate::domains::user::UserId;
use crate::entities;
use bincode::error::DecodeError;
use chrono::{DateTime, Utc};
use sea_orm::{
    ConnectionTrait, DbErr, EntityTrait, IntoActiveModel, TransactionError, TransactionTrait,
};
use uuid::Uuid;

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

impl Session {
    pub(super) async fn create(
        self,
        db: &(impl ConnectionTrait + TransactionTrait),
        first_user: UserId,
    ) -> Result<(), DbErr> {
        let session_user = SessionUser {
            session: self.id,
            user: first_user,
            role: UserRole::Admin,
            added_at: Utc::now(),
            last_access: None,
        };
        let model: entities::session::Model = self.into();
        // Running the save and add a user in a transaction so no session without user is ever created
        db.transaction(|db| {
            Box::pin(async move {
                // Create the session
                entities::prelude::Session::insert(model.into_active_model())
                    .exec(db)
                    .await?;
                // Add this user as the first admin of the session
                session_user.create(db).await?;
                Ok(())
            })
        })
        .await
        .map_err(|err| match err {
            TransactionError::Connection(err) => err,
            TransactionError::Transaction(err) => err,
        })
    }

    pub(super) async fn find_by_id(
        db: &impl ConnectionTrait,
        id: SessionId,
    ) -> Result<Option<Self>, DbErr> {
        entities::prelude::Session::find_by_id(Uuid::from(id))
            .one(db)
            .await?
            .map(|model| {
                model.try_into().map_err(|err| DbErr::TryIntoErr {
                    from: "entities::session::Model",
                    into: "Session",
                    source: Box::new(err),
                })
            })
            .transpose()
    }

    pub(super) async fn fetch_users(
        &self,
        db: &impl ConnectionTrait,
    ) -> Result<impl Iterator<Item = Result<SessionUser, UsersGetNextError>>, UsersGetError> {
        Ok(entities::prelude::Session::find_by_id(Uuid::from(self.id))
            .find_with_related(entities::prelude::SessionUser)
            .all(db)
            .await?
            .pop()
            .expect("All the session calling this method should be present in the database")
            .1
            .into_iter()
            .map(|user| Ok(user.into())))
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

impl SessionUser {
    pub(super) async fn create(self, db: &impl ConnectionTrait) -> Result<(), DbErr> {
        let model: entities::session_user::Model = self.into();
        entities::prelude::SessionUser::insert(model.into_active_model())
            .exec(db)
            .await?;
        Ok(())
    }
    pub(super) async fn find(
        db: &impl ConnectionTrait,
        session: SessionId,
        user: UserId,
    ) -> Result<Option<Self>, DbErr> {
        entities::prelude::SessionUser::find_by_id((*session.as_ref(), *user.as_ref()))
            .one(db)
            .await
            .map(|o| o.map(Into::into))
    }
}
