use application::{
    dto::user_dto::{CreateUserRequest, UserResponse},
    errors::AppError,
};
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

use crate::{
    AppState,
    error::ApiError,
    extractors::{auth_context::AuthContext, validator::ValidatedBodyJson},
};

/// Trả về thông tin user đang đăng nhập (AuthContext do middleware require_auth gắn).
pub async fn me(data: AuthContext) -> impl IntoResponse {
    Json(UserResponse::from(data.user))
}

/// Admin tạo user mới (cần permission USER_CREATE — đã kiểm ở middleware).
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
