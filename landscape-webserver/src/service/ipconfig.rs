use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use landscape_common::service::controller_service_v2::ControllerService;
use serde_json::Value;

use landscape_common::{
    config::iface_ip::IfaceIpServiceConfig, service::DefaultWatchServiceStatus,
};

use crate::{error::LandscapeApiError, LandscapeApp, SimpleResult};

pub async fn get_iface_ipconfig_paths() -> Router<LandscapeApp> {
    Router::new()
        .route("/ipconfigs/status", get(get_all_ipconfig_status))
        .route("/ipconfigs", post(handle_iface_service_status))
        .route(
            "/ipconfigs/:iface_name",
            get(get_iface_service_conifg).delete(delete_and_stop_iface_service),
        )
    // .route("/ipconfigs/:iface_name/status", get(get_iface_service_status))
}

async fn get_all_ipconfig_status(State(state): State<LandscapeApp>) -> Json<Value> {
    let result = serde_json::to_value(state.wan_ip_service.get_all_status().await);
    Json(result.unwrap())
}

async fn get_iface_service_conifg(
    State(state): State<LandscapeApp>,
    Path(iface_name): Path<String>,
) -> Result<Json<IfaceIpServiceConfig>, LandscapeApiError> {
    if let Some(iface_config) = state.wan_ip_service.get_config_by_name(iface_name).await {
        Ok(Json(iface_config))
    } else {
        Err(LandscapeApiError::NotFound("can not find".into()))
    }
}

// async fn get_iface_service_status(
//     State(state): State<LandscapeApp>,
//     Path(iface_name): Path<String>,
// ) -> Json<Value> {
//     let read_lock = state.wan_ip_service.service.services.read().await;
//     let data = if let Some((iface_status, _)) = read_lock.get(&iface_name) {
//         iface_status.clone()
//     } else {
//         DefaultWatchServiceStatus::new()
//     };
//     let result = serde_json::to_value(data);
//     Json(result.unwrap())
// }

async fn handle_iface_service_status(
    State(state): State<LandscapeApp>,
    Json(config): Json<IfaceIpServiceConfig>,
) -> Json<SimpleResult> {
    let result = SimpleResult { success: true };
    state.wan_ip_service.handle_service_config(config).await;
    Json(result)
}

async fn delete_and_stop_iface_service(
    State(state): State<LandscapeApp>,
    Path(iface_name): Path<String>,
) -> Json<Option<DefaultWatchServiceStatus>> {
    Json(state.wan_ip_service.delete_and_stop_iface_service(iface_name).await)
}
