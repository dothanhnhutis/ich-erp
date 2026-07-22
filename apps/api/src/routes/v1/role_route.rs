use axum::{
    Router,
    extract::FromRef,
    routing::{get, post},
};

use crate::{AppState, handlers::role_handler};

pub fn role_routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
{
    Router::new()
        .route("/", post(role_handler::create).get(role_handler::list))
        .route(
            "/{id}",
            get(role_handler::get_by_id)
                .patch(role_handler::update)
                .delete(role_handler::delete),
        )
}
