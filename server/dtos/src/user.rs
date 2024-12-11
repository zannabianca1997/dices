use dices_server_entities::user::UserId;
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct UserCreateDto {
    pub name: String,
    pub password: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct UserUpdateDto {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub struct UserPathData {
    #[serde(rename = "user", alias = "user-id")]
    pub id: UserId,
}
