use argon2::{self, PasswordVerifier};
use chrono::Duration;
use domain::{
    entities::{session::NewSession, user::UserStatus},
    repositories::{UserRepository, UserSessionRepository},
};

use crate::{
    dto::auth_dto::{ClientContext, LoginRequest, LoginResponse},
    errors::AppError,
    security::session_token::SessionToken,
};

pub struct AuthService<UR, USR>
where
    UR: UserRepository,
    USR: UserSessionRepository,
{
    user_repo: UR,
    user_session_repo: USR,
    session_ttl: Duration,
}

impl<UR, USR> AuthService<UR, USR>
where
    UR: UserRepository,
    USR: UserSessionRepository,
{
    pub fn new(user_repo: UR, user_session_repo: USR, session_ttl: Duration) -> Self {
        Self {
            user_repo,
            user_session_repo,
            session_ttl,
        }
    }

    pub async fn login(
        &self,
        request: LoginRequest,
        ctx: ClientContext,
    ) -> Result<LoginResponse, AppError> {
        let user = self
            .user_repo
            .find_by_email(&request.email)
            .await?
            .ok_or_else(|| AppError::Unauthorized("Email hoặc mật khẩu không đúng".into()))?;

        match user.status {
            UserStatus::Active => {}
            UserStatus::PendingPassword => {
                return Err(AppError::Unauthorized(
                    "Tài khoản chưa đặt mật khẩu. Vui lòng kiểm tra email.".into(),
                ));
            }
            UserStatus::Deactivated => {
                return Err(AppError::Unauthorized("Tài khoản đã bị vô hiệu hóa".into()));
            }
        }

        let hash_str = user.password_hash.clone().ok_or_else(|| {
            AppError::Unauthorized("Tài khoản chưa đặt mật khẩu. Vui lòng kiểm tra email.".into())
        })?;

        let parsed = argon2::PasswordHash::new(&hash_str)
            .map_err(|e| AppError::Internal(format!("Invalid password hash: {}", e)))?;

        argon2::Argon2::default()
            .verify_password(request.password.as_bytes(), &parsed)
            .map_err(|_| AppError::Unauthorized("Email hoặc mật khẩu không đúng".into()))?;

        let token = SessionToken::generate();
        let expires_at = chrono::Utc::now() + self.session_ttl;

        let new_session = NewSession {
            user_id: user.id,
            token_hash: token.hash,
            device_type: request.device_type,
            device_id: request.device_id,
            device_name: request.device_name,
            platform: request.platform,
            app_version: request.app_version,
            user_agent: ctx.user_agent,
            ip_address: ctx.ip_address,
            expires_at,
        };

        self.user_session_repo.create(new_session).await?;

        // 5. Return response — trả token THÔ cho client
        Ok(LoginResponse {
            user_id: user.id.to_string(),
            session: token.raw,
            expires_in: self.session_ttl.num_seconds(),
        })
    }
}
