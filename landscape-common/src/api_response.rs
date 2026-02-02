use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Default, Clone, TS)]
#[ts(export, export_to = "common/api.ts")]
pub struct LandscapeApiResp<T> {
    pub code: u32,
    pub message: String,
    pub data: Option<T>,
}

impl<T> LandscapeApiResp<T> {
    pub fn success(data: T) -> Self {
        Self {
            code: 200,
            message: "success".to_string(),
            data: Some(data),
        }
    }

    pub fn error(code: u32, message: impl Into<String>) -> Self {
        Self { code, message: message.into(), data: None }
    }
}
