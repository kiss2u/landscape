use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use landscape_common::config::{dns::DNSRuleConfig, ConfigId, FlowId};
use landscape_common::service::controller_service::ConfigController;
use landscape_common::service::controller_service::FlowConfigController;

use crate::{
    error::{LandscapeApiError, LandscapeResult},
    LandscapeApp, SimpleResult,
};

pub async fn get_dns_rule_config_paths() -> Router<LandscapeApp> {
    Router::new()
        .route("/dns_rules", get(get_dns_rules).post(add_dns_rules))
        .route("/dns_rules/set_many", post(add_many_dns_rules))
        .route("/dns_rules/:id", get(get_dns_rule).delete(del_dns_rules))
        .route("/dns_rules/flow/:flow_id", get(get_flow_dns_rules))
}

async fn get_dns_rules(State(state): State<LandscapeApp>) -> Json<Vec<DNSRuleConfig>> {
    let result = state.dns_rule_service.list().await;
    Json(result)
}

async fn get_flow_dns_rules(
    State(state): State<LandscapeApp>,
    Path(id): Path<FlowId>,
) -> Json<Vec<DNSRuleConfig>> {
    let mut result = state.dns_rule_service.list_flow_configs(id).await;
    result.sort_by(|a, b| a.index.cmp(&b.index));
    Json(result)
}

async fn get_dns_rule(
    State(state): State<LandscapeApp>,
    Path(id): Path<ConfigId>,
) -> LandscapeResult<Json<DNSRuleConfig>> {
    let result = state.dns_rule_service.find_by_id(id).await;
    if let Some(config) = result {
        Ok(Json(config))
    } else {
        Err(LandscapeApiError::NotFound(format!("Dns Rule id: {:?}", id)))
    }
}

async fn add_many_dns_rules(
    State(state): State<LandscapeApp>,
    Json(dns_rules): Json<Vec<DNSRuleConfig>>,
) -> Json<SimpleResult> {
    state.dns_rule_service.set_list(dns_rules).await;
    Json(SimpleResult { success: true })
}

async fn add_dns_rules(
    State(state): State<LandscapeApp>,
    Json(dns_rule): Json<DNSRuleConfig>,
) -> Json<DNSRuleConfig> {
    let result = state.dns_rule_service.set(dns_rule).await;
    Json(result)
}

async fn del_dns_rules(
    State(state): State<LandscapeApp>,
    Path(id): Path<ConfigId>,
) -> Json<SimpleResult> {
    state.dns_rule_service.delete(id).await;
    Json(SimpleResult { success: true })
}
