use domain::{
    entities::user::{User, UserStatus},
    errors::DomainError,
    repositories::UserRepository,
};
use sqlx::{
    AssertSqlSafe, PgPool,
    types::chrono::{DateTime, Utc},
};
use std::str::FromStr;
use uuid::Uuid;

use crate::persistence::error::map_sqlx_error;

#[derive(Debug, sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    email: String,
    password_hash: Option<String>,
    username: Option<String>,
    status: String,
    deactivated_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

/// Mapping từ DB row → Domain entity
impl TryFrom<UserRow> for User {
    type Error = DomainError;
    fn try_from(row: UserRow) -> Result<Self, Self::Error> {
        Ok(User {
            id: row.id,
            email: row.email,
            password_hash: row.password_hash,
            username: row.username,
            status: UserStatus::from_str(&row.status)?,
            deactivated_at: row.deactivated_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

const SELECT_USER: &str = r#"
    SELECT id, email, password_hash, username, status,
           deactivated_at, created_at, updated_at
    FROM users
"#;

#[derive(Clone)]
pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// fn map_sqlx_err(e: sqlx::Error) -> DomainError {
//     if let sqlx::Error::Database(db) = &e {
//         if db.is_unique_violation() {
//             return DomainError::AlreadyExists("Email đã tồn tại".into());
//         }
//         if db.is_foreign_key_violation() {
//             return DomainError::Validation("Role không tồn tại".into());
//         }
//     }
//     DomainError::Internal(e.to_string())
// }

impl UserRepository for PgUserRepository {
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, DomainError> {
        let sql = format!("{} WHERE email = $1 AND deleted_at IS NULL", SELECT_USER);
        let row: Option<UserRow> = sqlx::query_as(AssertSqlSafe(sql))
            .bind(email)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx_error)?;

        row.map(User::try_from).transpose()
    }
}
