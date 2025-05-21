use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use landscape::config_service::flow_rule::FlowRuleService;
use landscape_common::service::controller_service::ConfigController;
use landscape_common::{config::ConfigId, flow::FlowConfig};
use landscape_database::provider::LandscapeDBServiceProvider;

use crate::{
    error::{LandscapeApiError, LandscapeResult},
    SimpleResult,
};

pub async fn get_flow_rule_config_paths(store: LandscapeDBServiceProvider) -> Router {
    let share_state = FlowRuleService::new(store);

    Router::new()
        .route("/flow_rules", get(get_flow_rules).post(add_flow_rule))
        .route("/flow_rules/:id", get(get_flow_rule).delete(del_flow_rule))
        .with_state(share_state)
}

async fn get_flow_rules(State(state): State<FlowRuleService>) -> Json<Vec<FlowConfig>> {
    let result = state.list().await;
    Json(result)
}

async fn get_flow_rule(
    State(state): State<FlowRuleService>,
    Path(id): Path<ConfigId>,
) -> LandscapeResult<Json<FlowConfig>> {
    let result = state.find_by_id(id).await;
    if let Some(config) = result {
        Ok(Json(config))
    } else {
        Err(LandscapeApiError::NotFound(format!("Flow rule id: {:?}", id)))
    }
}

async fn add_flow_rule(
    State(state): State<FlowRuleService>,
    Json(flow_rule): Json<FlowConfig>,
) -> Json<FlowConfig> {
    let result = state.set(flow_rule).await;
    Json(result)
}

async fn del_flow_rule(
    State(state): State<FlowRuleService>,
    Path(id): Path<ConfigId>,
) -> Json<SimpleResult> {
    state.delete(id).await;
    Json(SimpleResult { success: true })
}
