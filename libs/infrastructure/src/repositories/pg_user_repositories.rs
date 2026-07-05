use domain::{
    entities::user::{User, UserStatus},
    errors::DomainError,
    repositories::{RepositoryError, UserRepository},
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

impl UserRepository for PgUserRepository {
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, RepositoryError> {
        let sql = format!("{} WHERE email = $1 AND deleted_at IS NULL", SELECT_USER);
        let row: Option<UserRow> = sqlx::query_as(AssertSqlSafe(sql))
            .bind(email)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx_error)?;

        Ok(row.map(User::try_from).transpose()?) // (4) ? tự đổi DomainError -> Mapping
    }

    async fn find_by_id(&self, id: uuid::Uuid) -> Result<Option<User>, RepositoryError> {
        let sql = format!("{} WHERE id = $1 AND deleted_at IS NULL", SELECT_USER);
        let row: Option<UserRow> = sqlx::query_as(AssertSqlSafe(sql))
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx_error)?;

        Ok(row.map(User::try_from).transpose()?)
    }

    async fn create_with_roles(
        &self,
        new_user: domain::entities::user::NewUser,
        role_ids: &[uuid::Uuid],
    ) -> Result<User, RepositoryError> {
        let mut tx = self.pool.begin().await.map_err(map_sqlx_error)?;

        // Tạo user — status mặc định PENDING_PASSWORD, id do DB sinh.
        let row: UserRow = sqlx::query_as(
            r#"
            INSERT INTO users (email)
            VALUES ($1)
            RETURNING id, email, password_hash, username, status,
                      deactivated_at, created_at, updated_at
            "#,
        )
        .bind(&new_user.email)
        .fetch_one(&mut *tx)
        .await
        .map_err(map_sqlx_error)?; // unique_violation → "Email đã tồn tại"

        // Gán role — FK RESTRICT đảm bảo role tồn tại (fk_violation → "Role không tồn tại").
        for role_id in role_ids {
            sqlx::query(r#"INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2)"#)
                .bind(row.id)
                .bind(role_id)
                .execute(&mut *tx)
                .await
                .map_err(map_sqlx_error)?;
        }

        tx.commit().await.map_err(map_sqlx_error)?;
        Ok(User::try_from(row)?)
    }

    async fn activate_account(
        &self,
        user_id: uuid::Uuid,
        username: &str,
        password_hash: &str,
        token_id: uuid::Uuid,
    ) -> Result<(), RepositoryError> {
        let mut tx = self.pool.begin().await.map_err(map_sqlx_error)?;

        sqlx::query(
            r#"
            UPDATE users
            SET username = $2, password_hash = $3, status = 'ACTIVE', password_changed_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .bind(username)
        .bind(password_hash)
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;

        mark_token_used(&mut tx, token_id).await?;

        tx.commit().await.map_err(map_sqlx_error)?;
        Ok(())
    }
}

// Đánh dấu password token đã dùng; rows_affected != 1 nghĩa là token đã bị dùng (race) → lỗi → rollback.
async fn mark_token_used(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    token_id: Uuid,
) -> Result<(), RepositoryError> {
    let used = sqlx::query(
        r#"UPDATE password_tokens SET used_at = NOW() WHERE id = $1 AND used_at IS NULL"#,
    )
    .bind(token_id)
    .execute(&mut **tx)
    .await
    .map_err(map_sqlx_error)?;

    if used.rows_affected() != 1 {
        return Err(RepositoryError::Mapping(DomainError::RevokedPasswordToken(
            "Liên kết đã được sử dụng".into(),
        )));
    }
    Ok(())
}
