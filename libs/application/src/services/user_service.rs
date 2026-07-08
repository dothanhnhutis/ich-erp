use crate::{
    dto::user_dto::{CreateUserRequest, CreateUserResponse, UpdateUserRequest, UserResponse},
    errors::AppError,
    ports::EmailPublisher,
    security::session_token::SessionToken,
};
use chrono::{Duration, Utc};
use domain::{
    entities::{
        password_token::{NewPasswordToken, PasswordTokenType},
        user::{NewUser, UserStatus, UserUpdate},
    },
    repositories::{PasswordTokenRepository, RoleRepository, UserRepository},
};
use shared::messaging::{EmailJob, SetPasswordEmail};
use std::str::FromStr;

// Parse chuỗi trạng thái → UserStatus, lỗi → Validation.
fn parse_status(s: &str) -> Result<UserStatus, AppError> {
    UserStatus::from_str(s).map_err(|_| AppError::Validation("Trạng thái không hợp lệ".into()))
}

// Parse trạng thái cho cập nhật — chỉ cho ACTIVE/DEACTIVATED (không cho PENDING_PASSWORD).
fn parse_updatable_status(s: &str) -> Result<UserStatus, AppError> {
    match parse_status(s)? {
        UserStatus::PendingPassword => Err(AppError::Validation(
            "Không thể đặt trạng thái PENDING_PASSWORD".into(),
        )),
        st => Ok(st),
    }
}

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
    pub fn new(
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

    // Sinh token INIT (raw vào link, hash lưu DB) + publish mail thiết lập tài khoản.
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

    // Cập nhật username/status của user. status chỉ nhận ACTIVE/DEACTIVATED.
    pub async fn update_user(
        &self,
        id: uuid::Uuid,
        req: UpdateUserRequest,
    ) -> Result<UserResponse, AppError> {
        let status = match req.status.as_deref() {
            Some(s) => Some(parse_updatable_status(s)?),
            None => None,
        };

        let changes = UserUpdate {
            username: req.username.map(|u| u.trim().to_string()),
            status,
        };

        let updated = self
            .user_repo
            .update(id, changes)
            .await?
            .ok_or_else(|| AppError::NotFound("Người dùng không tồn tại".into()))?;
        Ok(UserResponse::from(updated))
    }

    // Xoá mềm user (404 nếu không tồn tại / đã xoá). Thu hồi phiên do handler điều phối.
    pub async fn delete_user(&self, id: uuid::Uuid) -> Result<(), AppError> {
        self.user_repo.soft_delete(id).await?;
        Ok(())
    }

    // Admin gửi lại mail thiết lập cho user CHƯA kích hoạt: vô hiệu token INIT cũ
    // + cấp token mới (24h) + gửi lại mail.
    pub async fn resend_setup(&self, user_id: uuid::Uuid) -> Result<(), AppError> {
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Người dùng không tồn tại".into()))?;
        if user.status != UserStatus::PendingPassword {
            return Err(AppError::Validation(
                "Tài khoản đã kích hoạt, không thể gửi lại liên kết".into(),
            ));
        }

        // Vô hiệu token INIT cũ còn hiệu lực → chỉ link mới nhất dùng được.
        self.password_token_repo
            .invalidate_active(user_id, PasswordTokenType::Init)
            .await?;
        self.send_setup_email(user_id, &user.email).await?;
        Ok(())
    }
}
