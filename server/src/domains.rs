use axum::Router;
use utoipa::openapi::OpenApi;

use crate::app::AppState;

pub use commons::ErrorCodes;

mod commons;
mod sessions;
mod user;
mod version;

/// Static description of a domain
pub(super) struct Domain {
    pub(super) name: &'static str,
    pub(super) version: u16,
    pub(super) api: fn() -> Router<AppState>,
    pub(super) api_docs: fn() -> OpenApi,
}
pub(super) const DOMAINS: &[Domain] = &[version::DOMAIN, sessions::DOMAIN, user::DOMAIN];
