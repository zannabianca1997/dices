//! Endpoints to access the session history

use std::thread::panicking;

use axum::{
    debug_handler,
    extract::{FromRef, State},
};
use chrono::{FixedOffset, Local};
use dices_server_dtos::{
    engine::LogsGetError,
    paginated::{
        FixedSizePaginationParams, LimitAlign, PaginatedDto, TimePageInfo, TimePaginationParams,
    },
    session::SessionPathData,
};
use dices_server_entities::{log, user::UserId};
use sea_orm::{
    ColumnTrait as _, DatabaseConnection, EntityTrait as _, PaginatorTrait as _, QueryFilter as _,
    QueryOrder as _,
};
use utoipa_axum::{router::OpenApiRouter, routes};

use dices_server_auth::{Autenticated, AuthKey};

use super::fetch_session_data;

#[utoipa::path(get, path = "/", responses(PaginatedDto<log::Model,TimePageInfo>, LogsGetError), params(TimePaginationParams,SessionPathData))]
#[debug_handler(state = crate::app::App)]
/// Get the logs of the session
///
/// Get the log history of this session.
/// The logs are paginated and ordered by date. Each one may contains a link to the
async fn logs_get(
    State(db): State<DatabaseConnection>,
    user_id: Autenticated<UserId>,
    SessionPathData { id: session_id }: SessionPathData,
    TimePaginationParams {
        start,
        end,
        limit,
        limit_align,
    }: TimePaginationParams,
) -> Result<PaginatedDto<log::Model, TimePageInfo>, LogsGetError> {
    let (session, _) = fetch_session_data(&db, user_id, session_id).await?;

    if start > end {
        return Err(LogsGetError::StartAfterEnd);
    }

    // Create the paginated query
    let query = log::Entity::find().filter(log::Column::SessionId.eq(session_id));

    let mut cursor_query = query.clone().cursor_by(log::Column::CreatedAt);
    cursor_query.after(start).before(end);
    match limit_align {
        LimitAlign::Start => cursor_query.first(limit.get()),
        LimitAlign::End => cursor_query.last(limit.get()),
    };

    // Query the database

    let fetch_page = cursor_query.all(&db);
    let fetch_number_of_items = query.count(&db);

    let (page, number_of_items) = tokio::try_join!(fetch_page, fetch_number_of_items)?;

    debug_assert!(
        page.is_sorted_by_key(|p| p.created_at),
        "The query should return sorted results"
    );

    // Lenght of the requested page
    let requested_duration = end - start;
    // Start and end of the data actually fetched
    let start = match limit_align {
        LimitAlign::Start => start,
        LimitAlign::End => {
            if (page.len() as u64) < limit.get() {
                start
            } else {
                page.first().map(|f| f.created_at).unwrap_or(start)
            }
        }
    };
    let end = match limit_align {
        LimitAlign::Start => {
            if (page.len() as u64) < limit.get() {
                end
            } else {
                page.last().map(|f| f.created_at).unwrap_or(end)
            }
        }
        LimitAlign::End => end,
    };

    let page_info = TimePageInfo {
        start,
        end,
        number_of_items,
        page_size: page.len() as _,
        next: (end < Local::now()).then(|| TimePaginationParams {
            start: end,
            end: end + requested_duration,
            limit,
            limit_align: LimitAlign::Start,
        }),
        prev: (start > session.created_at).then(|| TimePaginationParams {
            start: Ord::max(start - requested_duration, session.created_at),
            end: start,
            limit,
            limit_align: LimitAlign::End,
        }),
    };

    Ok(PaginatedDto {
        data: page.into_boxed_slice(),
        page: page_info,
    })
}

pub fn router<S: Clone + Send + Sync + 'static>() -> OpenApiRouter<S>
where
    DatabaseConnection: FromRef<S>,
    AuthKey: FromRef<S>,
{
    OpenApiRouter::default().routes(routes!(logs_get))
}
