use crate::api::LandscapeApiResp;
use axum::extract::State;
use axum::Json;
use landscape_common::config::{GetUIConfigResponse, LandscapeUIConfig, UpdateUIConfigRequest};

use crate::error::LandscapeApiResult;
use crate::LandscapeApp;

pub async fn get_ui_config(
    State(state): State<LandscapeApp>,
) -> LandscapeApiResult<GetUIConfigResponse> {
    let (config, hash) = state.config_service.get_config_with_hash().await?;
    LandscapeApiResp::success(GetUIConfigResponse { ui: config.ui, hash })
}

pub async fn get_ui_config_fast(
    State(state): State<LandscapeApp>,
) -> LandscapeApiResult<LandscapeUIConfig> {
    let ui_config = state.config_service.get_ui_config_from_memory();
    LandscapeApiResp::success(ui_config)
}

pub async fn update_ui_config(
    State(state): State<LandscapeApp>,
    Json(payload): Json<UpdateUIConfigRequest>,
) -> LandscapeApiResult<()> {
    state.config_service.update_ui_config(payload.new_ui, payload.expected_hash).await?;
    LandscapeApiResp::success(())
}
