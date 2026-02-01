use axum::{
    extract::{Query, State},
    routing::{get, post},
    Json, Router,
};
use landscape_common::metric::connect::{
    ConnectHistoryStatus, ConnectKey, ConnectMetric, ConnectRealtimeStatus,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize)]
pub struct QueryHistoryParams {
    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
    pub limit: Option<usize>,
}

use crate::{api::LandscapeApiResp, error::LandscapeApiResult};

use crate::LandscapeApp;

pub async fn get_metric_service_paths() -> Router<LandscapeApp> {
    Router::new()
        .route("/status", get(get_metric_status))
        .route("/connects", get(get_connects_info))
        .route("/connects/chart", post(get_connect_metric_info))
        .route("/connects/history", get(get_connect_history))
}

pub async fn get_metric_status(State(state): State<LandscapeApp>) -> LandscapeApiResult<Value> {
    LandscapeApiResp::success(serde_json::to_value(&state.metric_service.status).unwrap())
}

pub async fn get_connects_info(
    State(state): State<LandscapeApp>,
) -> LandscapeApiResult<Vec<ConnectRealtimeStatus>> {
    let data = state.metric_service.data.connect_metric.connect_infos().await;
    LandscapeApiResp::success(data)
}

pub async fn get_connect_metric_info(
    State(state): State<LandscapeApp>,
    Json(key): Json<ConnectKey>,
) -> LandscapeApiResult<Vec<ConnectMetric>> {
    let data = state.metric_service.data.connect_metric.query_metric_by_key(key).await;
    LandscapeApiResp::success(data)
}

pub async fn get_connect_history(
    State(state): State<LandscapeApp>,
    Query(params): Query<QueryHistoryParams>,
) -> LandscapeApiResult<Vec<ConnectHistoryStatus>> {
    let data = state
        .metric_service
        .data
        .connect_metric
        .history_summaries(params.limit, params.start_time, params.end_time)
        .await;
    LandscapeApiResp::success(data)
}
