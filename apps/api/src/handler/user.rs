use axum::response::IntoResponse;

use crate::middleware::auth::AuthContext;

/// Trả về thông tin user đang đăng nhập (AuthContext do middleware require_auth gắn).
pub async fn me(data: AuthContext) -> impl IntoResponse {
    println!("{data:#?}");
    "oker"
}
