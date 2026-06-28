use axum::{Router, extract::FromRef, routing::post};
use sqlx::PgPool;

use crate::handler::auth;

pub fn auth_route<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    PgPool: FromRef<S>,
{
    Router::new().route("/login", post(auth::login_handler))
}
