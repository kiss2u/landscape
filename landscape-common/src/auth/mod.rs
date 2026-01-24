use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "common/auth.d.ts")]
pub struct LoginInfo {
    pub username: String,
    pub password: String,
}

#[derive(Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "common/auth.d.ts")]
pub struct LoginResult {
    pub success: bool,
    pub token: String,
}
