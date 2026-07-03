#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("Trạng thái user không hợp lệ: {0}")]
    InvalidUserStatus(String),

    #[error("Email không hợp lệ: {0}")]
    InvalidEmail(String),

    #[error("Trạng thái role không hợp lệ: {0}")]
    InvalidRoleStatus(String),

    #[error("Password token không hợp lệ: {0}")]
    InvalidPasswordTokenType(String),

    #[error("{0}")]
    Conflict(String),
}
