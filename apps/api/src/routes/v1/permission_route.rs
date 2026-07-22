use axum::Router;
use axum::extract::FromRef;
use axum::routing::get;

use crate::AppState;
use crate::handlers::permission_handler;

pub fn permission_route<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
{
    Router::new().route("/", get(permission_handler::list))
}
