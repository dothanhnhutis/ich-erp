use axum::{
    Router,
    extract::FromRef,
    routing::{delete, get, patch, post},
};

use crate::{AppState, handlers::user_handler};

pub fn user_route<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
{
    let profile = Router::<S>::new().route("/me", get(user_handler::me));

    let manage = Router::<S>::new()
        .route("/", post(user_handler::create_user))
        .route("/{id}", patch(user_handler::update_user))
        .route("/{id}", delete(user_handler::delete_user))
        .route("/{id}/resend-setup", post(user_handler::resend_setup));

    profile.merge(manage)
}
