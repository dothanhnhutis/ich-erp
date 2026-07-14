use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, Debug, Serialize)]
#[serde(tag = "kind")]
pub enum ApiError {
    #[error("{message}")]
    WindowError { message: String },

    #[error("{message}")]
    Network { message: String },

    #[error("{message}")]
    Unauthorized { message: String },

    #[error("Server error ({status}): {message}")]
    Server { status: u16, message: String },

    #[error("Chưa đăng nhập")]
    NotAuthenticated,
}

impl From<reqwest::Error> for ApiError {
    fn from(e: reqwest::Error) -> Self {
        ApiError::Network {
            message: e.to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ErrorBody {
    error: String,
}

pub async fn map_response_error(res: reqwest::Response) -> ApiError {
    let status = res.status().as_u16();
    let text = res.text().await.unwrap_or_default();
    let message = serde_json::from_str::<ErrorBody>(&text)
        .map(|e| e.error)
        .unwrap_or_else(|_| {
            if text.is_empty() {
                format!("HTTP {}", status)
            } else {
                text
            }
        });

    if status == 401 {
        ApiError::Unauthorized { message }
    } else {
        ApiError::Server { status, message }
    }
}
