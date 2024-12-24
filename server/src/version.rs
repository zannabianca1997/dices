use axum::{debug_handler, routing::get, Json, Router};
use dices_version::Version;

#[debug_handler]
async fn server() -> Json<Version> {
    Json(crate::VERSION)
}
#[debug_handler]
async fn engine() -> Json<Version> {
    Json(dices_engine::VERSION)
}
#[debug_handler]
async fn ast() -> Json<Version> {
    Json(dices_ast::VERSION)
}

pub fn router<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route("/", get(server))
        .route("/server", get(server))
        .route("/engine", get(engine))
        .route("/ast", get(ast))
}
