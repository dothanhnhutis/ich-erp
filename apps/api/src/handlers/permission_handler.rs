use axum::Json;
use axum::extract::State;

use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::AppState;
use crate::error::ApiError;

pub async fn list(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let perms = state.permission_service.list().await?;
    Ok((StatusCode::OK, Json(perms)))
}
