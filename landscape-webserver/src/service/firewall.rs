use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use landscape::firewall::FirewallServiceManagerService;
use landscape_common::service::controller_service::ControllerService;
use landscape_common::{
    config::firewall::FirewallServiceConfig, observer::IfaceObserverAction,
    service::DefaultWatchServiceStatus,
};
use landscape_database::provider::LandscapeDBServiceProvider;

use serde_json::Value;
use tokio::sync::broadcast;

use crate::{error::LandscapeApiError, SimpleResult};

pub async fn get_firewall_service_paths(
    store: LandscapeDBServiceProvider,
    dev_observer: broadcast::Receiver<IfaceObserverAction>,
) -> Router {
    let share_state = FirewallServiceManagerService::new(store, dev_observer).await;

    Router::new()
        .route("/firewall/status", get(get_all_iface_service_status))
        .route("/firewall", post(handle_service_config))
        .route(
            "/firewall/:iface_name",
            get(get_iface_service_conifg).delete(delete_and_stop_iface_service),
        )
        // .route("/firewall/:iface_name/restart", post(restart_mark_service_status))
        .with_state(share_state)
}

async fn get_all_iface_service_status(
    State(state): State<FirewallServiceManagerService>,
) -> Json<Value> {
    let result = serde_json::to_value(state.get_all_status().await);
    Json(result.unwrap())
}

async fn get_iface_service_conifg(
    State(state): State<FirewallServiceManagerService>,
    Path(iface_name): Path<String>,
) -> Result<Json<FirewallServiceConfig>, LandscapeApiError> {
    if let Some(iface_config) = state.get_config_by_name(iface_name).await {
        Ok(Json(iface_config))
    } else {
        Err(LandscapeApiError::NotFound("can not find".into()))
    }
}

async fn handle_service_config(
    State(state): State<FirewallServiceManagerService>,
    Json(config): Json<FirewallServiceConfig>,
) -> Json<SimpleResult> {
    let result = SimpleResult { success: true };
    state.handle_service_config(config).await;
    Json(result)
}

async fn delete_and_stop_iface_service(
    State(state): State<FirewallServiceManagerService>,
    Path(iface_name): Path<String>,
) -> Json<Option<DefaultWatchServiceStatus>> {
    Json(state.delete_and_stop_iface_service(iface_name).await)
}
