mod v1;
use crate::routes::v1::routes_v1;
use axum::{Router, routing::get};

pub fn create_app<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    let v1_routes: Router<S> = Router::new().nest("/v1", routes_v1());

    let routes: Router<S> = Router::new()
        .merge(v1_routes)
        .route("/", get(async || "Hello, World!"));

    Router::new().nest("/api", routes)
}
