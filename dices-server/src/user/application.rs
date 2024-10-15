use axum::{
    routing::{get, post},
    Json, Router,
};
use utoipa::OpenApi;

use crate::AppState;

use super::domain::models::{
    LoginRequest, RefreshRequest, RefreshResponse, RegisterRequest, SignInResponse, User,
};

#[utoipa::path(
    post,
    path = "/login",
    description = "Login to the server",
    responses(
        (status=200, description="The login completed with success", body = SignInResponse)
    )
)]
async fn login(login: Json<LoginRequest>) -> Json<SignInResponse> {
    todo!()
}

#[utoipa::path(
    post,
    path = "/register",
    description = "Register into the server",
    responses(
        (status=200, description="The registration completed with success", body = SignInResponse)
    )
)]
async fn register(register: Json<RegisterRequest>) -> Json<SignInResponse> {
    todo!()
}

#[utoipa::path(
    post,
    path = "/refresh",
    description = "Refresh the access token",
    responses(
        (status=200, description="The refreshed access token", body = RefreshResponse)
    )
)]
async fn refresh(refresh: Json<RefreshRequest>) -> Json<RefreshResponse> {
    todo!()
}

#[utoipa::path(
    get,
    path = "/",
    description = "Get the info about the user logged in",
    responses(
        (status=200, description="Data about the user", body = User)
    )
)]
async fn info() -> Json<User> {
    todo!()
}

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(info))
        .route("/login", post(login))
        .route("/register", post(register))
        .route("/refresh", post(register))
}

#[derive(OpenApi)]
#[openapi(
    paths(login, register, refresh, info),
    components(schemas(
        LoginRequest,
        RegisterRequest,
        SignInResponse,
        RefreshRequest,
        RefreshResponse,
        User
    ))
)]
pub(super) struct ApiDocs;
