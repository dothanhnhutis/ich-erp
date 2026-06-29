use domain::{
    entities::session::{NewSession, Session},
    errors::DomainError,
    repositories::UserSessionRepository,
};
use sqlx::{
    PgPool,
    types::chrono::{DateTime, Utc},
};
use uuid::Uuid;

/// Struct riêng cho DB layer — tách biệt khỏi domain entity.
/// `ip_address` lấy về dạng text (`ip_address::text`) vì cột là kiểu INET.
#[derive(Debug, sqlx::FromRow)]
struct SessionRow {
    id: Uuid,
    user_id: Uuid,
    token_hash: String,
    device_id: Option<String>,
    device_name: Option<String>,
    device_type: String,
    platform: Option<String>,
    app_version: Option<String>,
    user_agent: Option<String>,
    ip_address: Option<String>,
    revoked_at: Option<DateTime<Utc>>,
    revoke_reason: Option<String>,
    expires_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

/// Mapping từ DB row → Domain entity (1:1, không có field nào cần convert).
impl From<SessionRow> for Session {
    fn from(row: SessionRow) -> Self {
        Session {
            id: row.id,
            user_id: row.user_id,
            token_hash: row.token_hash,
            device_id: row.device_id,
            device_name: row.device_name,
            device_type: row.device_type,
            platform: row.platform,
            app_version: row.app_version,
            user_agent: row.user_agent,
            ip_address: row.ip_address,
            revoked_at: row.revoked_at,
            revoke_reason: row.revoke_reason,
            expires_at: row.expires_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

/// `id`, `created_at`, `updated_at` do DB sinh; `revoked_at`/`revoke_reason` mặc định NULL.
/// `ip_address` được ép kiểu `$9::inet` khi insert và `ip_address::text` khi RETURNING.
const INSERT_SESSION: &str = r#"
    INSERT INTO user_sessions
        (user_id, token_hash, device_id, device_name, device_type,
         platform, app_version, user_agent, ip_address, expires_at)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9::inet, $10)
    RETURNING
        id, user_id, token_hash, device_id, device_name, device_type,
        platform, app_version, user_agent,
        ip_address::text AS ip_address,
        revoked_at, revoke_reason, expires_at, created_at, updated_at
"#;

fn map_sqlx_err(e: sqlx::Error) -> DomainError {
    if let sqlx::Error::Database(db) = &e {
        if db.is_unique_violation() {
            return AppError::Validation("Session token đã tồn tại".into());
        }
        if db.is_foreign_key_violation() {
            return AppError::Validation("User không tồn tại".into());
        }
    }
    DomainError::Conflict(e.to_string())
}

#[derive(Clone)]
pub struct PgUserSessionRepository {
    pool: PgPool,
}

impl PgUserSessionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl UserSessionRepository for PgUserSessionRepository {
    async fn create(&self, new_session: NewSession) -> Result<Session, DomainError> {
        let row: SessionRow = sqlx::query_as(INSERT_SESSION)
            .bind(new_session.user_id)
            .bind(new_session.token_hash)
            .bind(new_session.device_id)
            .bind(new_session.device_name)
            .bind(new_session.device_type)
            .bind(new_session.platform)
            .bind(new_session.app_version)
            .bind(new_session.user_agent)
            .bind(new_session.ip_address)
            .bind(new_session.expires_at)
            .fetch_one(&self.pool)
            .await
            .map_err(map_sqlx_err)?;

        Ok(row.into())
    }
}
