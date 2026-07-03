mod error;
mod extractors;
mod handlers;
mod middlewares;
mod routes;
use crate::routes::create_routes;
use application::services::{
    auth_service::AuthService,
    user_service::{self, UserService},
};
use axum::{Router, extract::FromRef};
use chrono::Duration;
use dotenvy::dotenv;
use infrastructure::{
    cache::{init_redis, session::RedisSessionCache},
    postgres::pool::init_db_pool,
    repositories::{
        pg_password_token_repository::PgPasswordTokenRepository,
        pg_role_repositories::PgRoleRepository, pg_user_repositories::PgUserRepository,
        pg_user_session_repositories::PgUserSessionRepository,
    },
};
use shared::config::AppConfig;
use std::{net::SocketAddr, sync::Arc};

#[derive(Clone, FromRef)]
struct AppState {
    auth_service: Arc<
        AuthService<PgUserRepository, PgUserSessionRepository, PgRoleRepository, RedisSessionCache>,
    >,
    user_service: Arc<UserService<PgUserRepository, PgRoleRepository, PgPasswordTokenRepository>>,
    config: Arc<AppConfig>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::fmt::init();

    let config = AppConfig::from_env().expect("Failed to load config");

    let pool = init_db_pool(&config.database_url)
        .await
        .expect("không kết nối được database");

    let redis_conn = init_redis(&config.redis_url)
        .await
        .expect("Failed to connect to Redis");

    let cache = RedisSessionCache::new(redis_conn);

    let user_repo = PgUserRepository::new(pool.clone());
    let user_session_repo = PgUserSessionRepository::new(pool.clone());
    let role_repo = PgRoleRepository::new(pool.clone());
    let password_token_repo = PgPasswordTokenRepository::new(pool.clone());

    let auth_service = Arc::new(AuthService::new(
        user_repo.clone(),
        user_session_repo.clone(),
        role_repo.clone(),
        cache,
        Duration::seconds(config.session_ttl_secs),
        Duration::seconds(config.session_cache_ttl_secs),
        Duration::seconds(config.session_db_sync_secs),
    ));

    let user_service = Arc::new(UserService::new(
        user_repo.clone(),
        role_repo.clone(),
        password_token_repo.clone(),
        email_publisher.clone(),
        config.app_web_url.clone(),
        config.password_token_ttl_secs,
    ));

    let state = AppState {
        auth_service,
        user_service,
        config: Arc::new(config),
    };

    let app = Router::new()
        .nest("/api", create_routes(state.clone()))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4000").await.unwrap();
    let addr = format!("{}:{}", "0.0.0.0", 4000);
    tracing::info!("Server đang chạy tại: http://{}", addr);
    // axum::serve(listener, app).await.unwrap();

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
