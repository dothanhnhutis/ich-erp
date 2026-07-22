use application::{
    dto::{
        pagination_dto::{PaginatedResponse, PaginationParams},
        role_dto::{CreateRoleRequest, RoleResponse, UpdateRoleRequest},
    },
    errors::AppError,
};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use sqlx::types::uuid;

use crate::{
    AppState,
    error::ApiError,
    extractors::{auth_context::AuthContext, validator::ValidatedBodyJson},
};

pub async fn create(
    data: AuthContext,
    State(state): State<AppState>,
    ValidatedBodyJson(payload): ValidatedBodyJson<CreateRoleRequest>,
) -> Result<impl IntoResponse, ApiError> {
    if !data.permission_codes.iter().any(|p| p == "ROLE_CREATE") {
        return Err(ApiError::Domain(AppError::Forbidden(format!(
            "Cần quyền: {}",
            "ROLE_CREATE"
        ))));
    }
    let role = state.role_service.create(payload).await?;
    Ok((StatusCode::CREATED, Json(role)))
}

pub async fn list(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<RoleResponse>>, ApiError> {
    let result = state.role_service.list_paginated(&params).await?;
    Ok(Json(result))
}

pub async fn get_by_id(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<RoleResponse>, ApiError> {
    let role = state.role_service.get(id).await?;
    Ok(Json(role))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
    ValidatedBodyJson(payload): ValidatedBodyJson<UpdateRoleRequest>,
) -> Result<Json<RoleResponse>, ApiError> {
    let role = state.role_service.update(id, payload).await?;
    Ok(Json(role))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> Result<StatusCode, ApiError> {
    state.role_service.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}
