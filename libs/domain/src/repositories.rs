use chrono::{DateTime, Utc};

use crate::{
    entities::{
        password_token::{NewPasswordToken, PasswordToken, PasswordTokenType},
        permission::Permission,
        role::{Role, RoleStatus},
        session::{NewSession, Session},
        user::{NewUser, User, UserUpdate},
    },
    errors::DomainError,
};

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("không tìm thấy")]
    NotFound,

    #[error("vi phạm unique: {0}")]
    UniqueViolation(String), // mang tên constraint, để application tự dịch message

    #[error("vi phạm khóa ngoại: {0}")]
    ForeignKeyViolation(String),

    #[error("dữ liệu không hợp lệ: {0}")]
    Mapping(#[from] DomainError), // row -> entity fail (vd InvalidUserStatus)

    #[error(transparent)]
    Unexpected(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
}

pub trait UserRepository: Send + Sync {
    /// Trả về 1 trang user + tổng số rows (tính theo search filter).
    /// `search` ILIKE trên email + username (case-insensitive substring).
    fn find_paginated(
        &self,
        offset: u32,
        limit: u32,
        search: Option<&str>,
    ) -> impl std::future::Future<Output = Result<(Vec<User>, u64), RepositoryError>> + Send;

    fn find_by_email(
        &self,
        email: &str,
    ) -> impl Future<Output = Result<Option<User>, RepositoryError>> + Send;

    fn find_by_id(
        &self,
        id: uuid::Uuid,
    ) -> impl Future<Output = Result<Option<User>, RepositoryError>> + Send;

    fn create_with_roles(
        &self,
        new_user: NewUser,
        role_ids: &[uuid::Uuid],
    ) -> impl Future<Output = Result<User, RepositoryError>> + Send;

    fn activate_account(
        &self,
        user_id: uuid::Uuid,
        username: &str,
        password_hash: &str,
        token_id: uuid::Uuid,
    ) -> impl Future<Output = Result<(), RepositoryError>> + Send;

    // RESET: đặt mật khẩu mới (+ password_changed_at) + đánh dấu token đã dùng, atomic.
    // Không đụng status/username (user đã ACTIVE).
    fn reset_password(
        &self,
        user_id: uuid::Uuid,
        password_hash: &str,
        token_id: uuid::Uuid,
    ) -> impl Future<Output = Result<(), RepositoryError>> + Send;

    fn update(
        &self,
        id: uuid::Uuid,
        changes: UserUpdate,
    ) -> impl Future<Output = Result<Option<User>, RepositoryError>> + Send;

    // Xoá mềm (set deleted_at). NotFound nếu user không tồn tại / đã xoá.
    fn soft_delete(
        &self,
        id: uuid::Uuid,
    ) -> impl Future<Output = Result<(), RepositoryError>> + Send;
}

pub trait UserSessionRepository: Send + Sync {
    fn create(
        &self,
        new_session: NewSession,
    ) -> impl Future<Output = Result<Session, RepositoryError>> + Send;

    fn find_by_token_hash(
        &self,
        token_hash: &str,
    ) -> impl Future<Output = Result<Option<Session>, RepositoryError>> + Send;

    fn touch_expires(
        &self,
        id: uuid::Uuid,
        expires_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<(), RepositoryError>> + Send;

    fn revoke(
        &self,
        id: uuid::Uuid,
        reason: &str,
    ) -> impl Future<Output = Result<(), RepositoryError>> + Send;

    // Thu hồi tất cả phiên còn hiệu lực của một user (logout mọi thiết bị).
    fn revoke_all_for_user(
        &self,
        user_id: uuid::Uuid,
        reason: &str,
    ) -> impl Future<Output = Result<(), RepositoryError>> + Send;
}

pub trait PermissionRepository: Send + Sync {
    fn find_all(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<Permission>, RepositoryError>> + Send;
}
pub trait RoleRepository: Send + Sync {
    /// Mã permission của một user (JOIN user_roles→role_permissions→permissions),
    /// chỉ tính role đang ACTIVE và chưa xoá mềm.
    fn find_permission_codes_for_user(
        &self,
        user_id: uuid::Uuid,
    ) -> impl Future<Output = Result<Vec<String>, RepositoryError>> + Send;

    fn find_all(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<Role>, RepositoryError>> + Send;

    /// Trả về 1 trang role + tổng số rows (tính theo search filter).
    /// `search` ILIKE trên name + description.
    fn find_paginated(
        &self,
        offset: u32,
        limit: u32,
        search: Option<&str>,
    ) -> impl std::future::Future<Output = Result<(Vec<Role>, u64), RepositoryError>> + Send;

    fn find_by_id(
        &self,
        id: uuid::Uuid,
    ) -> impl std::future::Future<Output = Result<Option<Role>, RepositoryError>> + Send;

    fn find_permissions_for_role(
        &self,
        role_id: uuid::Uuid,
    ) -> impl std::future::Future<Output = Result<Vec<Permission>, RepositoryError>> + Send;

    fn create(
        &self,
        name: &str,
        description: &str,
        permission_ids: &[uuid::Uuid],
    ) -> impl std::future::Future<Output = Result<Role, RepositoryError>> + Send;

    fn update(
        &self,
        id: uuid::Uuid,
        name: Option<&str>,
        description: Option<&str>,
        permission_ids: Option<&[uuid::Uuid]>,
        status: Option<RoleStatus>,
    ) -> impl std::future::Future<Output = Result<Role, RepositoryError>> + Send;

    fn delete(
        &self,
        id: uuid::Uuid,
    ) -> impl std::future::Future<Output = Result<(), RepositoryError>> + Send;
}

pub trait PasswordTokenRepository: Send + Sync {
    fn create(
        &self,
        token: NewPasswordToken,
    ) -> impl Future<Output = Result<PasswordToken, RepositoryError>> + Send;

    /// Token còn hiệu lực theo hash: chưa dùng (used_at IS NULL) và chưa hết hạn.
    fn find_active_by_hash(
        &self,
        token_hash: &str,
    ) -> impl Future<Output = Result<Option<PasswordToken>, RepositoryError>> + Send;

    /// Vô hiệu mọi token còn hiệu lực của user theo loại (đặt used_at = NOW()).
    fn invalidate_active(
        &self,
        user_id: uuid::Uuid,
        token_type: PasswordTokenType,
    ) -> impl Future<Output = Result<(), RepositoryError>> + Send;
}
