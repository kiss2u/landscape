use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};

use landscape::wifi::WifiServiceManagerService;
use landscape_common::service::controller_service::ControllerService;
use landscape_common::{config::wifi::WifiServiceConfig, service::DefaultWatchServiceStatus};

use landscape_database::provider::LandscapeDBServiceProvider;
use serde_json::Value;

use crate::{error::LandscapeApiError, SimpleResult};

pub async fn get_wifi_service_paths(store: LandscapeDBServiceProvider) -> Router {
    let share_state = WifiServiceManagerService::new(store).await;

    Router::new()
        .route("/wifi/status", get(get_all_iface_service_status))
        .route("/wifi", post(handle_service_config))
        .route(
            "/wifi/:iface_name",
            get(get_iface_service_conifg).delete(delete_and_stop_iface_service),
        )
        .with_state(share_state)
}

async fn get_all_iface_service_status(
    State(state): State<WifiServiceManagerService>,
) -> Json<Value> {
    let result = serde_json::to_value(state.get_all_status().await);
    Json(result.unwrap())
}

async fn get_iface_service_conifg(
    State(state): State<WifiServiceManagerService>,
    Path(iface_name): Path<String>,
) -> Result<Json<WifiServiceConfig>, LandscapeApiError> {
    if let Some(iface_config) = state.get_config_by_name(iface_name).await {
        Ok(Json(iface_config))
    } else {
        Err(LandscapeApiError::NotFound("can not find".into()))
    }
}

async fn handle_service_config(
    State(state): State<WifiServiceManagerService>,
    Json(config): Json<WifiServiceConfig>,
) -> Json<SimpleResult> {
    let result = SimpleResult { success: true };
    state.handle_service_config(config).await;
    Json(result)
}

async fn delete_and_stop_iface_service(
    State(state): State<WifiServiceManagerService>,
    Path(iface_name): Path<String>,
) -> Json<Option<DefaultWatchServiceStatus>> {
    Json(state.delete_and_stop_iface_service(iface_name).await)
}
