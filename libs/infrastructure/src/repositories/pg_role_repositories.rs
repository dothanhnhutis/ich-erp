use domain::{
    entities::{
        permission::Permission,
        role::{Role, RoleStatus},
    },
    errors::DomainError,
    repositories::{RepositoryError, RoleRepository},
};
use sqlx::{
    AssertSqlSafe, PgPool,
    types::chrono::{DateTime, Utc},
};
use std::str::FromStr;

use crate::persistence::error::map_sqlx_error;

#[derive(sqlx::FromRow)]
struct PermissionRow {
    id: uuid::Uuid,
    code: String,
    description: String,
    created_at: DateTime<Utc>,
}

impl From<PermissionRow> for Permission {
    fn from(r: PermissionRow) -> Self {
        Self {
            id: r.id,
            code: r.code,
            description: r.description,
            created_at: r.created_at,
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

const SELECT_ROLE: &str = r#"
    SELECT id, name, description, status, can_delete, can_update,
           deactivated_at, deleted_at, created_at, updated_at
    FROM roles
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

    async fn find_all(&self) -> Result<Vec<Role>, RepositoryError> {
        let sql = format!(
            "{} WHERE deleted_at IS NULL ORDER BY created_at DESC",
            SELECT_ROLE
        );
        let rows: Vec<RoleRow> = sqlx::query_as(AssertSqlSafe(sql))
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_error)?;

        let roles = rows
            .into_iter()
            .map(Role::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(roles)
    }

    async fn find_paginated(
        &self,
        offset: u32,
        limit: u32,
        search: Option<&str>,
    ) -> Result<(Vec<Role>, u64), RepositoryError> {
        let like_pattern = search.map(|s| format!("%{}%", s));

        let total: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM roles
            WHERE deleted_at IS NULL
              AND ($1::text IS NULL OR name ILIKE $1 OR description ILIKE $1)
            "#,
        )
        .bind(like_pattern.as_deref())
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        let sql = format!(
            r#"
            {}
            WHERE deleted_at IS NULL
              AND ($1::text IS NULL OR name ILIKE $1 OR description ILIKE $1)
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            SELECT_ROLE
        );
        let rows: Vec<RoleRow> = sqlx::query_as(AssertSqlSafe(sql))
            .bind(like_pattern.as_deref())
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_error)?;

        let roles = rows
            .into_iter()
            .map(Role::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok((roles, total as u64))
    }

    async fn find_by_id(&self, id: uuid::Uuid) -> Result<Option<Role>, RepositoryError> {
        let sql = format!("{} WHERE id = $1 AND deleted_at IS NULL", SELECT_ROLE);
        let row: Option<RoleRow> = sqlx::query_as(AssertSqlSafe(sql))
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx_error)?;

        Ok(row.map(Role::try_from).transpose()?)
    }

    async fn find_permissions_for_role(
        &self,
        role_id: uuid::Uuid,
    ) -> Result<Vec<Permission>, RepositoryError> {
        let rows: Vec<PermissionRow> = sqlx::query_as(
            r#"
            SELECT p.id, p.code, p.description, p.created_at
            FROM permissions p
            JOIN role_permissions rp ON rp.permission_id = p.id
            WHERE rp.role_id = $1
            ORDER BY p.code
            "#,
        )
        .bind(role_id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(Permission::from).collect())
    }

    async fn create(
        &self,
        name: &str,
        description: &str,
        permission_ids: &[uuid::Uuid],
    ) -> Result<Role, RepositoryError> {
        let mut tx = self.pool.begin().await.map_err(map_sqlx_error)?;

        let row: RoleRow = sqlx::query_as(
            r#"
            INSERT INTO roles (name, description)
            VALUES ($1, $2)
            RETURNING id, name, description, status, can_delete, can_update,
                      deactivated_at, deleted_at, created_at, updated_at
            "#,
        )
        .bind(name)
        .bind(description)
        .fetch_one(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;

        if !permission_ids.is_empty() {
            sqlx::query(
                r#"
                INSERT INTO role_permissions (role_id, permission_id)
                SELECT $1, UNNEST($2::uuid[])
                "#,
            )
            .bind(row.id)
            .bind(permission_ids)
            .execute(&mut *tx)
            .await
            .map_err(map_sqlx_error)?;
        }

        tx.commit().await.map_err(map_sqlx_error)?;

        Ok(Role::try_from(row)?)
    }

    async fn update(
        &self,
        id: uuid::Uuid,
        name: Option<&str>,
        description: Option<&str>,
        permission_ids: Option<&[uuid::Uuid]>,
        status: Option<RoleStatus>,
    ) -> Result<Role, RepositoryError> {
        let mut tx = self.pool.begin().await.map_err(map_sqlx_error)?;
        let status_str = status.map(|s| s.as_str());

        let row: RoleRow = sqlx::query_as(
            r#"
            UPDATE roles
            SET name           = COALESCE($2, name),
                description    = COALESCE($3, description),
                status         = COALESCE($4, status),
                deactivated_at = CASE
                                   WHEN $4 = 'DEACTIVATED' THEN NOW()
                                   WHEN $4 = 'ACTIVE'      THEN NULL
                                   ELSE deactivated_at
                                 END,
                updated_at     = NOW()
            WHERE id = $1 AND deleted_at IS NULL AND can_update = TRUE
            RETURNING id, name, description, status, can_delete, can_update,
                      deactivated_at, deleted_at, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(description)
        .bind(status_str)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_sqlx_error)?
        .ok_or(RepositoryError::NotFound)?;

        if let Some(ids) = permission_ids {
            sqlx::query("DELETE FROM role_permissions WHERE role_id = $1")
                .bind(id)
                .execute(&mut *tx)
                .await
                .map_err(map_sqlx_error)?;

            if !ids.is_empty() {
                sqlx::query(
                    r#"
                    INSERT INTO role_permissions (role_id, permission_id)
                    SELECT $1, UNNEST($2::uuid[])
                    "#,
                )
                .bind(id)
                .bind(ids)
                .execute(&mut *tx)
                .await
                .map_err(map_sqlx_error)?;
            }
        }

        tx.commit().await.map_err(map_sqlx_error)?;
        Ok(Role::try_from(row)?)
    }

    async fn delete(&self, id: uuid::Uuid) -> Result<(), RepositoryError> {
        let rows = sqlx::query(
            r#"
            UPDATE roles
            SET deleted_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL AND can_delete = TRUE
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?
        .rows_affected();

        if rows == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }
}
