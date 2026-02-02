use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use landscape_common::api_response::LandscapeApiResp as CommonLandscapeApiResp;
use serde::Serialize;

use crate::error::LandscapeApiError;

#[derive(Debug, Serialize, Default, Clone)]
pub struct LandscapeApiResp<T>(pub CommonLandscapeApiResp<T>);

impl<T> LandscapeApiResp<T> {
    pub fn success(data: T) -> Result<LandscapeApiResp<T>, LandscapeApiError> {
        Ok(Self(CommonLandscapeApiResp::success(data)))
    }

    pub fn error(code: u32, message: impl Into<String>) -> Self {
        Self(CommonLandscapeApiResp::error(code, message))
    }
}

impl<T: Serialize> IntoResponse for LandscapeApiResp<T> {
    fn into_response(self) -> Response {
        let http_code = self.0.code % 1000;
        let status =
            StatusCode::from_u16(http_code as u16).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, Json(self.0)).into_response()
    }
}
