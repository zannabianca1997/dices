//! # `/version`: Versioning
//!
//! Version info about the server, engine and AST.

use axum::{debug_handler, Json};

use dices_version::Version;
use utoipa_axum::{router::OpenApiRouter, routes};

#[utoipa::path(
    get, path = "/server", 
    responses(
        (status= OK, body = Version, description="The version of the server")
    )
)]
#[debug_handler]
/// Version of the server
///
/// The version of the server package.
async fn server() -> Json<Version> {
    Json(crate::VERSION)
}

#[utoipa::path(
    get, path = "/engine",
    responses(
        (status= OK, body = Version, description="The version of the engine")
    )
)]
#[debug_handler]
/// Version of the engine
///
/// The version of the engine package used by the server.
async fn engine() -> Json<Version> {
    Json(dices_engine::VERSION)
}

#[utoipa::path(
    get, path = "/ast", 
    responses(
        (status= OK, body = Version, description="The version of the ast")
    )
)]
#[debug_handler]
/// Version of the ast
///
/// The version of the AST package used by the server.
/// This is the main version that must match to talk with the api.
async fn ast() -> Json<Version> {
    Json(dices_ast::VERSION)
}

pub fn router<S: Clone + Send + Sync + 'static>() -> OpenApiRouter<S> {
    let mut router = OpenApiRouter::default()
        .routes(routes!(server))
        .routes(routes!(ast))
        .routes(routes!(engine));
    super::tag_api(router.get_openapi_mut(), "Version");
    router
}
