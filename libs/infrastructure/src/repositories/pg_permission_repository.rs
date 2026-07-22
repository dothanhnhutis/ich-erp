use domain::{
    entities::permission::Permission,
    repositories::{PermissionRepository, RepositoryError},
};
use sqlx::{
    PgPool,
    types::chrono::{DateTime, Utc},
};
use uuid::Uuid;

use crate::persistence::error::map_sqlx_error;

#[derive(Debug, sqlx::FromRow)]
struct PermissionRow {
    id: Uuid,
    code: String,
    description: String,
    created_at: DateTime<Utc>,
}

impl From<PermissionRow> for Permission {
    fn from(r: PermissionRow) -> Self {
        Permission {
            id: r.id,
            code: r.code,
            description: r.description,
            created_at: r.created_at,
        }
    }
}

#[derive(Clone)]
pub struct PgPermissionRepository {
    pool: PgPool,
}

impl PgPermissionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl PermissionRepository for PgPermissionRepository {
    async fn find_all(&self) -> Result<Vec<Permission>, RepositoryError> {
        let rows: Vec<PermissionRow> = sqlx::query_as(
            r#"
            SELECT id, code, description, created_at
            FROM permissions
            ORDER BY code
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(Permission::from).collect())
    }
}
