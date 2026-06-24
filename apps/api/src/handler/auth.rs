use axum::response;
use serde_json::{Value, json};

pub async fn login_handler() -> response::Json<Value> {
    response::Json(json!({ "data": 42 }))
}
