use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{Mailbox, header::ContentType},
    transport::smtp::authentication::Credentials,
};
use shared::messaging::{EmailJob, ResetPasswordEmail, SetPasswordEmail};

use crate::config::WorkerConfig;

#[derive(Debug, thiserror::Error)]
pub enum EmailError {
    /// Lỗi vĩnh viễn (địa chỉ sai, dựng message hỏng) — retry vô ích → reject.
    #[error("email không hợp lệ: {0}")]
    Invalid(String),
    /// Lỗi tạm thời (SMTP down/timeout) — nên requeue để thử lại.
    #[error("gửi SMTP thất bại: {0}")]
    Transport(String),
}

pub struct EmailSender {
    mailer: AsyncSmtpTransport<Tokio1Executor>,
    from: Mailbox,
}

impl EmailSender {
    pub fn new(cfg: &WorkerConfig) -> Result<Self, EmailError> {
        let creds = Credentials::new(cfg.smtp_username.clone(), cfg.smtp_password.clone());
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&cfg.smtp_host)
            .map_err(|e| EmailError::Transport(e.to_string()))?
            .port(cfg.smtp_port)
            .credentials(creds)
            .build();

        let from = cfg
            .smtp_from
            .parse()
            .map_err(|_| EmailError::Invalid(format!("SMTP_FROM sai định dạng: {}", cfg.smtp_from)))?;

        Ok(Self { mailer, from })
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

        self.mailer
            .send(email)
            .await
            .map_err(|e| EmailError::Transport(e.to_string()))?;
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
