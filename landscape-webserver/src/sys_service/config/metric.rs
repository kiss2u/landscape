use crate::api::LandscapeApiResp;
use axum::extract::State;
use axum::Json;
use landscape_common::config::{
    GetMetricConfigResponse, LandscapeMetricConfig, UpdateMetricConfigRequest,
};

use crate::error::LandscapeApiResult;
use crate::LandscapeApp;

pub async fn get_metric_config(
    State(state): State<LandscapeApp>,
) -> LandscapeApiResult<GetMetricConfigResponse> {
    let (config, hash) = state.config_service.get_config_with_hash().await?;
    LandscapeApiResp::success(GetMetricConfigResponse { metric: config.metric, hash })
}

pub async fn get_metric_config_fast(
    State(state): State<LandscapeApp>,
) -> LandscapeApiResult<LandscapeMetricConfig> {
    let metric_config = state.config_service.get_metric_config_from_memory();
    LandscapeApiResp::success(metric_config)
}

pub async fn update_metric_config(
    State(state): State<LandscapeApp>,
    Json(payload): Json<UpdateMetricConfigRequest>,
) -> LandscapeApiResult<()> {
    state.config_service.update_metric_config(payload.new_metric, payload.expected_hash).await?;
    LandscapeApiResp::success(())
}
