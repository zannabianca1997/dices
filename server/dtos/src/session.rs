use axum::extract::{
    rejection::{JsonRejection, PathRejection},
    FromRequest, FromRequestParts,
};
use dices_server_entities::session::SessionId;
use serde::Deserialize;

use crate::errors::ErrorResponse;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, FromRequest)]
#[from_request(via(axum::Json), rejection(ErrorResponse<JsonRejection>))]
pub struct SessionCreateDto {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, FromRequest)]
#[from_request(via(axum::Json), rejection(ErrorResponse<JsonRejection>))]
pub struct SessionUpdateDto {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default, with = "::serde_with::rust::double_option")]
    pub description: Option<Option<String>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, FromRequestParts)]
#[from_request(via(axum::extract::Path), rejection(ErrorResponse<PathRejection>))]
pub struct SessionPathData {
    #[serde(rename = "session", alias = "session-id")]
    pub id: SessionId,
}
