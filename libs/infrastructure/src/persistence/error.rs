use application::error::AppError;
use sqlx::error::ErrorKind;

pub fn map_sqlx_error(err: sqlx::Error) -> AppError {
    if let Some(db) = err.as_database_error() {
        match db.kind() {
            ErrorKind::UniqueViolation => {
                // constraint() cho biết ràng buộc NÀO bị vi phạm
                let msg = match db.constraint() {
                    Some("users_email_key") => "Email đã được sử dụng",
                    Some("users_username_key") => "Username đã được sử dụng",
                    _ => "Dữ liệu đã tồn tại",
                };
                return AppError::Conflict(msg.into());
            }
            ErrorKind::ForeignKeyViolation => {
                return AppError::Validation("Tham chiếu không hợp lệ".into());
            }
            _ => {} // ErrorKind là #[non_exhaustive] → BẮT BUỘC có nhánh chặn
        }
    }
    // còn lại: mất kết nối, timeout, decode... → nội bộ, không lộ cho client
    AppError::Internal(err.to_string())
}
