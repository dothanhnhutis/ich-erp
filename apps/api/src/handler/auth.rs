use axum::{Json, extract::rejection::JsonRejection, http::StatusCode, response};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use validator::{Validate, ValidationError};

#[derive(Deserialize, Debug, Validate, Serialize)]
pub struct CreateUser {
    #[validate(email)]
    email: String,
    #[validate(length(min = 8, max = 20))]
    password: String,
}

pub async fn login_handler(
    result: Result<Json<CreateUser>, JsonRejection>,
) -> Result<response::Json<CreateUser>, (StatusCode, String)> {
    match result {
        Ok(payload) => match payload.validate() {
            Ok(body) => Ok(payload),
            Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string())),
        },
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Unknown error".to_string(),
        )),
    }
}
