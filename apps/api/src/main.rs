mod error;
mod extractors;
mod handler;
mod routes;
use crate::routes::create_app;
use dotenvy::dotenv;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    dotenv().ok();
    // tracing_subscriber::fmt::init();

    let filter_layer = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")); // Mặc định là info nếu RUST_LOG trống

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(true) // Bật màu sắc trên terminal
                .with_level(true) // Hiển thị mức log (INFO, DEBUG)
                .with_file(true) // Hiển thị tên file tạo ra log
                .with_line_number(true), // Hiển thị số dòng code tạo ra log
        )
        .init();

    let app = create_app();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4000").await.unwrap();
    let addr = format!("{}:{}", "0.0.0.0", 4000);
    tracing::info!("Server đang chạy tại: http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}
