use domain::{
    entities::password_token::{PasswordToken, PasswordTokenType},
    repositories::{PasswordTokenRepository, UserRepository},
};

use crate::{
    dto::auth_dto::SetupAccountRequest,
    errors::AppError,
    ports::EmailPublisher,
    security::{password::hash_password, session_token::hash_token},
};

pub struct AccountService<UR, PTR, EP>
where
    UR: UserRepository,
    PTR: PasswordTokenRepository,
    EP: EmailPublisher,
{
    user_repo: UR,
    token_repo: PTR,
    email_publisher: EP,
    app_web_url: String,
    reset_token_ttl_secs: i64,
}

impl<UR, PTR, EP> AccountService<UR, PTR, EP>
where
    UR: UserRepository,
    PTR: PasswordTokenRepository,
    EP: EmailPublisher,
{
    pub fn new(
        user_repo: UR,
        token_repo: PTR,
        email_publisher: EP,
        app_web_url: String,
        reset_token_ttl_secs: i64,
    ) -> Self {
        Self {
            user_repo,
            token_repo,
            email_publisher,
            app_web_url,
            reset_token_ttl_secs,
        }
    }
    pub async fn setup_account(&self, req: SetupAccountRequest) -> Result<(), AppError> {
        let tok = self
            .consume_token(&req.token, PasswordTokenType::Init)
            .await?;
        let password_hash = hash_password(&req.password)?;
        self.user_repo
            .activate_account(tok.user_id, req.username.trim(), &password_hash, tok.id)
            .await?;
        Ok(())
    }

    // Tìm token còn hiệu lực theo raw token + kiểm đúng loại; lỗi → Validation.
    async fn consume_token(
        &self,
        raw_token: &str,
        expected: PasswordTokenType,
    ) -> Result<PasswordToken, AppError> {
        let token_hash = hash_token(raw_token);
        let tok = self
            .token_repo
            .find_active_by_hash(&token_hash)
            .await?
            .ok_or_else(|| AppError::Validation("Liên kết không hợp lệ hoặc đã hết hạn".into()))?;
        if tok.token_type != expected {
            return Err(AppError::Validation("Liên kết không hợp lệ".into()));
        }
        Ok(tok)
    }
}
