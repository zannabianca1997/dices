use axum::Router;
use utoipa::OpenApi;

use crate::AppState;

pub(super) fn router() -> Router<AppState> {
    Router::new()
}

#[derive(OpenApi)]
pub(super) struct ApiDocs;
