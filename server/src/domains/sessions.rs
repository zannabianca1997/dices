//! # `/sessions`: Available sessions
//!
//! Sessions available for the current user

use axum::{
    debug_handler,
    extract::{FromRef, State},
};
use dices_server_migration::SimpleExpr;
use sea_orm::{
    sea_query, ActiveModelTrait, ActiveValue::NotSet, ColumnTrait as _, DatabaseConnection,
    EntityTrait, IntoSimpleExpr, PaginatorTrait, QueryFilter, QueryOrder, Set, TransactionTrait,
};
use utoipa::{Modify, OpenApi as _};
use utoipa_axum::{router::OpenApiRouter, routes};

use dices_server_auth::{Autenticated, AuthKey, RequireUserToken};
use dices_server_dtos::{
    paginated::{FixedSizePageInfo, FixedSizePaginationParams, PaginatedDto},
    session::{
        SessionCreateDto, SessionCreateError, SessionCreateResponseDto, SessionListGetError,
        SessionQueryDto, SessionShortQueryDto,
    },
};
use dices_server_entities::{
    prelude::*,
    sea_orm_active_enums::UserRole,
    session::{self, SessionId},
    session_user,
    user::UserId,
};

mod single;

#[utoipa::path(post, path = "/", request_body=SessionCreateDto, responses(SessionCreateResponseDto, SessionCreateError))]
#[debug_handler(state = crate::app::App)]
/// Create a new session
///
/// This will create a new session, with the current user as its first member
/// and admin.
async fn sessions_post(
    State(db): State<DatabaseConnection>,
    user_id: Autenticated<UserId>,
    SessionCreateDto { name, description }: SessionCreateDto,
) -> Result<SessionCreateResponseDto, SessionCreateError> {
    let name = name.trim().to_owned();
    if name.contains('\n') {
        return Err(SessionCreateError::NameContainsNewline);
    }

    let session_id = SessionId::gen();
    let session = session::ActiveModel {
        id: Set(session_id),
        name: Set(name),
        description: Set(description),
        created_at: NotSet,
    };
    let session_user = session_user::ActiveModel {
        session: Set(session_id),
        user: Set(*user_id.inner()),
        role: Set(UserRole::Admin),
        added_at: NotSet,
        last_access: NotSet,
    };

    db.transaction(|db| {
        Box::pin(async move {
            let session = session.insert(db).await?;
            let session_user = session_user.insert(db).await?.into();
            Ok(SessionQueryDto {
                session,
                session_user,
            })
        })
    })
    .await
    .map(SessionCreateResponseDto)
    .map_err(|err| match err {
        sea_orm::TransactionError::Connection(db_err) => db_err.into(),
        sea_orm::TransactionError::Transaction(err) => err,
    })
}

#[utoipa::path(get, path = "/", responses(PaginatedDto<SessionShortQueryDto, FixedSizePageInfo>, SessionListGetError), params(FixedSizePaginationParams))]
#[debug_handler(state = crate::app::App)]
/// Get a list of available sessions
///
/// Get the sessions the current user is part of.
/// The sessions are paginated and ordered by date of last interaction.
async fn sessions_get(
    State(db): State<DatabaseConnection>,
    user_id: Autenticated<UserId>,
    FixedSizePaginationParams { page, page_size }: FixedSizePaginationParams,
) -> Result<PaginatedDto<SessionShortQueryDto, FixedSizePageInfo>, SessionListGetError> {
    // Create the paginated query

    // TODO: this should be a view query, instead of mapping after
    let paginated_query = Session::find()
        .find_also_related(SessionUser)
        .filter(session_user::Column::User.eq(*user_id.inner()))
        .order_by_desc(SimpleExpr::FunctionCall(sea_query::Func::coalesce(
            [
                session_user::Column::LastAccess,
                session_user::Column::AddedAt,
            ]
            .map(IntoSimpleExpr::into_simple_expr),
        )))
        .paginate(&db, page_size.get());

    // Query the database

    let fetch_pageinfo = FixedSizePageInfo::new(
        FixedSizePaginationParams { page, page_size },
        &paginated_query,
    );
    let fetch_page = paginated_query.fetch_page(page);

    let (page, page_info) = tokio::try_join!(fetch_page, fetch_pageinfo)?;

    // Format the data and remove unuseful information

    let data = page
        .into_iter()
        .map(|(session, session_user)| {
            SessionShortQueryDto::from(SessionQueryDto {
                session,
                session_user: session_user
                    .expect("The query should return only sessions with related session_users")
                    .into(),
            })
        })
        .collect();
    Ok(PaginatedDto {
        data,
        page: page_info,
    })
}

pub fn router<S: Clone + Send + Sync + 'static>() -> OpenApiRouter<S>
where
    DatabaseConnection: FromRef<S>,
    AuthKey: FromRef<S>,
{
    let mut router =
        OpenApiRouter::with_openapi(dices_server_dtos::session::ApiComponents::openapi())
            .routes(routes!(sessions_post, sessions_get))
            .nest("/{session}", single::router());
    RequireUserToken.modify(router.get_openapi_mut());
    super::tag_api(router.get_openapi_mut(), "Sessions");
    router
}
