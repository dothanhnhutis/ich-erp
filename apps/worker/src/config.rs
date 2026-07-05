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
    /// Backend gửi mail — chọn bằng env `EMAIL_PROVIDER`.
    pub email: EmailBackendConfig,
}

/// Cách xác thực khi gửi mail. `EMAIL_PROVIDER=smtp` (mặc định) dùng user/password;
/// `EMAIL_PROVIDER=gmail_oauth2` dùng OAuth2 của Google (SMTP XOAUTH2).
pub enum EmailBackendConfig {
    /// SMTP thường (username/password hoặc App Password).
    Smtp {
        host: String,
        port: u16,
        username: String,
        password: String,
        /// Địa chỉ gửi, dạng `Tên <a@b.com>` hoặc `a@b.com`.
        from: String,
    },
    /// Gmail qua OAuth2 — access token được refresh động từ refresh token.
    GmailOAuth2 {
        host: String,
        port: u16,
        client_id: String,
        client_secret: String,
        refresh_token: String,
        /// Địa chỉ Gmail: vừa là SMTP user (XOAUTH2) vừa là From.
        sender: String,
    },
}

impl WorkerConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let provider = env::var("EMAIL_PROVIDER").unwrap_or_else(|_| "smtp".into());
        let email = match provider.as_str() {
            "smtp" => EmailBackendConfig::Smtp {
                host: env::var("SMTP_HOST").map_err(|_| "SMTP_HOST must be set")?,
                port: env::var("SMTP_PORT")
                    .unwrap_or_else(|_| "465".into())
                    .parse()
                    .map_err(|_| "SMTP_PORT must be a valid port number")?,
                username: env::var("SMTP_USERNAME").map_err(|_| "SMTP_USERNAME must be set")?,
                password: env::var("SMTP_PASSWORD").map_err(|_| "SMTP_PASSWORD must be set")?,
                from: env::var("SMTP_FROM").map_err(|_| "SMTP_FROM must be set")?,
            },
            "gmail_oauth2" => EmailBackendConfig::GmailOAuth2 {
                host: env::var("GMAIL_SMTP_HOST").unwrap_or_else(|_| "smtp.gmail.com".into()),
                port: env::var("GMAIL_SMTP_PORT")
                    .unwrap_or_else(|_| "465".into())
                    .parse()
                    .map_err(|_| "GMAIL_SMTP_PORT must be a valid port number")?,
                client_id: env::var("GMAIL_CLIENT_ID").map_err(|_| "GMAIL_CLIENT_ID must be set")?,
                client_secret: env::var("GMAIL_CLIENT_SECRET")
                    .map_err(|_| "GMAIL_CLIENT_SECRET must be set")?,
                refresh_token: env::var("GMAIL_REFRESH_TOKEN")
                    .map_err(|_| "GMAIL_REFRESH_TOKEN must be set")?,
                sender: env::var("GMAIL_SENDER").map_err(|_| "GMAIL_SENDER must be set")?,
            },
            other => {
                return Err(format!(
                    "EMAIL_PROVIDER không hợp lệ: {other} (dùng 'smtp' hoặc 'gmail_oauth2')"
                )
                .into());
            }
        };

        Ok(Self {
            rabbitmq_url: env::var("RABBITMQ_URL")
                .unwrap_or_else(|_| "amqp://guest:guest@127.0.0.1:5672/%2f".into()),
            email_queue: env::var("RABBITMQ_EMAIL_QUEUE").unwrap_or_else(|_| "email_jobs".into()),
            prefetch: env::var("WORKER_PREFETCH")
                .unwrap_or_else(|_| "10".into())
                .parse()
                .map_err(|_| "WORKER_PREFETCH must be an integer")?,
            email,
        })
    }
}
