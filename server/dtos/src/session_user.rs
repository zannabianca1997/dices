use dices_server_entities::sea_orm_active_enums::UserRole;
use serde::Deserialize;

use crate::{session::SessionPathData, user::UserPathData};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub struct SessionUserCreateDto {
    pub role: UserRole,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub struct SessionUserUpdateDto {
    #[serde(default)]
    pub role: Option<UserRole>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub struct SessionUserPathData {
    #[serde(flatten)]
    pub session: SessionPathData,
    #[serde(flatten)]
    pub user: UserPathData,
}
