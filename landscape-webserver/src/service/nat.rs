use std::collections::HashMap;

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use landscape::service::nat_service::NatServiceManagerService;
use landscape_common::service::controller_service::ControllerService;
use landscape_common::{
    config::nat::NatServiceConfig, observer::IfaceObserverAction,
    service::DefaultWatchServiceStatus,
};
use landscape_database::provider::LandscapeDBServiceProvider;
use tokio::sync::broadcast;

use crate::error::LandscapeApiError;
use crate::{api::LandscapeApiResp, error::LandscapeApiResult};

pub async fn get_iface_nat_paths(
    store: LandscapeDBServiceProvider,
    dev_observer: broadcast::Receiver<IfaceObserverAction>,
) -> Router {
    let share_state = NatServiceManagerService::new(store, dev_observer).await;

    Router::new()
        .route("/nats/status", get(get_all_nat_status))
        .route("/nats", post(handle_iface_nat_status))
        .route("/nats/:iface_name", get(get_iface_nat_conifg).delete(delete_and_stop_iface_nat))
        // .route("/nats/:iface_name/restart", post(restart_nat_service_status))
        .with_state(share_state)
}

async fn get_all_nat_status(
    State(state): State<NatServiceManagerService>,
) -> LandscapeApiResult<HashMap<String, DefaultWatchServiceStatus>> {
    LandscapeApiResp::success(state.get_all_status().await)
}

async fn get_iface_nat_conifg(
    State(state): State<NatServiceManagerService>,
    Path(iface_name): Path<String>,
) -> LandscapeApiResult<NatServiceConfig> {
    if let Some(iface_config) = state.get_config_by_name(iface_name).await {
        LandscapeApiResp::success(iface_config)
    } else {
        Err(LandscapeApiError::NotFound("Nat Service Config".into()))
    }
}

async fn handle_iface_nat_status(
    State(state): State<NatServiceManagerService>,
    Json(config): Json<NatServiceConfig>,
) -> LandscapeApiResult<()> {
    state.handle_service_config(config).await;
    LandscapeApiResp::success(())
}

async fn delete_and_stop_iface_nat(
    State(state): State<NatServiceManagerService>,
    Path(iface_name): Path<String>,
) -> LandscapeApiResult<Option<DefaultWatchServiceStatus>> {
    LandscapeApiResp::success(state.delete_and_stop_iface_service(iface_name).await)
}
