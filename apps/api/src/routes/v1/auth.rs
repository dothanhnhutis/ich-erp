use axum::{Router, extract::FromRef, routing::post};

use crate::{AppState, handler::auth};

pub fn public_routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
{
    Router::new().route("/login", post(auth::login_handler))
}
