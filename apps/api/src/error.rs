use application::error::AppError;
use axum::{
    Json,
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;
use validator::ValidationErrors;
// pub struct ApiError(pub AppError);

#[derive(Debug, Error)]
pub enum ApiError {
    #[error(transparent)]
    JsonRejection(#[from] JsonRejection), // input: body hỏng / sai content-type / sai kiểu

    #[error(transparent)]
    Validation(#[from] ValidationErrors), // input: fail rule validator (email, length...)

    #[error(transparent)]
    Domain(#[from] AppError), // nghiệp vụ bubble từ use case lên
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        // let (status, message) = match &self.0 {
        //     AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
        //     AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
        //     AppError::Internal(msg) => {
        //         // tracing::error!("Internal error: {}", msg);
        //         (StatusCode::INTERNAL_SERVER_ERROR, "Lỗi hệ thống".into())
        //     }
        // };

        // (status, Json(json!({ "error": message }))).into_response()

        match self {
            // JsonRejection tự biết status: 400 syntax / 415 content-type / 422 sai kiểu
            ApiError::JsonRejection(rejection) => {
                let status = rejection.status();
                (status, Json(json!({ "error": rejection.body_text() }))).into_response()
            }
            // field-level errors → FE map được vào từng input (kiểu Zod)
            ApiError::Validation(errors) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({ "errors": fields_to_json(&errors) })),
            )
                .into_response(),
            ApiError::Domain(err) => domain_to_response(err),
        }
    }
}

fn domain_to_response(err: AppError) -> Response {
    let (status, msg) = match err {
        AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
        AppError::Validation(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg),
        AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
        AppError::Internal(msg) => {
            tracing::error!("Internal error: {msg}"); // log đầy đủ phía server
            (StatusCode::INTERNAL_SERVER_ERROR, "Lỗi hệ thống".to_owned()) // client nhận câu chung
        } // AppError::Conflict(msg) => (StatusCode::CONFLICT, msg),
    };
    (status, Json(json!({ "error": msg }))).into_response()
}

fn fields_to_json(errors: &ValidationErrors) -> serde_json::Value {
    let map: serde_json::Map<_, _> = errors
        .field_errors()
        .iter()
        .map(|(field, errs)| {
            let msgs: Vec<String> = errs
                .iter()
                .map(|e| {
                    e.message
                        .as_ref()
                        .map(|m| m.to_string())
                        .unwrap_or_else(|| format!("{field} không hợp lệ"))
                })
                .collect();
            (field.to_string(), json!(msgs))
        })
        .collect();
    json!(map)
}

// impl From<JsonRejection> for ApiError {
//     fn from(rejection: JsonRejection) -> Self {
//         Self::JsonRejection(rejection.body_text())
//     }
// }

// impl From<ValidationErrors> for ApiError {
//     fn from(errors: ValidationErrors) -> Self {
//         let message = errors
//             .field_errors()
//             .iter()
//             .flat_map(|(field, errs)| {
//                 errs.iter().map(move |e| {
//                     let msg = e
//                         .message
//                         .as_ref()
//                         .map(|m| m.to_string())
//                         .unwrap_or_else(|| format!("{} không hợp lệ", field));
//                     format!("{}: {}", field, msg)
//                 })
//             })
//             .collect::<Vec<_>>()
//             .join("; ");
//         Self(AppError::Validation(message))
//     }
// }
