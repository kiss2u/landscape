use std::collections::HashMap;

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use landscape_common::service::controller_service::ControllerService;

use landscape::service::mss_clamp::MssClampServiceManagerService;
use landscape_common::{
    config::mss_clamp::MSSClampServiceConfig, observer::IfaceObserverAction,
    service::DefaultWatchServiceStatus,
};

use landscape_database::provider::LandscapeDBServiceProvider;
use tokio::sync::broadcast;

use crate::error::LandscapeApiError;
use crate::{api::LandscapeApiResp, error::LandscapeApiResult};

pub async fn get_mss_clamp_service_paths(
    store: LandscapeDBServiceProvider,
    dev_observer: broadcast::Receiver<IfaceObserverAction>,
) -> Router {
    let share_state = MssClampServiceManagerService::new(store, dev_observer).await;
    Router::new()
        .route("/mss_clamp/status", get(get_all_iface_service_status))
        .route("/mss_clamp", post(handle_service_config))
        .route(
            "/mss_clamp/{iface_name}",
            get(get_iface_service_conifg).delete(delete_and_stop_iface_service),
        )
        // .route("/mss_clamp/{iface_name}/restart", post(restart_mark_service_status))
        .with_state(share_state)
}

async fn get_all_iface_service_status(
    State(state): State<MssClampServiceManagerService>,
) -> LandscapeApiResult<HashMap<String, DefaultWatchServiceStatus>> {
    LandscapeApiResp::success(state.get_all_status().await)
}

async fn get_iface_service_conifg(
    State(state): State<MssClampServiceManagerService>,
    Path(iface_name): Path<String>,
) -> LandscapeApiResult<MSSClampServiceConfig> {
    if let Some(iface_config) = state.get_config_by_name(iface_name).await {
        LandscapeApiResp::success(iface_config)
    } else {
        Err(LandscapeApiError::NotFound("MSS Clamp Service Config".into()))
    }
}

async fn handle_service_config(
    State(state): State<MssClampServiceManagerService>,
    Json(config): Json<MSSClampServiceConfig>,
) -> LandscapeApiResult<()> {
    state.handle_service_config(config).await;
    LandscapeApiResp::success(())
}

async fn delete_and_stop_iface_service(
    State(state): State<MssClampServiceManagerService>,
    Path(iface_name): Path<String>,
) -> LandscapeApiResult<Option<DefaultWatchServiceStatus>> {
    LandscapeApiResp::success(state.delete_and_stop_iface_service(iface_name).await)
}
