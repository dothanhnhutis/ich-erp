use argon2::{self, PasswordVerifier};
use domain::{entities::user::UserStatus, repositories::UserRepository};

use crate::{
    dto::auth_dto::{LoginRequest, LoginResponse},
    error::AppError,
};

pub struct AuthService<UR>
where
    UR: UserRepository,
{
    user_repo: UR,
}

impl<UR> AuthService<UR>
where
    UR: UserRepository,
{
    pub fn new(user_repo: UR) -> Self {
        Self { user_repo }
    }

    pub async fn login(&self, request: LoginRequest) -> Result<LoginResponse, AppError> {
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

        // 5. Return response — trả token THÔ cho client
        Ok(LoginResponse {
            user_id: user.id.to_string(),
            session: String::new(),
            expires_in: 10000,
        })
    }
}
