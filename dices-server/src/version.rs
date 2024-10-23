use crate::Domain;
use axum::{routing::get, Json, Router};
use dices_version::Version;
use utoipa::OpenApi;

#[utoipa::path(
    get,
    path = "/", path="/server",
    description = "Get the version of the server",
    responses(
        (status=200, description="The version of the server", body = Version)
    )
)]
async fn version_server() -> Json<Version> {
    Json(crate::VERSION)
}
#[utoipa::path(
    get,
    path="/ast",
    description = "Get the version of the ast used in the server",
    responses(
        (status=200, description="The version of the ast used", body = Version)
    )
)]
async fn version_ast() -> Json<Version> {
    Json(dices_ast::VERSION)
}
#[utoipa::path(
    get,
    path = "/engine",
    description = "Get the version of the engine used in the server",
    responses(
        (status=200, description="The version of the engine used", body = Version)
    )
)]
async fn version_engine() -> Json<Version> {
    Json(dices_engine::VERSION)
}

pub fn router<S: Clone + Send + Sync + 'static>() -> Router<S> {
    return Router::new()
        .route("/", get(version_server))
        .route("/server", get(version_server))
        .route("/ast", get(version_ast))
        .route("/engine", get(version_engine));
}

#[derive(OpenApi)]
#[openapi(
    paths(version_server, version_engine, version_ast),
    components(schemas(Version))
)]
pub struct ApiDocs;

pub const DOMAIN: Domain = Domain {
    name: "version",
    version: 1,
    api: router,
    api_docs: ApiDocs::openapi,
};