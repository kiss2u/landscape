use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use landscape::service::pppd_service::PPPDServiceConfigManagerService;
use landscape_common::service::controller_service::ControllerService;
use landscape_common::{config::ppp::PPPDServiceConfig, service::DefaultWatchServiceStatus};
use landscape_database::provider::LandscapeDBServiceProvider;
use serde_json::Value;

use crate::{error::LandscapeApiError, SimpleResult};

pub async fn get_iface_pppd_paths(store: LandscapeDBServiceProvider) -> Router {
    let share_state = PPPDServiceConfigManagerService::new(store).await;
    Router::new()
        .route("/pppds/status", get(get_all_pppd_status))
        .route("/pppds", post(handle_iface_pppd_config))
        .route(
            "/pppds/attach/:iface_name",
            get(get_iface_pppd_conifg_by_attach_iface_name)
                .delete(delete_and_stop_iface_pppd_by_attach_iface_name),
        )
        .route("/pppds/:iface_name", get(get_iface_pppd_conifg).delete(delete_and_stop_iface_pppd))
        .with_state(share_state)
}

async fn get_all_pppd_status(State(state): State<PPPDServiceConfigManagerService>) -> Json<Value> {
    let result = serde_json::to_value(state.get_all_status().await);
    Json(result.unwrap())
}

async fn get_iface_pppd_conifg_by_attach_iface_name(
    State(state): State<PPPDServiceConfigManagerService>,
    Path(iface_name): Path<String>,
) -> Result<Json<Vec<PPPDServiceConfig>>, LandscapeApiError> {
    let configs = state.get_pppd_configs_by_attach_iface_name(iface_name).await;

    Ok(Json(configs))
}

async fn get_iface_pppd_conifg(
    State(state): State<PPPDServiceConfigManagerService>,
    Path(iface_name): Path<String>,
) -> Result<Json<PPPDServiceConfig>, LandscapeApiError> {
    if let Some(iface_config) = state.get_config_by_name(iface_name).await {
        Ok(Json(iface_config))
    } else {
        Err(LandscapeApiError::NotFound("can not find".into()))
    }
}

async fn handle_iface_pppd_config(
    State(state): State<PPPDServiceConfigManagerService>,
    Json(config): Json<PPPDServiceConfig>,
) -> Json<SimpleResult> {
    let result = SimpleResult { success: true };
    state.handle_service_config(config).await;
    Json(result)
}

async fn delete_and_stop_iface_pppd_by_attach_iface_name(
    State(state): State<PPPDServiceConfigManagerService>,
    Path(attach_name): Path<String>,
) -> Json<SimpleResult> {
    state.stop_pppds_by_attach_iface_name(attach_name).await;
    Json(SimpleResult { success: true })
}

async fn delete_and_stop_iface_pppd(
    State(state): State<PPPDServiceConfigManagerService>,
    Path(iface_name): Path<String>,
) -> Json<Option<DefaultWatchServiceStatus>> {
    Json(state.delete_and_stop_iface_service(iface_name).await)
}
