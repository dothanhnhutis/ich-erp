use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{error::ApiError, extractors::validator::ValidatedBodyJson};

#[derive(Deserialize, Debug, Validate, Serialize)]
pub struct CreateUser {
    #[validate(email)]
    email: String,
    #[validate(length(min = 8, max = 20))]
    password: String,
}

pub async fn login_handler(
    ValidatedBodyJson(payload): ValidatedBodyJson<CreateUser>,
) -> Result<impl IntoResponse, ApiError> {
    println!("{:#?}", payload);
    Ok(("ok"))
}
