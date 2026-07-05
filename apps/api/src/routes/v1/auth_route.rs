use axum::{Router, extract::FromRef, routing::post};

use crate::{AppState, handlers::auth_handler};

pub fn public_routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
{
    Router::new()
        .route("/login", post(auth_handler::login_handler))
        .route("/setup-account", post(auth_handler::setup_account))
}

pub fn private_routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
{
    Router::new().route("/logout", post(auth_handler::logout))
}
