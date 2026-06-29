use application::dto::auth_dto::LoginRequest;
use axum::{extract::State, response::IntoResponse};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{AppState, error::ApiError, extractors::validator::ValidatedBodyJson};

pub async fn login_handler(
    State(state): State<AppState>,
    ValidatedBodyJson(payload): ValidatedBodyJson<LoginRequest>,
) -> Result<impl IntoResponse, ApiError> {
    println!("{:#?}", payload);

    let response = state.auth_service.login(payload).await?;

    println!("{:#?}", response);

    Ok(("ok"))
}
