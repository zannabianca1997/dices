use axum::{
    body::Body,
    debug_handler,
    extract::State,
    routing::{get, post},
    Router,
};
use sea_orm::{
    ActiveModelTrait,
    ActiveValue::{self, NotSet},
    ColumnTrait as _, DatabaseConnection, DeleteResult, EntityTrait, QueryFilter as _, Set,
};

use dices_server_auth::{
    check_password, hash_password, new_token, Autenticated, AuthKey, CheckPasswordError,
};
use dices_server_dtos::user::{
    token::UserToken, SigninError, SignupError, UserGetError, UserLoginResponseDto, UserQueryDto,
    UserSigninDto, UserSignupDto, UserSignupResponseDto, UserUpdateDto,
};
use dices_server_entities::user::{self, UserId};

use crate::app::App;

#[debug_handler(state = crate::app::App)]
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
        name.to_owned()
    };

    let id = UserId::new();

    let (auth_id, password) = hash_password(id, &password);

    let user = user::ActiveModel {
        id: ActiveValue::Set(id),
        name: ActiveValue::Set(name),
        password: ActiveValue::Set(password),
        ..Default::default()
    };

    let user = match user.insert(&db).await {
        Ok(inserted) => inserted,
        Err(err) => match err {
            dices_server_migration::DbErr::RecordNotInserted => {
                return Err(SignupError::UserAlreadyExist)
            }
            err => return Err(SignupError::DbErr(err)),
        },
    };

    let token = new_token(auth_id, auth_key);

    Ok(UserSignupResponseDto(UserLoginResponseDto { token, user }))
}

#[debug_handler(state = crate::app::App)]
async fn signin(
    State(auth_key): State<AuthKey>,
    State(db): State<DatabaseConnection>,
    UserSigninDto { name, password }: UserSigninDto,
) -> Result<UserSignupResponseDto, SigninError> {
    let user = match dices_server_entities::prelude::User::find()
        .filter(dices_server_entities::user::Column::Name.eq(name))
        .one(&db)
        .await
    {
        Ok(Some(found)) => found,
        Ok(None) => return Err(SigninError::UserDoNotExist),
        Err(err) => return Err(SigninError::DbErr(err)),
    };

    let auth_id = check_password(user.id, &user.password, password).map_err(|err| match err {
        CheckPasswordError::Password => SigninError::WrongPassword,
        err => SigninError::CheckPasswordError(err),
    })?;

    let token = new_token(auth_id, auth_key);

    Ok(UserSignupResponseDto(UserLoginResponseDto { token, user }))
}

#[debug_handler(state = crate::app::App)]
async fn user_get(
    State(db): State<DatabaseConnection>,
    id: Autenticated<UserId>,
) -> Result<UserQueryDto, UserGetError> {
    let user = match dices_server_entities::prelude::User::find_by_id(id.into_inner())
        .one(&db)
        .await
    {
        Ok(Some(found)) => found,
        Ok(None) => return Err(UserGetError::Deleted),
        Err(err) => return Err(UserGetError::DbErr(err)),
    };
    Ok(UserQueryDto(user))
}

#[debug_handler(state = crate::app::App)]
async fn user_delete(
    State(db): State<DatabaseConnection>,
    id: Autenticated<UserId>,
) -> Result<(), UserGetError> {
    match dices_server_entities::prelude::User::delete_by_id(id.into_inner())
        .exec(&db)
        .await
    {
        Ok(DeleteResult { rows_affected: 1 }) => Ok(()),
        Ok(DeleteResult { rows_affected: 0 }) => Err(UserGetError::Deleted),
        Ok(DeleteResult { rows_affected: _ }) => unreachable!(),
        Err(err) => Err(UserGetError::DbErr(err)),
    }
}

#[debug_handler(state = crate::app::App)]
async fn user_put(
    State(db): State<DatabaseConnection>,
    id: Autenticated<UserId>,
    UserSignupDto(UserSigninDto { name, password }): UserSignupDto,
) -> Result<UserQueryDto, UserGetError> {
    let (_, password) = hash_password(*id.inner(), &password);

    let user = user::ActiveModel {
        id: Set(id.into_inner()),
        name: Set(name),
        password: Set(password),
        created_at: NotSet,
        last_access: NotSet,
    }
    .update(&db)
    .await
    .map_err(|err| match err {
        sea_orm::DbErr::RecordNotUpdated => UserGetError::DbErr(err),
        _ => UserGetError::Deleted,
    })?;

    Ok(UserQueryDto(user))
}

#[debug_handler(state = crate::app::App)]
async fn user_patch(
    State(db): State<DatabaseConnection>,
    id: Autenticated<UserId>,
    UserUpdateDto { name, password }: UserUpdateDto,
) -> Result<UserQueryDto, UserGetError> {
    let password = password.map(|password| hash_password(*id.inner(), &password).1);

    let user = user::ActiveModel {
        id: Set(id.into_inner()),
        name: name.map(Set).unwrap_or_default(),
        password: password.map(Set).unwrap_or_default(),
        created_at: NotSet,
        last_access: NotSet,
    }
    .update(&db)
    .await
    .map_err(|err| match err {
        sea_orm::DbErr::RecordNotUpdated => UserGetError::DbErr(err),
        _ => UserGetError::Deleted,
    })?;

    Ok(UserQueryDto(user))
}

pub(super) fn router() -> Router<App> {
    Router::new()
        .route(
            "/",
            get(user_get)
                .put(user_put)
                .patch(user_patch)
                .delete(user_delete),
        )
        .route("/signup", post(signup))
        .route("/signin", post(signin))
}
