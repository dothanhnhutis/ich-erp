pub mod auth;

use axum::Router;

use crate::routes::v1::auth::auth_route;

pub fn routes_v1<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new().nest("/auth", auth_route())
}
