use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};

use landscape::service::dhcp_v4::DHCPv4ServerManagerService;
use landscape_common::service::controller_service_v2::ControllerService;
use landscape_common::{
    config::dhcp_v4_server::DHCPv4ServiceConfig, observer::IfaceObserverAction,
    service::dhcp::DHCPv4ServiceWatchStatus,
};

use landscape_database::provider::LandscapeDBServiceProvider;
use serde_json::Value;
use tokio::sync::broadcast;

use crate::{error::LandscapeApiError, SimpleResult};

pub async fn get_dhcp_v4_service_paths(
    store: LandscapeDBServiceProvider,
    dev_observer: broadcast::Receiver<IfaceObserverAction>,
) -> Router {
    let share_state = DHCPv4ServerManagerService::new(store, dev_observer).await;

    Router::new()
        .route("/dhcp_v4/status", get(get_all_iface_service_status))
        .route("/dhcp_v4", post(handle_service_config))
        .route(
            "/dhcp_v4/:iface_name",
            get(get_iface_service_conifg).delete(delete_and_stop_iface_service),
        )
        // .route("/dhcp_v4/:iface_name/restart", post(restart_mark_service_status))
        .with_state(share_state)
}

async fn get_all_iface_service_status(
    State(state): State<DHCPv4ServerManagerService>,
) -> Json<Value> {
    let result = serde_json::to_value(state.get_all_status().await);
    Json(result.unwrap())
}

async fn get_iface_service_conifg(
    State(state): State<DHCPv4ServerManagerService>,
    Path(iface_name): Path<String>,
) -> Result<Json<DHCPv4ServiceConfig>, LandscapeApiError> {
    if let Some(iface_config) = state.get_config_by_name(iface_name).await {
        Ok(Json(iface_config))
    } else {
        Err(LandscapeApiError::NotFound("can not find".into()))
    }
}

async fn handle_service_config(
    State(state): State<DHCPv4ServerManagerService>,
    Json(config): Json<DHCPv4ServiceConfig>,
) -> Result<Json<SimpleResult>, LandscapeApiError> {
    if config.enable {
        if let Err(conflict_msg) = state.check_ip_range_conflict(&config).await {
            return Err(LandscapeApiError::DHCPConflict(conflict_msg));
        }
    }

    state.handle_service_config(config).await;

    let result = SimpleResult { success: true };
    Ok(Json(result))
}

async fn delete_and_stop_iface_service(
    State(state): State<DHCPv4ServerManagerService>,
    Path(iface_name): Path<String>,
) -> Json<Option<DHCPv4ServiceWatchStatus>> {
    Json(state.delete_and_stop_iface_service(iface_name).await)
}
