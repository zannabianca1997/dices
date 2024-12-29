//! Pagination instruments

use axum::{
    extract::{FromRequestParts, Query},
    response::IntoResponse,
    Json,
};
use sea_orm::{ConnectionTrait, DbErr, ItemsAndPagesNumber, Paginator, SelectorTrait};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, IntoResponses, ToSchema};

#[derive(
    Deserialize, Debug, Clone, Copy, PartialEq, Eq, FromRequestParts, IntoParams, Serialize,
)]
#[from_request(via(Query))]
#[into_params(parameter_in=Query)]
/// Parameter needed on a paginated endpoint
pub struct PaginationParams {
    /// The requested page
    pub page: u64,
    /// The page size
    #[serde(default = "default_page_size")]
    #[param(default=default_page_size)]
    pub page_size: u64,
}

fn default_page_size() -> u64 {
    15
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, ToSchema, IntoResponses, Deserialize)]
#[response(status=OK)]
/// Paginated result
pub struct PaginatedDto<T: ToSchema> {
    /// Paginated data
    pub data: Box<[T]>,
    /// Page info
    pub page: PageInfo,
}
impl<T: Serialize + ToSchema> IntoResponse for PaginatedDto<T> {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, ToSchema, Deserialize)]
/// Page info
pub struct PageInfo {
    /// Current page number
    pub current: u64,
    /// Total number of pages
    pub number_of_pages: u64,
    /// Total number of items
    pub number_of_items: u64,
    /// Size of each page
    pub size: u64,
    /// Next page, if any
    pub next: Option<u64>,
    /// Previous page, if any
    pub prev: Option<u64>,
}

impl PageInfo {
    pub async fn new<C: ConnectionTrait, S: SelectorTrait>(
        current: u64,
        size: u64,
        paginator: &Paginator<'_, C, S>,
    ) -> Result<Self, DbErr> {
        let ItemsAndPagesNumber {
            number_of_items,
            number_of_pages,
        } = paginator.num_items_and_pages().await?;
        Ok(Self {
            current,
            number_of_items,
            number_of_pages,

            size,
            prev: current.checked_sub(1),
            next: current.checked_add(1).filter(|n| *n < number_of_pages),
        })
    }
}
