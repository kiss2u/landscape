use std::collections::HashMap;

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use landscape_common::service::controller_service_v2::ControllerService;
use landscape_common::{
    config::dhcp_v6_client::IPV6PDServiceConfig, service::DefaultWatchServiceStatus,
};

use crate::{api::LandscapeApiResp, error::LandscapeApiResult};
use crate::{error::LandscapeApiError, LandscapeApp};

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

async fn get_all_status(
    State(state): State<LandscapeApp>,
) -> LandscapeApiResult<HashMap<String, DefaultWatchServiceStatus>> {
    LandscapeApiResp::success(state.ipv6_pd_service.get_all_status().await)
}

async fn get_iface_pd_conifg(
    State(state): State<LandscapeApp>,
    Path(iface_name): Path<String>,
) -> LandscapeApiResult<IPV6PDServiceConfig> {
    if let Some(iface_config) = state.ipv6_pd_service.get_config_by_name(iface_name).await {
        LandscapeApiResp::success(iface_config)
    } else {
        Err(LandscapeApiError::NotFound("can not find".into()))
    }
}

/// 处理新建 IPv6 PD 获取配置
async fn handle_iface_pd(
    State(state): State<LandscapeApp>,
    Json(config): Json<IPV6PDServiceConfig>,
) -> LandscapeApiResult<()> {
    state.ipv6_pd_service.handle_service_config(config).await;
    LandscapeApiResp::success(())
}

async fn delete_and_stop_iface_service(
    State(state): State<LandscapeApp>,
    Path(iface_name): Path<String>,
) -> LandscapeApiResult<Option<DefaultWatchServiceStatus>> {
    LandscapeApiResp::success(state.ipv6_pd_service.delete_and_stop_iface_service(iface_name).await)
}
