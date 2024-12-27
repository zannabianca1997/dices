use axum::extract::{FromRequest, FromRequestParts};
use dices_server_entities::session::SessionId;
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, FromRequest)]
#[from_request(via(axum::Json))]
pub struct SessionCreateDto {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, FromRequest)]
#[from_request(via(axum::Json))]
pub struct SessionUpdateDto {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default, with = "::serde_with::rust::double_option")]
    pub description: Option<Option<String>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, FromRequestParts)]
#[from_request(via(axum::extract::Path))]
pub struct SessionPathData {
    #[serde(rename = "session", alias = "session-id")]
    pub id: SessionId,
}
