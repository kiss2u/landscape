use std::collections::HashMap;

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};

use landscape::wifi::WifiServiceManagerService;
use landscape_common::service::controller_service::ControllerService;
use landscape_common::{config::wifi::WifiServiceConfig, service::DefaultWatchServiceStatus};

use landscape_database::provider::LandscapeDBServiceProvider;

use crate::error::LandscapeApiError;

use crate::{api::LandscapeApiResp, error::LandscapeApiResult};

pub async fn get_wifi_service_paths(store: LandscapeDBServiceProvider) -> Router {
    let share_state = WifiServiceManagerService::new(store).await;

    Router::new()
        .route("/wifi/status", get(get_all_iface_service_status))
        .route("/wifi", post(handle_service_config))
        .route(
            "/wifi/{iface_name}",
            get(get_iface_service_conifg).delete(delete_and_stop_iface_service),
        )
        .with_state(share_state)
}

async fn get_all_iface_service_status(
    State(state): State<WifiServiceManagerService>,
) -> LandscapeApiResult<HashMap<String, DefaultWatchServiceStatus>> {
    LandscapeApiResp::success(state.get_all_status().await)
}

async fn get_iface_service_conifg(
    State(state): State<WifiServiceManagerService>,
    Path(iface_name): Path<String>,
) -> LandscapeApiResult<WifiServiceConfig> {
    if let Some(iface_config) = state.get_config_by_name(iface_name).await {
        LandscapeApiResp::success(iface_config)
    } else {
        Err(LandscapeApiError::NotFound("Wifi Service Config".into()))
    }
}

async fn handle_service_config(
    State(state): State<WifiServiceManagerService>,
    Json(config): Json<WifiServiceConfig>,
) -> LandscapeApiResult<()> {
    state.handle_service_config(config).await;
    LandscapeApiResp::success(())
}

async fn delete_and_stop_iface_service(
    State(state): State<WifiServiceManagerService>,
    Path(iface_name): Path<String>,
) -> LandscapeApiResult<Option<DefaultWatchServiceStatus>> {
    LandscapeApiResp::success(state.delete_and_stop_iface_service(iface_name).await)
}
