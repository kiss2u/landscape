use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use landscape::config_service::dst_ip_rule::DstIpRuleService;
use landscape_common::service::controller_service::{ConfigController, FlowConfigController};
use landscape_common::{
    config::{ConfigId, FlowId},
    ip_mark::WanIpRuleConfig,
};
use landscape_database::provider::LandscapeDBServiceProvider;

use crate::{
    error::{LandscapeApiError, LandscapeResult},
    SimpleResult,
};

pub async fn get_dst_ip_rule_config_paths(store: LandscapeDBServiceProvider) -> Router {
    let share_state = DstIpRuleService::new(store).await;

    Router::new()
        .route("/dst_ip_rules", get(get_dst_ip_rules).post(add_dst_ip_rules))
        .route("/dst_ip_rules/:id", get(get_dst_ip_rule).delete(del_dst_ip_rule))
        .route("/dst_ip_rules/flow/:flow_id", get(get_flow_dst_ip_rules))
        .with_state(share_state)
}

async fn get_dst_ip_rules(State(state): State<DstIpRuleService>) -> Json<Vec<WanIpRuleConfig>> {
    let result = state.list().await;
    Json(result)
}

async fn get_flow_dst_ip_rules(
    State(state): State<DstIpRuleService>,
    Path(id): Path<FlowId>,
) -> Json<Vec<WanIpRuleConfig>> {
    let result = state.list_flow_configs(id).await;
    Json(result)
}

async fn get_dst_ip_rule(
    State(state): State<DstIpRuleService>,
    Path(id): Path<ConfigId>,
) -> LandscapeResult<Json<WanIpRuleConfig>> {
    let result = state.find_by_id(id).await;
    if let Some(config) = result {
        Ok(Json(config))
    } else {
        Err(LandscapeApiError::NotFound(format!("DstIpRule id: {:?}", id)))
    }
}

async fn add_dst_ip_rules(
    State(state): State<DstIpRuleService>,
    Json(rule): Json<WanIpRuleConfig>,
) -> Json<WanIpRuleConfig> {
    let result = state.set(rule).await;
    Json(result)
}

async fn del_dst_ip_rule(
    State(state): State<DstIpRuleService>,
    Path(id): Path<ConfigId>,
) -> Json<SimpleResult> {
    state.delete(id).await;
    Json(SimpleResult { success: true })
}
