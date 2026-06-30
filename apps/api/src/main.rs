mod error;
mod extractors;
mod handler;
mod routes;
use crate::routes::create_app;
use application::services::auth_service::AuthService;
use axum::{Router, extract::FromRef};
use chrono::Duration;
use dotenvy::dotenv;
use infrastructure::{
    postgres::pool::init_db_pool,
    repositories::{
        pg_user_repositories::PgUserRepository,
        pg_user_session_repositories::PgUserSessionRepository,
    },
};
use shared::config::AppConfig;
use std::{net::SocketAddr, sync::Arc};

#[derive(Clone, FromRef)]
struct AppState {
    auth_service: Arc<AuthService<PgUserRepository, PgUserSessionRepository>>,
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

    let user_repo = PgUserRepository::new(pool.clone());
    let user_session_repo = PgUserSessionRepository::new(pool.clone());

    let auth_service = Arc::new(AuthService::new(
        user_repo.clone(),
        user_session_repo.clone(),
        Duration::seconds(config.session_ttl_secs),
    ));
    let state = AppState {
        auth_service,
        config: Arc::new(config),
    };

    let app = Router::new().nest("/api", create_app()).with_state(state);

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
