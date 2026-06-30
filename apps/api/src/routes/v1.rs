mod auth;
mod user;
use crate::{
    AppState,
    middleware::auth::require_auth,
    routes::v1::{auth::public_routes, user::user_route},
};
use axum::{Router, extract::FromRef, middleware::from_fn_with_state};

pub fn routes_v1<S>(state: AppState) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
{
    let public = Router::new().nest("/auth", public_routes());

    let protected = Router::new()
        .nest("/users", user_route())
        .route_layer(from_fn_with_state(state, require_auth));

    public.merge(protected)
}
