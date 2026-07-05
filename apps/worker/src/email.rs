use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{Mailbox, header::ContentType},
    transport::smtp::authentication::{Credentials, Mechanism},
};
use shared::messaging::{EmailJob, ResetPasswordEmail, SetPasswordEmail};

use crate::config::{EmailBackendConfig, WorkerConfig};
use crate::gmail_oauth::GmailOAuth;

#[derive(Debug, thiserror::Error)]
pub enum EmailError {
    /// Lỗi vĩnh viễn (địa chỉ sai, dựng message hỏng) — retry vô ích → reject.
    #[error("email không hợp lệ: {0}")]
    Invalid(String),
    /// Lỗi tạm thời (SMTP down/timeout, lấy token lỗi) — nên requeue để thử lại.
    #[error("gửi SMTP thất bại: {0}")]
    Transport(String),
}

/// Backend gửi mail. SMTP tĩnh dựng transport 1 lần; Gmail phải dựng lại mỗi lần
/// gửi vì access token OAuth2 đổi theo thời gian.
enum Backend {
    Smtp(AsyncSmtpTransport<Tokio1Executor>),
    Gmail {
        host: String,
        port: u16,
        sender: String,
        oauth: GmailOAuth,
    },
}

pub struct EmailSender {
    backend: Backend,
    from: Mailbox,
}

impl EmailSender {
    pub fn new(cfg: &WorkerConfig) -> Result<Self, EmailError> {
        match &cfg.email {
            EmailBackendConfig::Smtp {
                host,
                port,
                username,
                password,
                from,
            } => {
                let creds = Credentials::new(username.clone(), password.clone());
                let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(host)
                    .map_err(|e| EmailError::Transport(e.to_string()))?
                    .port(*port)
                    .credentials(creds)
                    .build();

                let from = from
                    .parse()
                    .map_err(|_| EmailError::Invalid(format!("SMTP_FROM sai định dạng: {from}")))?;

                Ok(Self {
                    backend: Backend::Smtp(mailer),
                    from,
                })
            }
            EmailBackendConfig::GmailOAuth2 {
                host,
                port,
                client_id,
                client_secret,
                refresh_token,
                sender,
            } => {
                let from = sender.parse().map_err(|_| {
                    EmailError::Invalid(format!("GMAIL_SENDER sai định dạng: {sender}"))
                })?;
                let oauth = GmailOAuth::new(
                    client_id.clone(),
                    client_secret.clone(),
                    refresh_token.clone(),
                );

                Ok(Self {
                    backend: Backend::Gmail {
                        host: host.clone(),
                        port: *port,
                        sender: sender.clone(),
                        oauth,
                    },
                    from,
                })
            }
        }
    }

    pub async fn send(&self, job: EmailJob) -> Result<(), EmailError> {
        let (to, subject, body) = render(&job);

        let email = Message::builder()
            .from(self.from.clone())
            .to(to.parse().map_err(|_| EmailError::Invalid(to.clone()))?)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(body)
            .map_err(|e| EmailError::Invalid(e.to_string()))?;

        match &self.backend {
            Backend::Smtp(mailer) => {
                mailer
                    .send(email)
                    .await
                    .map_err(|e| EmailError::Transport(e.to_string()))?;
            }
            Backend::Gmail {
                host,
                port,
                sender,
                oauth,
            } => {
                // Access token hết hạn ~1h → lấy token còn hạn rồi dựng transport
                // XOAUTH2 cho lần gửi này (lettre tự tạo chuỗi `user=..\x01auth=Bearer ..`).
                let token = oauth.access_token().await?;
                let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(host)
                    .map_err(|e| EmailError::Transport(e.to_string()))?
                    .port(*port)
                    .authentication(vec![Mechanism::Xoauth2])
                    .credentials(Credentials::new(sender.clone(), token))
                    .build();

                mailer
                    .send(email)
                    .await
                    .map_err(|e| EmailError::Transport(e.to_string()))?;
            }
        }
        Ok(())
    }
}

/// (người nhận, tiêu đề, body HTML)
fn render(job: &EmailJob) -> (String, String, String) {
    match job {
        EmailJob::SetPassword(SetPasswordEmail {
            to,
            set_password_url,
            expires_in_hours,
        }) => (
            to.clone(),
            "Thiết lập tài khoản của bạn".to_owned(),
            format!(
                "<p>Nhấn vào liên kết dưới đây để thiết lập tài khoản \
                 (hết hạn sau {expires_in_hours} giờ):</p>\
                 <p><a href=\"{set_password_url}\">{set_password_url}</a></p>"
            ),
        ),
        EmailJob::ResetPassword(ResetPasswordEmail {
            to,
            reset_password_url,
            expires_in_hours,
        }) => (
            to.clone(),
            "Đặt lại mật khẩu".to_owned(),
            format!(
                "<p>Nhấn vào liên kết dưới đây để đặt lại mật khẩu \
                 (hết hạn sau {expires_in_hours} giờ):</p>\
                 <p><a href=\"{reset_password_url}\">{reset_password_url}</a></p>"
            ),
        ),
    }
}
