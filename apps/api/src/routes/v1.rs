mod auth_route;
mod permission_route;
mod role_route;
mod user_route;
use crate::{AppState, middlewares::auth_middleware::require_auth};
use axum::{Router, extract::FromRef, middleware::from_fn_with_state};

pub fn routes_v1<S>(state: AppState) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
{
    let public = Router::new().nest("/auth", auth_route::public_routes());

    let protected = Router::new()
        .nest("/users", user_route::user_route())
        .nest("/auth", auth_route::private_routes())
        .nest("/roles", role_route::role_routes())
        .nest("/permissions", permission_route::permission_route())
        .route_layer(from_fn_with_state(state, require_auth));

    public.merge(protected)
}
