use axum::extract::FromRequestParts;
use axum::http::request::Parts;

use application::errors::AppError;

use crate::error::ApiError;
use crate::middleware::auth::AuthContext;

impl<S> FromRequestParts<S> for AuthContext
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthContext>()
            .cloned()
            .ok_or_else(|| {
                ApiError::Domain(AppError::Unauthorized("Thiếu thông tin xác thực".into()))
            })
    }
}
