use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use landscape_common::service::controller_service_v2::ControllerService;
use landscape_common::{
    config::dhcp_v6_client::IPV6PDServiceConfig, service::DefaultWatchServiceStatus,
};
use serde_json::Value;

use crate::{error::LandscapeApiError, LandscapeApp, SimpleResult};

pub async fn get_iface_pdclient_paths() -> Router<LandscapeApp> {
    Router::new()
        .route("/ipv6pd/status", get(get_all_status))
        .route("/ipv6pd", post(handle_iface_pd))
        .route(
            "/ipv6pd/:iface_name",
            get(get_iface_pd_conifg).delete(delete_and_stop_iface_service),
        )
    // .route("/nats/:iface_name/restart", post(restart_nat_service_status))
}

async fn get_all_status(State(state): State<LandscapeApp>) -> Json<Value> {
    let result = serde_json::to_value(state.ipv6_pd_service.get_all_status().await);
    Json(result.unwrap())
}

async fn get_iface_pd_conifg(
    State(state): State<LandscapeApp>,
    Path(iface_name): Path<String>,
) -> Result<Json<IPV6PDServiceConfig>, LandscapeApiError> {
    if let Some(iface_config) = state.ipv6_pd_service.get_config_by_name(iface_name).await {
        Ok(Json(iface_config))
    } else {
        Err(LandscapeApiError::NotFound("can not find".into()))
    }
}

/// 处理新建 IPv6 PD 获取配置
async fn handle_iface_pd(
    State(state): State<LandscapeApp>,
    Json(config): Json<IPV6PDServiceConfig>,
) -> Json<SimpleResult> {
    let result = SimpleResult { success: true };
    state.ipv6_pd_service.handle_service_config(config).await;
    Json(result)
}

async fn delete_and_stop_iface_service(
    State(state): State<LandscapeApp>,
    Path(iface_name): Path<String>,
) -> Json<Option<DefaultWatchServiceStatus>> {
    Json(state.ipv6_pd_service.delete_and_stop_iface_service(iface_name).await)
}
