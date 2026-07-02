use axum::extract::FromRequestParts;
use axum::http::request::Parts;

use application::errors::AppError;
use domain::entities::session::Session;
use domain::entities::user::User;

use crate::error::ApiError;
/// Ngữ cảnh xác thực, được middleware nhét vào request extensions
/// để các handler protected đọc lại qua `Extension<AuthContext>`.

#[derive(Clone, Debug)]
pub struct AuthContext {
    pub user: User,
    pub session: Session,
    pub permission_codes: Vec<String>,
}

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
