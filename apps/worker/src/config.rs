use std::env;

type ConfigError = Box<dyn std::error::Error + Send + Sync>;

/// Cấu hình riêng của worker — chỉ những gì cần để consume + gửi mail.
/// Cố tình KHÔNG dùng chung `AppConfig` của api (đỡ kéo theo DB/redis/cookie...).
pub struct WorkerConfig {
    /// URL kết nối RabbitMQ.
    pub rabbitmq_url: String,
    /// Tên queue chứa email job (phải khớp với bên api bind vào).
    pub email_queue: String,
    /// Số message chưa-ack tối đa mỗi lúc (QoS prefetch).
    pub prefetch: u16,

    // SMTP (Gmail: smtp.gmail.com, 465 SSL / 587 STARTTLS)
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    /// Địa chỉ gửi, dạng `Tên <a@b.com>` hoặc `a@b.com`.
    pub smtp_from: String,
}

impl WorkerConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            rabbitmq_url: env::var("RABBITMQ_URL")
                .unwrap_or_else(|_| "amqp://guest:guest@127.0.0.1:5672/%2f".into()),
            email_queue: env::var("RABBITMQ_EMAIL_QUEUE").unwrap_or_else(|_| "email_jobs".into()),
            prefetch: env::var("WORKER_PREFETCH")
                .unwrap_or_else(|_| "10".into())
                .parse()
                .map_err(|_| "WORKER_PREFETCH must be an integer")?,

            smtp_host: env::var("SMTP_HOST").map_err(|_| "SMTP_HOST must be set")?,
            smtp_port: env::var("SMTP_PORT")
                .unwrap_or_else(|_| "465".into())
                .parse()
                .map_err(|_| "SMTP_PORT must be a valid port number")?,
            smtp_username: env::var("SMTP_USERNAME").map_err(|_| "SMTP_USERNAME must be set")?,
            smtp_password: env::var("SMTP_PASSWORD").map_err(|_| "SMTP_PASSWORD must be set")?,
            smtp_from: env::var("SMTP_FROM").map_err(|_| "SMTP_FROM must be set")?,
        })
    }
}
