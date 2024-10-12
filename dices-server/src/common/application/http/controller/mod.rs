use axum::Router;

mod version;

pub fn router<S: Clone + Send + Sync + 'static>() -> Router<S> {
    return Router::new().nest("/version", version::router());
}
