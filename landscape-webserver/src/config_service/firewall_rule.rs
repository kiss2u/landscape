use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use landscape::config_service::firewall_rule::FirewallRuleService;
use landscape_common::service::controller_service::ConfigController;
use landscape_common::{config::ConfigId, firewall::FirewallRuleConfig};
use landscape_database::provider::LandscapeDBServiceProvider;

use crate::{
    error::{LandscapeApiError, LandscapeResult},
    SimpleResult,
};

pub async fn get_firewall_rule_config_paths(store: LandscapeDBServiceProvider) -> Router {
    let share_state = FirewallRuleService::new(store).await;

    Router::new()
        .route("/firewall_rules", get(get_firewall_rules).post(add_firewall_rule))
        .route("/firewall_rules/:id", get(get_firewall_rule).delete(del_firewall_rule))
        .with_state(share_state)
}

async fn get_firewall_rules(
    State(state): State<FirewallRuleService>,
) -> Json<Vec<FirewallRuleConfig>> {
    let result = state.list().await;
    Json(result)
}

async fn get_firewall_rule(
    State(state): State<FirewallRuleService>,
    Path(id): Path<ConfigId>,
) -> LandscapeResult<Json<FirewallRuleConfig>> {
    let result = state.find_by_id(id).await;
    if let Some(config) = result {
        Ok(Json(config))
    } else {
        Err(LandscapeApiError::NotFound(format!("Firewall Rule id: {:?}", id)))
    }
}

async fn add_firewall_rule(
    State(state): State<FirewallRuleService>,
    Json(firewall_rule): Json<FirewallRuleConfig>,
) -> Json<FirewallRuleConfig> {
    let result = state.set(firewall_rule).await;
    Json(result)
}

async fn del_firewall_rule(
    State(state): State<FirewallRuleService>,
    Path(id): Path<ConfigId>,
) -> Json<SimpleResult> {
    state.delete(id).await;
    Json(SimpleResult { success: true })
}
