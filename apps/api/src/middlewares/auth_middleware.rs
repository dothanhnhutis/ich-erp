use axum::extract::{Request, State};
use axum::http::{HeaderMap, header::AUTHORIZATION};
use axum::middleware::Next;
use axum::response::Response;
use axum_extra::extract::cookie::CookieJar;

use application::errors::AppError;

use crate::AppState;
use crate::error::ApiError;
use crate::extractors::auth_context::AuthContext;

/// Middleware bắt buộc đăng nhập: lấy token từ Bearer/cookie, xác thực, gắn AuthContext.
pub async fn require_auth(
    State(state): State<AppState>,
    jar: CookieJar,
    mut req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let token = extract_token(req.headers(), &jar)
        .ok_or_else(|| AppError::Unauthorized("Thiếu thông tin xác thực".into()))?;

    let (session, user, permission_codes) = state.auth_service.authenticate(&token).await?;

    req.extensions_mut().insert(AuthContext {
        session,
        user,
        permission_codes,
    });

    Ok(next.run(req).await)
}

/// Ưu tiên `Authorization: Bearer <token>`, sau đó tới cookie `session`.
fn extract_token(headers: &HeaderMap, jar: &CookieJar) -> Option<String> {
    if let Some(value) = headers.get(AUTHORIZATION).and_then(|v| v.to_str().ok())
        && let Some(token) = value.strip_prefix("Bearer ")
    {
        let token = token.trim();
        if !token.is_empty() {
            return Some(token.to_owned());
        }
    }

    jar.get("session").map(|c| c.value().to_owned())
}
