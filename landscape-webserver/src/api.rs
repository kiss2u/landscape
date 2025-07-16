use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use ts_rs::TS;

use crate::error::LandscapeApiError;

#[derive(Debug, Serialize, Default, Clone, TS)]
#[ts(export, export_to = "webserver.d.ts")]
pub struct LandscapeApiResp<T> {
    code: u32,
    message: String,
    data: Option<T>,
}

impl<T> LandscapeApiResp<T> {
    pub fn success(data: T) -> Result<LandscapeApiResp<T>, LandscapeApiError> {
        Ok(Self {
            code: 200,
            message: "success".to_string(),
            data: Some(data),
        })
    }

    pub fn error(code: u32, message: impl Into<String>) -> Self {
        Self { code, message: message.into(), data: None }
    }
}

impl<T: Serialize> IntoResponse for LandscapeApiResp<T> {
    fn into_response(self) -> Response {
        let http_code = self.code % 1000;
        let status =
            StatusCode::from_u16(http_code as u16).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, Json(self)).into_response()
    }
}
