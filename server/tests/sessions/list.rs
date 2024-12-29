use dices_server_dtos::{
    paginated::{PageInfo, PaginatedDto, PaginationParams},
    session::SessionShortQueryDto,
};
use futures::FutureExt;
use rand::{seq::SliceRandom, thread_rng};
use test_log::test;

use crate::infrastructure::Infrastructure;

#[test(tokio::test)]
async fn should_get_list() {
    Infrastructure::with(|infrastructure| {
        Box::pin(async move {
            let (_, token) = infrastructure.signup("user", "password").await;
            let ids =
                futures_util::future::join_all((0..5).map(|n| {
                    infrastructure.new_session(&token, format!("session_{n}"), None::<&str>)
                }))
                .await;

            let response = infrastructure
                .server()
                .get(&format!("/sessions"))
                .add_query_params(&PaginationParams {
                    page: 0,
                    page_size: 10,
                })
                .authorization_bearer(token)
                .expect_success()
                .await;

            response.assert_status_ok();

            let PaginatedDto { data, page } = response.json();

            assert_eq!(
                page,
                PageInfo {
                    current: 0,
                    number_of_pages: 1,
                    number_of_items: ids.len() as _,
                    size: 10,
                    next: None,
                    prev: None
                }
            );

            assert_eq!(data.len(), ids.len());

            for SessionShortQueryDto {
                id,
                name,
                last_interaction: _,
            } in data
            {
                let n = ids
                    .iter()
                    .position(|i| *i == id)
                    .expect("A session with a strange id was returned");
                assert_eq!(name, format!("session_{n}"));
            }
        })
    })
    .await;
}

#[test(tokio::test)]
async fn should_not_contain_sessions_that_user_is_not_member_of() {
    Infrastructure::with(|infrastructure| {
        Box::pin(async move {
            let (_, token) = infrastructure.signup("user", "password").await;
            let tokens = futures_util::future::join_all((0..5).map(|n| {
                infrastructure
                    .signup(format!("other_user_{n}"), format!("password_{n}"))
                    .map(|(_, t)| t)
            }))
            .await;
            let mut futures: Vec<_> = tokens
                .iter()
                .map(|t| (t, "other_"))
                .chain(Some((&token, "")))
                .flat_map(|(token, prefix)| {
                    (0..5).map(move |n| {
                        infrastructure.new_session(
                            token,
                            format!("{prefix}session_{n}"),
                            None::<&str>,
                        )
                    })
                })
                .collect();
            // Mixing the futures so the order is not important
            futures.shuffle(&mut thread_rng());
            let ids = futures_util::future::join_all(futures).await;

            let response = infrastructure
                .server()
                .get(&format!("/sessions"))
                .add_query_params(&PaginationParams {
                    page: 0,
                    page_size: 10,
                })
                .authorization_bearer(token)
                .expect_success()
                .await;

            response.assert_status_ok();

            let PaginatedDto { data, page } = response.json();

            assert_eq!(
                page,
                PageInfo {
                    current: 0,
                    number_of_pages: 1,
                    number_of_items: 5,
                    size: 10,
                    next: None,
                    prev: None
                }
            );

            assert_eq!(data.len(), 5);

            for SessionShortQueryDto {
                id,
                name,
                last_interaction: _,
            } in data
            {
                assert!(ids.contains(&id));
                assert!(name.starts_with("session_"));
            }
        })
    })
    .await;
}
