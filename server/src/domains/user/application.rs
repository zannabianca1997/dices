use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use sea_orm::DatabaseConnection;
use utoipa::OpenApi;

use crate::{
    app::{AppState, AuthKey},
    domains::commons::{ErrorCodes, ErrorResponse},
};

use super::domain::{
    models::{LoginRequest, RefreshResponse, RegisterRequest, SignInResponse, User},
    security::{generate_token, AutenticatedUser},
};

#[utoipa::path(
    post,
    path = "/login",
    description = "Login to the server",
    responses(
        (status=StatusCode::OK, description="The login completed with success", body = SignInResponse),
        (status=StatusCode::UNAUTHORIZED, description="The login info are wrong", body = ErrorResponse)
    )
)]
async fn login(
    State(database): State<DatabaseConnection>,
    State(auth_key): State<AuthKey>,
    Json(login): Json<LoginRequest>,
) -> Result<Json<SignInResponse>, ErrorResponse> {
    let (user, auth) = User::login(&database, login).await?;
    Ok(Json(SignInResponse::new(user, auth, auth_key)))
}

#[utoipa::path(
    post,
    path = "/register",
    description = "Register into the server",
    responses(
        (status=StatusCode::CREATED, description="The registration completed with success", body = SignInResponse),
        (status=StatusCode::CONFLICT, description="The username already exist", body=ErrorResponse),
        (status=StatusCode::BAD_REQUEST, description="The login informations are invalid", body=ErrorResponse)
    )
)]
async fn register(
    State(database): State<DatabaseConnection>,
    State(auth_key): State<AuthKey>,
    Json(register): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<SignInResponse>), ErrorResponse> {
    let (user, auth) = User::new(&database, register).await?;
    Ok((
        StatusCode::CREATED,
        Json(SignInResponse::new(user, auth, auth_key)),
    ))
}

#[utoipa::path(
    post,
    path = "/refresh",
    description = "Refresh the access token",
    responses(
        (status=StatusCode::OK, description="The refreshed access token", body = RefreshResponse)
    ),
    security(("UserJWT" = []))
)]
async fn refresh(user: AutenticatedUser, State(auth_key): State<AuthKey>) -> Json<RefreshResponse> {
    Json(RefreshResponse {
        token: generate_token(user, auth_key),
    })
}

#[utoipa::path(
    get,
    path = "",
    description = "Get the info about the user logged in",
    responses(
        (status=StatusCode::OK, description="Data about the user", body = User),
        (status=StatusCode::GONE, description="The user was deleted", body = ErrorResponse)
    ),
    security(("UserJWT" = []))
)]
async fn info(
    State(db): State<DatabaseConnection>,
    user: AutenticatedUser,
) -> Result<Json<User>, ErrorResponse> {
    User::find_by_id(&db, user.id())
        .await
        .map_err(ErrorResponse::internal_server_error)
        .and_then(|user_found| match user_found {
            Some(user) => Ok(Json(user)),
            None => Err(ErrorResponse::builder()
                .code(ErrorCodes::UserDeleted)
                .http_code(StatusCode::GONE) // The user info requested are gone, as the user was deleted
                .msg(format!("The user {} was deleted", user.id()))
                .add("deleted_id", user.id())
                .build()),
        })
}

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(info))
        .route("/login", post(login))
        .route("/register", post(register))
        .route("/refresh", post(refresh))
}

#[derive(OpenApi)]
#[openapi(
    paths(login, register, refresh, info),
    components(schemas(LoginRequest, RegisterRequest, SignInResponse, RefreshResponse, User))
)]
pub(super) struct ApiDocs;
