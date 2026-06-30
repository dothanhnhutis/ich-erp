mod v1;
use crate::{AppState, routes::v1::routes_v1};
use axum::{Router, extract::FromRef, http::StatusCode, routing::get};

async fn handle_not_found() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "NOT_FOUND")
}

pub fn create_app<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
{
    let v1_routes: Router<S> = Router::new().nest("/v1", routes_v1());

    Router::new()
        .merge(v1_routes)
        .route("/", get(async || "Hello, World!"))
        .fallback(handle_not_found)

    // Router::new().nest("/api", routes)
}
