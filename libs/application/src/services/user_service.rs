use chrono::{Duration, Utc};
use domain::{
    entities::{
        password_token::{NewPasswordToken, PasswordTokenType},
        user::NewUser,
    },
    repositories::{PasswordTokenRepository, RoleRepository, UserRepository},
};
use shared::messaging::{EmailJob, SetPasswordEmail};

use crate::{
    dto::user_dto::{CreateUserRequest, CreateUserResponse},
    errors::AppError,
    ports::EmailPublisher,
    security::session_token::SessionToken,
};

pub struct UserService<UR, RR, PTR, EP>
where
    UR: UserRepository,
    RR: RoleRepository,
    PTR: PasswordTokenRepository,
    EP: EmailPublisher,
{
    user_repo: UR,
    role_repo: RR,
    password_token_repo: PTR,
    email_publisher: EP,
    app_web_url: String,
    token_ttl_secs: i64,
}

impl<UR, RR, PTR, EP> UserService<UR, RR, PTR, EP>
where
    UR: UserRepository,
    RR: RoleRepository,
    PTR: PasswordTokenRepository,
    EP: EmailPublisher,
{
    fn new(
        user_repo: UR,
        role_repo: RR,
        password_token_repo: PTR,
        email_publisher: EP,
        app_web_url: String,
        token_ttl_secs: i64,
    ) -> Self {
        Self {
            user_repo,
            role_repo,
            password_token_repo,
            email_publisher,
            app_web_url,
            token_ttl_secs,
        }
    }

    /// Sinh token INIT (raw vào link, hash lưu DB) + publish mail thiết lập tài khoản.
    async fn send_setup_email(&self, user_id: uuid::Uuid, email: &str) -> Result<(), AppError> {
        let token = SessionToken::generate();
        let expires_at = Utc::now() + Duration::seconds(self.token_ttl_secs);
        self.password_token_repo
            .create(NewPasswordToken {
                user_id,
                token_hash: token.hash,
                token_type: PasswordTokenType::Init,
                expires_at,
            })
            .await?;

        let url = format!("{}/setup-account?token={}", self.app_web_url, token.raw);
        self.email_publisher
            .publish(EmailJob::SetPassword(SetPasswordEmail {
                to: email.to_string(),
                set_password_url: url,
                expires_in_hours: self.token_ttl_secs / 3600,
            }))
            .await?;
        Ok(())
    }

    pub async fn create_user(
        &self,
        req: CreateUserRequest,
    ) -> Result<CreateUserResponse, AppError> {
        if req.role_ids.is_empty() {
            return Err(AppError::Validation("Phải chọn ít nhất một vai trò".into()));
        }
        // 1. Tạo user + gán role (transaction trong repo).
        let user = self
            .user_repo
            .create_with_roles(NewUser { email: req.email }, &req.role_ids)
            .await?;

        // 2. Sinh token INIT + đẩy email "thiết lập tài khoản" vào hàng chờ.
        self.send_setup_email(user.id, &user.email).await?;

        Ok(CreateUserResponse {
            user_id: user.id.to_string(),
        })
    }
}
