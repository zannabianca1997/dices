//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.2

use sea_orm::{entity::prelude::*, TryFromU64};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize)]
#[sea_orm(schema_name = "public", table_name = "user")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: UserId,
    #[sea_orm(column_type = "Text", unique)]
    pub name: String,
    #[sea_orm(column_type = "Text")]
    pub password: String,
    pub created_at: DateTimeWithTimeZone,
    pub last_access: DateTimeWithTimeZone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, DeriveValueType, Serialize, Deserialize)]
pub struct UserId(Uuid);

impl TryFromU64 for UserId {
    fn try_from_u64(n: u64) -> Result<Self, DbErr> {
        Uuid::try_from_u64(n).map(Self)
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::session_user::Entity")]
    SessionUser,
}

impl Related<super::session_user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SessionUser.def()
    }
}

impl Related<super::session::Entity> for Entity {
    fn to() -> RelationDef {
        super::session_user::Relation::Session.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::session_user::Relation::User.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
