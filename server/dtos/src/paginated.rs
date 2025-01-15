//! Pagination instruments

use std::num::NonZeroU64;

use axum::{
    extract::{FromRequestParts, Query},
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, FixedOffset};
use sea_orm::{ConnectionTrait, DbErr, ItemsAndPagesNumber, Paginator, SelectorTrait};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, IntoResponses, ToSchema};

#[derive(
    Deserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    FromRequestParts,
    IntoParams,
    Serialize,
    ToSchema,
)]
#[from_request(via(Query))]
#[into_params(parameter_in=Query)]
/// Parameters for an endpoint that paginated by time
pub struct TimePaginationParams {
    /// Start of the requested page
    pub start: DateTime<FixedOffset>,
    /// End of the requested page
    pub end: DateTime<FixedOffset>,
    /// Limit to the number of returned results
    ///
    /// If the number of values goes over the limit, the actual limit
    /// are returned in the `page` element of the returned DTO.
    #[serde(default = "default_page_size")]
    #[param(default = default_page_size, value_type=u64, minimum=1)]
    #[schema(default = default_page_size, value_type=u64, minimum=1)]
    pub limit: NonZeroU64,
    /// The side of the range to return when the range contains more than the limit
    #[serde(default = "LimitAlign::default")]
    #[param(default = LimitAlign::default)]
    pub limit_align: LimitAlign,
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema, Serialize, Default)]
#[serde(rename_all = "snake_case")]
/// The side of the range to return when the range contains more than the limit
pub enum LimitAlign {
    /// Return the items at the start of the range
    Start,
    /// Return the items at the end of the range
    #[default]
    End,
}

#[derive(
    Deserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    FromRequestParts,
    IntoParams,
    Serialize,
    ToSchema,
)]
#[from_request(via(Query))]
#[into_params(parameter_in=Query)]
/// Parameter needed on a paginated endpoint
pub struct FixedSizePaginationParams {
    /// The requested page
    pub page: u64,
    /// The page size
    #[serde(default = "default_page_size")]
    #[param(default = default_page_size, value_type=u64, minimum=1)]
    #[schema(default = default_page_size, value_type=u64, minimum=1)]
    pub page_size: NonZeroU64,
}

fn default_page_size() -> NonZeroU64 {
    NonZeroU64::new(50).unwrap()
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, ToSchema, IntoResponses, Deserialize)]
#[response(status=OK)]
/// Paginated result
pub struct PaginatedDto<T: ToSchema, PageInfo: ToSchema> {
    /// Paginated data
    pub data: Box<[T]>,
    /// Page info
    pub page: PageInfo,
}
impl<T: Serialize + ToSchema, PageInfo: Serialize + ToSchema> IntoResponse
    for PaginatedDto<T, PageInfo>
{
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, ToSchema, Deserialize)]
/// Page info
pub struct FixedSizePageInfo {
    /// Current page number
    pub page: u64,
    /// Total number of pages
    pub number_of_pages: u64,
    /// Total number of items
    pub number_of_items: u64,
    /// Size of each page
    pub page_size: u64,
    /// Next page, if any
    pub next: Option<FixedSizePaginationParams>,
    /// Previous page, if any
    pub prev: Option<FixedSizePaginationParams>,
}

impl FixedSizePageInfo {
    pub async fn new<C: ConnectionTrait, S: SelectorTrait>(
        FixedSizePaginationParams { page, page_size }: FixedSizePaginationParams,
        paginator: &Paginator<'_, C, S>,
    ) -> Result<Self, DbErr> {
        let ItemsAndPagesNumber {
            number_of_items,
            number_of_pages,
        } = paginator.num_items_and_pages().await?;
        Ok(Self {
            page,
            number_of_items,
            number_of_pages,

            page_size: page_size.get(),
            prev: page
                .checked_sub(1)
                .map(|page| FixedSizePaginationParams { page, page_size }),
            next: page
                .checked_add(1)
                .filter(|n| *n < number_of_pages)
                .map(|page| FixedSizePaginationParams { page, page_size }),
        })
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, ToSchema, Deserialize)]
/// Page info
pub struct TimePageInfo {
    /// Start of the requested page
    pub start: DateTime<FixedOffset>,
    /// End of the requested page
    pub end: DateTime<FixedOffset>,

    /// Total number of items
    pub number_of_items: u64,
    /// Size of the page
    pub page_size: u64,

    /// Next page, if any
    pub next: Option<TimePaginationParams>,
    /// Previous page, if any
    pub prev: Option<TimePaginationParams>,
}
