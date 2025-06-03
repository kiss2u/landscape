use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use landscape_common::service::controller_service::ControllerService;
use landscape_database::provider::LandscapeDBServiceProvider;
use serde_json::Value;
use tokio::sync::broadcast;

use landscape::service::ipconfig::IfaceIpServiceManagerService;
use landscape_common::{
    config::iface_ip::IfaceIpServiceConfig, observer::IfaceObserverAction,
    service::DefaultWatchServiceStatus,
};

use crate::{error::LandscapeApiError, SimpleResult};

pub async fn get_iface_ipconfig_paths(
    store: LandscapeDBServiceProvider,
    dev_observer: broadcast::Receiver<IfaceObserverAction>,
) -> Router {
    let share_state = IfaceIpServiceManagerService::new(store, dev_observer).await;

    Router::new()
        .route("/ipconfigs/status", get(get_all_ipconfig_status))
        .route("/ipconfigs", post(handle_iface_service_status))
        .route(
            "/ipconfigs/:iface_name",
            get(get_iface_service_conifg).delete(delete_and_stop_iface_service),
        )
        // .route("/ipconfigs/:iface_name/status", get(get_iface_service_status))
        .with_state(share_state)
}

async fn get_all_ipconfig_status(State(state): State<IfaceIpServiceManagerService>) -> Json<Value> {
    let result = serde_json::to_value(state.get_all_status().await);
    Json(result.unwrap())
}

async fn get_iface_service_conifg(
    State(state): State<IfaceIpServiceManagerService>,
    Path(iface_name): Path<String>,
) -> Result<Json<IfaceIpServiceConfig>, LandscapeApiError> {
    if let Some(iface_config) = state.get_config_by_name(iface_name).await {
        Ok(Json(iface_config))
    } else {
        Err(LandscapeApiError::NotFound("can not find".into()))
    }
}

// async fn get_iface_service_status(
//     State(state): State<IfaceIpServiceManagerService>,
//     Path(iface_name): Path<String>,
// ) -> Json<Value> {
//     let read_lock = state.service.services.read().await;
//     let data = if let Some((iface_status, _)) = read_lock.get(&iface_name) {
//         iface_status.clone()
//     } else {
//         DefaultWatchServiceStatus::new()
//     };
//     let result = serde_json::to_value(data);
//     Json(result.unwrap())
// }

async fn handle_iface_service_status(
    State(state): State<IfaceIpServiceManagerService>,
    Json(config): Json<IfaceIpServiceConfig>,
) -> Json<SimpleResult> {
    let result = SimpleResult { success: true };
    state.handle_service_config(config).await;
    Json(result)
}

async fn delete_and_stop_iface_service(
    State(state): State<IfaceIpServiceManagerService>,
    Path(iface_name): Path<String>,
) -> Json<Option<DefaultWatchServiceStatus>> {
    Json(state.delete_and_stop_iface_service(iface_name).await)
}
