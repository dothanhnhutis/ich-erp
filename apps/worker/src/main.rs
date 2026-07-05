mod config;
mod consumer;
mod email;
mod gmail_oauth;

use std::time::Duration;

use config::WorkerConfig;
use email::EmailSender;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let cfg = WorkerConfig::from_env().expect("load worker config");
    // SMTP sender dựng 1 lần, tái dùng cho mọi message (transport bên trong tự pool connection).
    let sender = EmailSender::new(&cfg).expect("khởi tạo SMTP sender");

    tracing::info!("email worker khởi động");

    // Vòng ngoài: tự kết nối lại với exponential backoff khi connection RabbitMQ rớt.
    let mut backoff = Duration::from_secs(1);
    const MAX_BACKOFF: Duration = Duration::from_secs(30);

    loop {
        match consumer::run(&cfg, &sender).await {
            Ok(()) => {
                tracing::warn!("consumer dừng (stream đóng), kết nối lại...");
                backoff = Duration::from_secs(1); // vừa chạy được → reset backoff
            }
            Err(e) => {
                tracing::error!(error = %e, "consumer lỗi, thử lại sau {:?}", backoff);
            }
        }

        tokio::time::sleep(backoff).await;
        backoff = (backoff * 2).min(MAX_BACKOFF);
    }
}
