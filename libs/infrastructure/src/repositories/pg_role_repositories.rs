use domain::{
    entities::{
        permission::Permission,
        role::{Role, RoleStatus},
    },
    errors::DomainError,
    repositories::{RepositoryError, RoleRepository},
};
use sqlx::{
    PgPool,
    types::chrono::{DateTime, Utc},
};
use std::str::FromStr;

use crate::persistence::error::map_sqlx_error;

#[derive(sqlx::FromRow)]
struct PermissionRow {
    id: uuid::Uuid,
    code: String,
    description: String,
}

impl From<PermissionRow> for Permission {
    fn from(r: PermissionRow) -> Self {
        Self {
            id: r.id,
            code: r.code,
            description: r.description,
        }
    }
}

#[derive(sqlx::FromRow)]
struct RoleRow {
    id: uuid::Uuid,
    name: String,
    description: String,
    status: String,
    deactivated_at: Option<DateTime<Utc>>,
    can_delete: bool,
    can_update: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<RoleRow> for Role {
    type Error = DomainError;
    fn try_from(r: RoleRow) -> Result<Self, Self::Error> {
        Ok(Role {
            id: r.id,
            name: r.name,
            description: r.description,
            status: RoleStatus::from_str(&r.status)?,
            deactivated_at: r.deactivated_at,
            can_delete: r.can_delete,
            can_update: r.can_update,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
    }
}

const PERMISSION_CODES_FOR_USER: &str = r#"
    SELECT DISTINCT p.code
    FROM user_roles ur
    JOIN role_permissions rp ON rp.role_id = ur.role_id
    JOIN permissions p ON p.id = rp.permission_id
    JOIN roles r ON r.id = ur.role_id
    WHERE ur.user_id = $1
      AND r.status = 'ACTIVE'
      AND r.deleted_at IS NULL
"#;

#[derive(Clone)]
pub struct PgRoleRepository {
    pool: PgPool,
}

impl PgRoleRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl RoleRepository for PgRoleRepository {
    async fn find_permission_codes_for_user(
        &self,
        user_id: uuid::Uuid,
    ) -> Result<Vec<String>, RepositoryError> {
        let codes: Vec<String> = sqlx::query_scalar(PERMISSION_CODES_FOR_USER)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_error)?;
        Ok(codes)
    }
}
