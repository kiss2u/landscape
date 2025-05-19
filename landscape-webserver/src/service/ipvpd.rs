use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use landscape::service::ipv6pd::DHCPv6ClientManagerService;
use landscape_common::service::controller_service::ControllerService;
use landscape_common::{
    config::dhcp_v6_client::IPV6PDServiceConfig, observer::IfaceObserverAction,
    service::DefaultWatchServiceStatus,
};
use landscape_database::provider::LandscapeDBServiceProvider;
use serde_json::Value;
use tokio::sync::broadcast;

use crate::{error::LandscapeApiError, SimpleResult};

pub async fn get_iface_pdclient_paths(
    store: LandscapeDBServiceProvider,
    dev_observer: broadcast::Receiver<IfaceObserverAction>,
) -> Router {
    let share_state = DHCPv6ClientManagerService::new(store, dev_observer).await;

    Router::new()
        .route("/ipv6pd/status", get(get_all_status))
        .route("/ipv6pd", post(handle_iface_pd))
        .route(
            "/ipv6pd/:iface_name",
            get(get_iface_pd_conifg).delete(delete_and_stop_iface_service),
        )
        // .route("/nats/:iface_name/restart", post(restart_nat_service_status))
        .with_state(share_state)
}

async fn get_all_status(State(state): State<DHCPv6ClientManagerService>) -> Json<Value> {
    let result = serde_json::to_value(state.get_all_status().await);
    Json(result.unwrap())
}

async fn get_iface_pd_conifg(
    State(state): State<DHCPv6ClientManagerService>,
    Path(iface_name): Path<String>,
) -> Result<Json<IPV6PDServiceConfig>, LandscapeApiError> {
    if let Some(iface_config) = state.get_config_by_name(iface_name).await {
        Ok(Json(iface_config))
    } else {
        Err(LandscapeApiError::NotFound("can not find".into()))
    }
}

/// 处理新建 IPv6 PD 获取配置
async fn handle_iface_pd(
    State(state): State<DHCPv6ClientManagerService>,
    Json(config): Json<IPV6PDServiceConfig>,
) -> Json<SimpleResult> {
    let result = SimpleResult { success: true };
    state.handle_service_config(config).await;
    Json(result)
}

async fn delete_and_stop_iface_service(
    State(state): State<DHCPv6ClientManagerService>,
    Path(iface_name): Path<String>,
) -> Json<Option<DefaultWatchServiceStatus>> {
    Json(state.delete_and_stop_iface_service(iface_name).await)
}
