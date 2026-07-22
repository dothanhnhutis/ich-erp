use domain::repositories::RoleRepository;
use uuid::Uuid;

use crate::dto::pagination_dto::{PaginatedResponse, PaginationParams};
use crate::dto::role_dto::{CreateRoleRequest, RoleResponse, UpdateRoleRequest, to_role_response};
use crate::errors::AppError;

pub struct RoleService<R: RoleRepository> {
    role_repo: R,
}

impl<R: RoleRepository> RoleService<R> {
    pub fn new(role_repo: R) -> Self {
        Self { role_repo }
    }

    pub async fn list(&self) -> Result<Vec<RoleResponse>, AppError> {
        let roles = self.role_repo.find_all().await?;
        let mut result = Vec::with_capacity(roles.len());
        for role in roles {
            let perms = self.role_repo.find_permissions_for_role(role.id).await?;
            result.push(to_role_response(role, perms));
        }
        Ok(result)
    }

    pub async fn list_paginated(
        &self,
        params: &PaginationParams,
    ) -> Result<PaginatedResponse<RoleResponse>, AppError> {
        let (roles, total) = self
            .role_repo
            .find_paginated(params.offset(), params.page_size(), params.search())
            .await?;
        let mut items = Vec::with_capacity(roles.len());
        for role in roles {
            let perms = self.role_repo.find_permissions_for_role(role.id).await?;
            items.push(to_role_response(role, perms));
        }
        Ok(PaginatedResponse {
            items,
            total,
            page: params.page(),
            page_size: params.page_size(),
        })
    }

    pub async fn get(&self, id: Uuid) -> Result<RoleResponse, AppError> {
        let role = self
            .role_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Role {} không tồn tại", id)))?;
        let perms = self.role_repo.find_permissions_for_role(role.id).await?;
        Ok(to_role_response(role, perms))
    }

    pub async fn create(&self, req: CreateRoleRequest) -> Result<RoleResponse, AppError> {
        let role = self
            .role_repo
            .create(&req.name, &req.description, &req.permission_ids)
            .await?;
        let perms = self.role_repo.find_permissions_for_role(role.id).await?;
        Ok(to_role_response(role, perms))
    }

    pub async fn update(&self, id: Uuid, req: UpdateRoleRequest) -> Result<RoleResponse, AppError> {
        let role = self
            .role_repo
            .update(
                id,
                req.name.as_deref(),
                req.description.as_deref(),
                req.permission_ids.as_deref(),
                req.status,
            )
            .await?;
        let perms = self.role_repo.find_permissions_for_role(role.id).await?;
        Ok(to_role_response(role, perms))
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        self.role_repo.delete(id).await?;
        Ok(())
    }
}
