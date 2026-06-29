use domain::errors::DomainError;
use sqlx::error::ErrorKind;

pub fn map_sqlx_error(err: sqlx::Error) -> DomainError {
    if let Some(db) = err.as_database_error() {
        match db.kind() {
            ErrorKind::UniqueViolation => {
                // constraint() cho biết ràng buộc NÀO bị vi phạm
                let msg = match db.constraint() {
                    Some("users_email_key") => "Email đã được sử dụng",
                    Some("users_username_key") => "Username đã được sử dụng",
                    _ => "Dữ liệu đã tồn tại",
                };
                return DomainError::Conflict(msg.into());
            }
            // ErrorKind::ForeignKeyViolation => {
            //     return DomainError::Validation("Tham chiếu không hợp lệ".into());
            // }
            _ => {}
        }
    }
    DomainError::Conflict(err.to_string())
}
