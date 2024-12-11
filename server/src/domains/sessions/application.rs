use axum::routing::get;
use axum::Router;
use utoipa::OpenApi;

use super::domain::models::{Session, SessionUser};
use crate::app::AppState;
use crate::domains::sessions::domain::models::SessionCreate;

mod sessions;
mod users;

mod commands {
    use axum::{extract::State, Json};
    use dices_ast::intrisics::NoInjectedIntrisics;
    use sea_orm::DatabaseConnection;

    use crate::domains::{
        commons::ErrorResponse, sessions::domain::models::SessionCreate, user::AutenticatedUser,
    };

    #[utoipa::path(
        post,
        path = "/commands",
        description = "Execute a command",
        request_body(
            description = "A serialized `dices` expression"
        ),
        responses(
            (status=StatusCode::OK, description="The command ran successfully", body = dices_ast::Value),
            (status=StatusCode::BAD_REQUEST, description="The command could not be serialized", body = ErrorResponse),
            (status=StatusCode::UNAUTHORIZED, description="A session can be created only by authenticated users", body = ErrorResponse),
            (status=StatusCode::UNAUTHORIZED, description="Sessions can be queried only by authenticated users", body = ErrorResponse),
            (status=StatusCode::NOT_FOUND, description="Session does not exist", body = ErrorResponse)
        ),
        security(("UserJWT" = []))
    )]
    pub(crate) async fn post_new(
        State(database): State<DatabaseConnection>,
        user: AutenticatedUser,
        Json(session_create): Json<dices_ast::Expression<NoInjectedIntrisics>>,
    ) -> Result<Json<dices_ast::Value>, ErrorResponse> {
        todo!()
    }
}

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(sessions::get_all).post(sessions::post_new))
        .route(
            "/:session-uuid",
            get(sessions::get).delete(sessions::delete),
        )
        .route("/:session-uuid/users", get(users::get_all))
}

#[derive(OpenApi)]
#[openapi(
    paths(
        sessions::get_all,
        sessions::post_new,
        sessions::get,
        sessions::delete,
        users::get_all
    ),
    components(schemas(Session, SessionCreate, SessionUser))
)]
pub(super) struct ApiDocs;
