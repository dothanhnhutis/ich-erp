mod auth;
mod user;
use crate::{
    AppState,
    routes::v1::{auth::public_routes, user::user_route},
};
use axum::{Router, extract::FromRef};

pub fn routes_v1<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
{
    let public = Router::new().nest("/auth", public_routes());

    let protected = Router::new().nest("/users", user_route());

    public.merge(protected)
}
