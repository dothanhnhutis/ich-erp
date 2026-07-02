use application::errors::AppError;
use axum::{Json, response::IntoResponse};

use crate::{error::ApiError, extractors::authenticated_user::AuthContext};

/// Trả về thông tin user đang đăng nhập (AuthContext do middleware require_auth gắn).
pub async fn me(data: AuthContext) -> Result<impl IntoResponse, ApiError> {
    // println!("{data:#?}");

    // if !data.permission_codes.iter().any(|p| p == "s") {
    //     return Err(ApiError::Domain(AppError::Unauthorized(
    //         "Thiếu thông tin xác thực".into(),
    //     )));
    // }

    Ok(Json(data.user))
}
