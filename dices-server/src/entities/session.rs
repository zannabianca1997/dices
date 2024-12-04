//! `SeaORM` Entity, @generated by sea-orm-codegen 1.0.0-rc.5

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "session")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(column_type = "Text")]
    pub name: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub description: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    #[sea_orm(column_type = "VarBinary(StringLen::None)", nullable)]
    pub image: Option<Vec<u8>>,
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

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        super::session_user::Relation::User.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::session_user::Relation::Session.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
