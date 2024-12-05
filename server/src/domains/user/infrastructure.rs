use chrono::Utc;
use sea_orm::{
    ActiveModelBehavior, ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, DbErr,
    EntityTrait, IntoActiveModel, PaginatorTrait, QueryFilter, Set,
};
use uuid::Uuid;

use crate::entities;

use super::domain::{
    models::{User, UserId},
    security::PasswordHash,
};

impl From<entities::user::Model> for User {
    fn from(
        entities::user::Model {
            id,
            name,
            password,
            created_at,
            last_access,
        }: entities::user::Model,
    ) -> Self {
        User {
            id: UserId::from(id),
            username: name,
            password: PasswordHash::from(password),
            created_at: created_at.to_utc(),
            last_access: last_access.to_utc(),
        }
    }
}
impl From<User> for entities::user::Model {
    fn from(
        User {
            id,
            username: name,
            password,
            created_at,
            last_access,
        }: User,
    ) -> Self {
        Self {
            id: id.into(),
            name,
            password: password.into(),
            created_at: created_at.fixed_offset(),
            last_access: last_access.fixed_offset(),
        }
    }
}

impl User {
    pub(super) async fn save(self, db: &impl ConnectionTrait) -> Result<(), DbErr> {
        let model: entities::user::Model = self.into();
        entities::prelude::User::insert(model.into_active_model())
            .exec(db)
            .await?;
        Ok(())
    }

    pub(super) async fn find_by_name(
        db: &impl ConnectionTrait,
        name: &str,
    ) -> Result<Option<Self>, DbErr> {
        Ok(entities::prelude::User::find()
            .filter(entities::user::Column::Name.eq(name))
            .one(db)
            .await?
            .map(Into::into))
    }
    pub(super) async fn exist_by_name(
        db: &impl ConnectionTrait,
        name: &str,
    ) -> Result<bool, DbErr> {
        Ok(entities::prelude::User::find()
            .filter(entities::user::Column::Name.eq(name))
            .count(db)
            .await?
            != 0)
    }
    pub(super) async fn update_last_access(
        &self,
        db: &impl ConnectionTrait,
    ) -> Result<Self, DbErr> {
        let mut user = entities::user::ActiveModel::new();
        user.id = ActiveValue::unchanged(self.id.into());
        user.last_access = Set(Utc::now().fixed_offset());
        user.update(db).await.map(Into::into)
    }
    pub(super) async fn find_by_id(
        db: &impl ConnectionTrait,
        id: UserId,
    ) -> Result<Option<Self>, DbErr> {
        Ok(entities::prelude::User::find_by_id(Uuid::from(id))
            .one(db)
            .await?
            .map(Self::from))
    }
}
