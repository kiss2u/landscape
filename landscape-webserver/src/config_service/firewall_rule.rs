use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use landscape_common::service::controller_service::ConfigController;
use landscape_common::{config::ConfigId, firewall::FirewallRuleConfig};

use crate::{
    error::{LandscapeApiError, LandscapeResult},
    LandscapeApp, SimpleResult,
};

pub async fn get_firewall_rule_config_paths() -> Router<LandscapeApp> {
    Router::new()
        .route("/firewall_rules", get(get_firewall_rules).post(add_firewall_rule))
        .route("/firewall_rules/:id", get(get_firewall_rule).delete(del_firewall_rule))
}

async fn get_firewall_rules(State(state): State<LandscapeApp>) -> Json<Vec<FirewallRuleConfig>> {
    let result = state.fire_wall_rule_service.list().await;
    Json(result)
}

async fn get_firewall_rule(
    State(state): State<LandscapeApp>,
    Path(id): Path<ConfigId>,
) -> LandscapeResult<Json<FirewallRuleConfig>> {
    let result = state.fire_wall_rule_service.find_by_id(id).await;
    if let Some(config) = result {
        Ok(Json(config))
    } else {
        Err(LandscapeApiError::NotFound(format!("Firewall Rule id: {:?}", id)))
    }
}

async fn add_firewall_rule(
    State(state): State<LandscapeApp>,
    Json(firewall_rule): Json<FirewallRuleConfig>,
) -> Json<FirewallRuleConfig> {
    let result = state.fire_wall_rule_service.set(firewall_rule).await;
    Json(result)
}

async fn del_firewall_rule(
    State(state): State<LandscapeApp>,
    Path(id): Path<ConfigId>,
) -> Json<SimpleResult> {
    state.fire_wall_rule_service.delete(id).await;
    Json(SimpleResult { success: true })
}
