use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use landscape::config_service::dns_rule::DNSConfigService;
use landscape_common::config::{dns::DNSRuleConfig, ConfigId, FlowId};
use landscape_common::service::controller_service::ConfigController;
use landscape_common::service::controller_service::FlowConfigController;
use landscape_database::provider::LandscapeDBServiceProvider;

use crate::{
    error::{LandscapeApiError, LandscapeResult},
    SimpleResult,
};

pub async fn get_dns_rule_config_paths(store: LandscapeDBServiceProvider) -> Router {
    let share_state = DNSConfigService::new(store);

    Router::new()
        .route("/dns_rules", get(get_dns_rules).post(add_dns_rules))
        .route("/dns_rules/:id", get(get_dns_rule).delete(del_dns_rules))
        .route("/dns_rules/flow/:flow_id", get(get_flow_dns_rules))
        .with_state(share_state)
}

async fn get_dns_rules(State(state): State<DNSConfigService>) -> Json<Vec<DNSRuleConfig>> {
    let result = state.list().await;
    Json(result)
}

async fn get_flow_dns_rules(
    State(state): State<DNSConfigService>,
    Path(id): Path<FlowId>,
) -> Json<Vec<DNSRuleConfig>> {
    let result = state.list_flow_configs(id).await;
    Json(result)
}

async fn get_dns_rule(
    State(state): State<DNSConfigService>,
    Path(id): Path<ConfigId>,
) -> LandscapeResult<Json<DNSRuleConfig>> {
    let result = state.find_by_id(id).await;
    if let Some(config) = result {
        Ok(Json(config))
    } else {
        Err(LandscapeApiError::NotFound(format!("Dns Rule id: {:?}", id)))
    }
}

async fn add_dns_rules(
    State(state): State<DNSConfigService>,
    Json(dns_rule): Json<DNSRuleConfig>,
) -> Json<DNSRuleConfig> {
    let result = state.set(dns_rule).await;
    Json(result)
}

async fn del_dns_rules(
    State(state): State<DNSConfigService>,
    Path(id): Path<ConfigId>,
) -> Json<SimpleResult> {
    state.delete(id).await;
    Json(SimpleResult { success: true })
}
