use axum::{routing::get, Json, Router};

use crate::common::application::http::dtos::VersionDto;

async fn version_server() -> Json<VersionDto> {
    Json(VersionDto::new(crate::VERSION))
}
async fn version_ast() -> Json<VersionDto> {
    Json(VersionDto::new(dices_ast::version::VERSION))
}
async fn version_engine() -> Json<VersionDto> {
    Json(VersionDto::new(dices_engine::VERSION))
}

pub fn router<S: Clone + Send + Sync + 'static>() -> Router<S> {
    return Router::new()
        .route("/", get(version_server))
        .route("/server", get(version_server))
        .route("/ast", get(version_ast))
        .route("/engine", get(version_engine));
}
