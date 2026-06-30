use domain::repositories::RepositoryError;
use sqlx::error::ErrorKind;

pub fn map_sqlx_error(err: sqlx::Error) -> RepositoryError {
    if let Some(db) = err.as_database_error() {
        match db.kind() {
            ErrorKind::UniqueViolation => {
                return RepositoryError::UniqueViolation(
                    db.constraint().unwrap_or_default().to_owned(),
                );
            }
            ErrorKind::ForeignKeyViolation => {
                return RepositoryError::ForeignKeyViolation(
                    db.constraint().unwrap_or_default().to_owned(),
                );
            }
            _ => {}
        }
    }
    RepositoryError::Unexpected(Box::new(err))
}
