use axum::{Router, routing::post};

use crate::handler::auth;

pub fn auth_route<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new().route("/login", post(auth::login_handler))
}
