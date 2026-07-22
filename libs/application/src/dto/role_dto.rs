use chrono::{DateTime, Utc};
use domain::entities::{
    permission::Permission,
    role::{Role, RoleStatus},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionResponse {
    pub id: Uuid,
    pub code: String,
    pub description: String,
}

impl From<Permission> for PermissionResponse {
    fn from(p: Permission) -> Self {
        Self {
            id: p.id,
            code: p.code,
            description: p.description,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct CreateRoleRequest {
    #[validate(length(min = 1, max = 255, message = "Tên vai trò 1-255 ký tự"))]
    pub name: String,

    #[serde(default)]
    pub description: String,

    #[serde(default)]
    pub permission_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct UpdateRoleRequest {
    #[validate(length(min = 1, max = 255, message = "Tên vai trò 1-255 ký tự"))]
    pub name: Option<String>,
    pub description: Option<String>,
    /// `None` = giữ nguyên; `Some([])` = bỏ hết permission.
    pub permission_ids: Option<Vec<Uuid>>,
    pub status: Option<RoleStatus>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoleResponse {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub status: String,
    pub deactivated_at: Option<DateTime<Utc>>,
    pub can_delete: bool,
    pub can_update: bool,
    pub permissions: Vec<PermissionResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub fn to_role_response(role: Role, perms: Vec<Permission>) -> RoleResponse {
    RoleResponse {
        id: role.id,
        name: role.name,
        description: role.description,
        status: role.status.as_str().to_string(),
        deactivated_at: role.deactivated_at,
        can_delete: role.can_delete,
        can_update: role.can_update,
        permissions: perms.into_iter().map(PermissionResponse::from).collect(),
        created_at: role.created_at,
        updated_at: role.updated_at,
    }
}
