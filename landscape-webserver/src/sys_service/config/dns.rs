use axum::extract::State;
use axum::Json;
use landscape_common::config::{GetDnsConfigResponse, UpdateDnsConfigRequest};

use crate::api::LandscapeApiResp;
use crate::error::LandscapeApiResult;
use crate::LandscapeApp;

pub async fn get_dns_config_fast(
    State(state): State<LandscapeApp>,
) -> LandscapeApiResult<GetDnsConfigResponse> {
    let (dns, hash) = state.config_service.get_dns_config();
    LandscapeApiResp::success(GetDnsConfigResponse { dns, hash })
}

pub async fn get_dns_config(
    State(state): State<LandscapeApp>,
) -> LandscapeApiResult<GetDnsConfigResponse> {
    let (dns, hash) = state.config_service.get_dns_config_from_file().await;
    LandscapeApiResp::success(GetDnsConfigResponse { dns, hash })
}

pub async fn update_dns_config(
    State(state): State<LandscapeApp>,
    Json(payload): Json<serde_json::Value>,
) -> LandscapeApiResult<String> {
    let request: UpdateDnsConfigRequest = serde_json::from_value(payload)?;
    state.config_service.update_dns_config(request.new_dns, request.expected_hash).await?;
    LandscapeApiResp::success("success".to_string())
}
