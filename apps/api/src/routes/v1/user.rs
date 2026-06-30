use axum::{Router, extract::FromRef, routing::post};

use crate::{AppState, handler::user};

pub fn user_route<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
{
    Router::new().route("/me", post(user::me))
}
