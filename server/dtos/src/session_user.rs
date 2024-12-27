use axum::extract::{FromRequest, FromRequestParts};
use dices_server_entities::sea_orm_active_enums::UserRole;
use serde::Deserialize;

use crate::{session::SessionPathData, user::UserPathData};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, FromRequest)]
#[from_request(via(axum::Json))]
pub struct SessionUserCreateDto {
    pub role: UserRole,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, FromRequest)]
#[from_request(via(axum::Json))]
pub struct SessionUserUpdateDto {
    #[serde(default)]
    pub role: Option<UserRole>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, FromRequestParts)]
pub struct SessionUserPathData {
    #[serde(flatten)]
    pub session: SessionPathData,
    #[serde(flatten)]
    pub user: UserPathData,
}
