use domain::repositories::PermissionRepository;

use crate::dto::role_dto::PermissionResponse;
use crate::errors::AppError;

pub struct PermissionService<PR: PermissionRepository> {
    permission_repo: PR,
}

impl<PR: PermissionRepository> PermissionService<PR> {
    pub fn new(permission_repo: PR) -> Self {
        Self { permission_repo }
    }

    pub async fn list(&self) -> Result<Vec<PermissionResponse>, AppError> {
        let perms = self.permission_repo.find_all().await?;
        Ok(perms.into_iter().map(PermissionResponse::from).collect())
    }
}
