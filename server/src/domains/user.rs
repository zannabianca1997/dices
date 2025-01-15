//! # `/user`: Current user
//!
//! Management of the user logged in: general info, change of username and password,
//! deleting of the user.

use axum::{
    debug_handler,
    extract::{FromRef, State},
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::NotSet, DatabaseConnection, DeleteResult, EntityTrait, Set,
};
use utoipa::{Modify, OpenApi};
use utoipa_axum::{router::OpenApiRouter, routes};

use dices_server_auth::{hash_password, Autenticated, AuthKey, RequireUserToken};
use dices_server_dtos::user::{
    UserGetError, UserQueryDto, UserSigninDto, UserSignupDto, UserUpdateDto,
};
use dices_server_entities::user::{self, UserId};

#[utoipa::path(get, path = "/", responses(UserQueryDto, UserGetError))]
#[debug_handler(state = crate::app::App)]
/// Info about the current user
///
/// Get the basic info about the current user, like username and time of last access.
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
        Err(err) => return Err(err.into()),
    };
    Ok(UserQueryDto(user))
}

#[utoipa::path(
    delete, path = "/",
    responses(
        (status= OK, description="User successfully deleted"), UserGetError
    )
)]
#[debug_handler(state = crate::app::App)]
/// Delete the current user
///
/// Remove the current user from the server, removing them from all the sessions they are part of.
async fn user_delete(
    State(db): State<DatabaseConnection>,
    id: Autenticated<UserId>,
) -> Result<(), UserGetError> {
    // TODO: remove sessions where this user was the last one
    match dices_server_entities::prelude::User::delete_by_id(id.into_inner())
        .exec(&db)
        .await
    {
        Ok(DeleteResult { rows_affected: 1 }) => Ok(()),
        Ok(DeleteResult { rows_affected: 0 }) => Err(UserGetError::Deleted),
        Ok(DeleteResult { rows_affected: _ }) => unreachable!(),
        Err(err) => Err(err.into()),
    }
}

#[utoipa::path(
    put, path = "/",
    request_body=UserSignupDto,
    responses(UserQueryDto, UserGetError)
)]
#[debug_handler(state = crate::app::App)]
/// Edit the current user
///
/// Change the info about the current user.
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
        sea_orm::DbErr::RecordNotUpdated => err.into(),
        _ => UserGetError::Deleted,
    })?;

    Ok(UserQueryDto(user))
}

#[utoipa::path(
    patch, path = "/",
    request_body=UserUpdateDto,
    responses(UserQueryDto, UserGetError),
)]
#[debug_handler(state = crate::app::App)]
/// Partial edit the current user
///
/// Change some info about the current user. Info that are either not given or `null` are left unchanged.
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
        sea_orm::DbErr::RecordNotUpdated => err.into(),
        _ => UserGetError::Deleted,
    })?;

    Ok(UserQueryDto(user))
}

pub(super) fn router<S: Clone + Send + Sync + 'static>() -> OpenApiRouter<S>
where
    DatabaseConnection: FromRef<S>,
    AuthKey: FromRef<S>,
{
    let mut router = OpenApiRouter::with_openapi(dices_server_dtos::user::ApiComponents::openapi())
        .routes(routes!(user_get, user_put, user_patch, user_delete));

    RequireUserToken.modify(router.get_openapi_mut());
    super::tag_api(router.get_openapi_mut(), "User");

    router
}
