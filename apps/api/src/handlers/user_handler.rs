use application::{
    dto::{
        pagination_dto::{PaginatedResponse, PaginationParams},
        user_dto::{CreateUserRequest, UpdateUserRequest, UserResponse},
    },
    errors::AppError,
};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::json;
use sqlx::types::uuid;

use crate::{
    AppState,
    error::ApiError,
    extractors::{auth_context::AuthContext, validator::ValidatedBodyJson},
};

// Trả về thông tin user đang đăng nhập (AuthContext do middleware require_auth gắn).
pub async fn me(data: AuthContext) -> impl IntoResponse {
    Json(
        json!({"user":UserResponse::from(data.user), "session": data.session, "permission_codes": data.permission_codes } ),
    )
}

pub async fn list(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<UserResponse>>, ApiError> {
    let result = state.user_service.list_paginated(&params).await?;
    Ok(Json(result))
}

// Admin tạo user mới (cần permission USER_CREATE — đã kiểm ở middleware).
pub async fn create_user(
    data: AuthContext,
    State(state): State<AppState>,
    ValidatedBodyJson(payload): ValidatedBodyJson<CreateUserRequest>,
) -> Result<impl IntoResponse, ApiError> {
    if !data.permission_codes.iter().any(|p| p == "USER_CREATE") {
        return Err(ApiError::Domain(AppError::Forbidden(format!(
            "Cần quyền: {}",
            "USER_CREATE"
        ))));
    }

    let res = state.user_service.create_user(payload).await?;
    Ok((StatusCode::CREATED, Json(res)))
}

// Cập nhật username/status của user (cần USER_UPDATE). Vô hiệu hoá → thu hồi phiên ngay.
pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
    ValidatedBodyJson(payload): ValidatedBodyJson<UpdateUserRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let res = state.user_service.update_user(id, payload).await?;
    if res.status == "DEACTIVATED" {
        state.auth_service.logout_all(id).await?;
    }
    Ok(Json(res))
}

// Xoá mềm user (cần USER_DELETE) + thu hồi mọi phiên đăng nhập của user đó.
pub async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    state.user_service.delete_user(id).await?;
    state.auth_service.logout_all(id).await?;
    Ok(Json(json!({ "message": "Đã xoá người dùng" })))
}

// Admin gửi lại email thiết lập tài khoản cho user chưa kích hoạt (cần USER_CREATE).
pub async fn resend_setup(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    state.user_service.resend_setup(id).await?;
    Ok(Json(
        json!({ "message": "Đã gửi lại email thiết lập tài khoản" }),
    ))
}
