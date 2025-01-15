//! `SeaORM` Entity, @generated by sea-orm-codegen 1.0.0-rc.5

use std::cmp::Ordering;

use chrono::{DateTime, FixedOffset};
use content::LogContent;
use sea_orm::{entity::prelude::*, sea_query::Nullable, TryFromU64};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{session::SessionId, user::UserId};

pub mod content;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, ToSchema)]
#[sea_orm(table_name = "log")]
#[schema(as=Log)]
/// Log item
pub struct Model {
    /// Consecutive unique id of the log item
    #[sea_orm(primary_key)]
    pub id: LogId,
    /// Id of the session this log refer to
    pub session_id: SessionId,
    /// User that issued the command that resulted in this log
    pub user_id: Option<UserId>,
    /// Id of the command that resulted in this log
    pub answer_to: Option<LogId>,
    /// Date of log creation
    pub created_at: DateTime<FixedOffset>,
    /// Content of the log
    pub content: LogContent,
}

impl Model {
    /// Order of the logs in the chat
    ///
    /// First they are ordered by creation date, then by id to ensure consistency
    #[must_use]
    pub fn log_order(&self, other: &Self) -> Ordering {
        self.created_at
            .cmp(&other.created_at)
            .then(self.id.cmp(&other.id))
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    DeriveValueType,
    Serialize,
    Deserialize,
    utoipa::ToSchema,
    PartialOrd,
    Ord,
)]
#[repr(transparent)]
/// Id of a log message
pub struct LogId(Uuid);

impl TryFromU64 for LogId {
    fn try_from_u64(n: u64) -> Result<Self, DbErr> {
        Uuid::try_from_u64(n).map(Self)
    }
}

impl Nullable for LogId {
    fn null() -> Value {
        Uuid::null()
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "Entity",
        from = "Column::AnswerTo",
        to = "Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    SelfRef,
    #[sea_orm(
        belongs_to = "super::session::Entity",
        from = "Column::SessionId",
        to = "super::session::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Session,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id",
        on_update = "Cascade",
        on_delete = "SetNull"
    )]
    User,
}

impl Related<super::session::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Session.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
