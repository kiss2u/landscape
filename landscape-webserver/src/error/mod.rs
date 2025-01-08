use axum::{http::StatusCode, response::IntoResponse, Json};

#[derive(thiserror::Error, Debug)]
pub enum LandscapeApiError {
    #[error("Interface `{0}` not found")]
    NotFound(String),
    #[error("Failed to parse configuration: {0}")]
    JsonParseError(#[from] serde_json::Error),
    #[error("IO error occurred: {0}")]
    IoError(#[from] std::io::Error),
}

impl IntoResponse for LandscapeApiError {
    fn into_response(self) -> axum::response::Response {
        match self {
            LandscapeApiError::NotFound(message) => {
                (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": message })))
                    .into_response()
            }
            LandscapeApiError::JsonParseError(message) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": message.to_string() })),
            )
                .into_response(),
            LandscapeApiError::IoError(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": error.to_string() })),
            )
                .into_response(),
        }
    }
}
