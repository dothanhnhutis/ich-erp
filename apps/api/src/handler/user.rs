use axum::response::IntoResponse;

/// Trả về thông tin user đang đăng nhập (AuthContext do middleware require_auth gắn).
pub async fn me() -> impl IntoResponse {
    "oker"
}
