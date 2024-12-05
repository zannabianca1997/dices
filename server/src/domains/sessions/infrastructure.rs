use bincode::error::DecodeError;
use sea_orm::{ConnectionTrait, DbErr, EntityTrait, IntoActiveModel};
use uuid::Uuid;

use crate::entities;

use super::domain::models::{Session, SessionId};

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
    pub(super) async fn save(self, db: &impl ConnectionTrait) -> Result<(), DbErr> {
        let model: entities::session::Model = self.into();
        entities::prelude::Session::insert(model.into_active_model())
            .exec(db)
            .await?;
        Ok(())
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
}
