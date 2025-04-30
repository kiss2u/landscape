use axum::{http::StatusCode, response::IntoResponse};
use landscape_common::error::LdError;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(thiserror::Error, Debug)]
pub enum LandscapeApiError {
    #[error("`{0}` not found")]
    NotFound(String),
    #[error("Failed to parse configuration: {0}")]
    JsonParseError(#[from] serde_json::Error),
    #[error("IO error occurred: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Landscape Error: {0}")]
    LdError(#[from] LdError),
}

impl IntoResponse for LandscapeApiError {
    fn into_response(self) -> axum::response::Response {
        let msg = self.to_string();

        let code = match self {
            LandscapeApiError::NotFound(_) => StatusCode::NOT_FOUND,
            LandscapeApiError::JsonParseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            LandscapeApiError::IoError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            LandscapeApiError::LdError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = axum::Json(serde_json::json!(ErrorMsg { msg }));
        (code, body).into_response()
    }
}

pub type LandscapeResult<T> = Result<T, LandscapeApiError>;

#[derive(Debug, Serialize, Deserialize, Default, Clone, TS)]
#[ts(export, export_to = "common.ts")]
pub struct ErrorMsg {
    msg: String,
}
