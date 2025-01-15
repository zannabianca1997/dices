//! # `/auth`: Authentication
//!
//! Endpoints to sign in and sign up in the server.

use axum::{
    debug_handler,
    extract::{FromRef, State},
};
use sea_orm::{
    ActiveModelTrait,
    ActiveValue::{self},
    ColumnTrait as _, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter as _,
};

use dices_server_auth::{
    check_password, hash_password, new_token, AuthKey, CheckPasswordError, SecurityAddon,
};
use dices_server_dtos::user::{
    SigninError, SignupError, UserSigninDto, UserSigninResponseDto, UserSignupDto,
    UserSignupResponseDto,
};
use dices_server_entities::user::{self, UserId};
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};

#[utoipa::path(
    post, path = "/signup",
    request_body=UserSignupDto,
    responses(UserSignupResponseDto, SignupError)
)]
#[debug_handler(state = crate::app::App)]
/// Create a new user
///
/// Provide the server with the authentication info, and get an access token in return (along with user info).
async fn signup(
    State(auth_key): State<AuthKey>,
    State(db): State<DatabaseConnection>,
    UserSignupDto(UserSigninDto { name, password }): UserSignupDto,
) -> Result<UserSignupResponseDto, SignupError> {
    // Checks
    let name = {
        let name = name.trim();
        if name.chars().any(char::is_whitespace) {
            return Err(SignupError::WhitespacesInUsername);
        }
        name
    };

    let id = UserId::gen();

    let (auth_id, password) = hash_password(id, &password);

    let user = user::ActiveModel {
        id: ActiveValue::Set(id),
        name: ActiveValue::Set(name.to_owned()),
        password: ActiveValue::Set(password),
        ..Default::default()
    };

    let user = match user.insert(&db).await {
        Ok(inserted) => inserted,
        Err(err) => {
            // check if it's a collision
            if dices_server_entities::prelude::User::find()
                .filter(dices_server_entities::user::Column::Name.eq(name))
                .count(&db)
                .await?
                == 1
            {
                return Err(SignupError::UserAlreadyExist);
            }
            return Err(err.into());
        }
    };

    let token = new_token(auth_id, &auth_key);

    Ok(UserSignupResponseDto(UserSigninResponseDto { token, user }))
}

#[utoipa::path(
    post, path = "/signin",
    request_body=UserSigninDto,
    responses(UserSigninResponseDto, SigninError)
)]
#[debug_handler(state = crate::app::App)]
/// Signin into the server
///
/// Provide the server with the authentication info, and get an access token in return (along with user info).
async fn signin(
    State(auth_key): State<AuthKey>,
    State(db): State<DatabaseConnection>,
    UserSigninDto { name, password }: UserSigninDto,
) -> Result<UserSigninResponseDto, SigninError> {
    let user = match dices_server_entities::prelude::User::find()
        .filter(dices_server_entities::user::Column::Name.eq(name))
        .one(&db)
        .await
    {
        Ok(Some(found)) => found,
        Ok(None) => return Err(SigninError::UserDoNotExist),
        Err(err) => return Err(err.into()),
    };

    let auth_id = check_password(user.id, &user.password, &password).map_err(|err| match err {
        CheckPasswordError::Password => SigninError::WrongPassword,
        err => err.into(),
    })?;

    let token = new_token(auth_id, &auth_key);

    Ok(UserSigninResponseDto { token, user })
}

#[derive(OpenApi)]
#[openapi(modifiers(&SecurityAddon))]
struct ApiInfo;

pub(super) fn router<S: Clone + Send + Sync + 'static>() -> OpenApiRouter<S>
where
    DatabaseConnection: FromRef<S>,
    AuthKey: FromRef<S>,
{
    let mut router = OpenApiRouter::with_openapi(ApiInfo::openapi())
        .routes(routes!(signup))
        .routes(routes!(signin));
    super::tag_api(router.get_openapi_mut(), "Auth");
    router
}
