use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use landscape_common::database::LandscapeDBTrait;
use landscape_common::service::controller_service_v2::ControllerService;
use landscape_common::{config::ppp::PPPDServiceConfig, service::DefaultWatchServiceStatus};

use serde_json::Value;

use crate::{error::LandscapeApiError, LandscapeApp, SimpleResult};

pub async fn get_iface_pppd_paths() -> Router<LandscapeApp> {
    Router::new()
        .route("/pppds", get(get_all_pppd_configs).post(handle_iface_pppd_config))
        .route("/pppds/:iface_name", get(get_iface_pppd_conifg).delete(delete_and_stop_iface_pppd))
        .route("/pppds/status", get(get_all_pppd_status))
        .route(
            "/pppds/attach/:iface_name",
            get(get_iface_pppd_conifg_by_attach_iface_name)
                .delete(delete_and_stop_iface_pppd_by_attach_iface_name),
        )
}

async fn get_all_pppd_configs(State(state): State<LandscapeApp>) -> Json<Vec<PPPDServiceConfig>> {
    Json(state.pppd_service.get_repository().list().await.unwrap_or_default())
}

async fn get_all_pppd_status(State(state): State<LandscapeApp>) -> Json<Value> {
    let result = serde_json::to_value(state.pppd_service.get_all_status().await);
    Json(result.unwrap())
}

async fn get_iface_pppd_conifg_by_attach_iface_name(
    State(state): State<LandscapeApp>,
    Path(iface_name): Path<String>,
) -> Result<Json<Vec<PPPDServiceConfig>>, LandscapeApiError> {
    let configs = state.pppd_service.get_pppd_configs_by_attach_iface_name(iface_name).await;

    Ok(Json(configs))
}

async fn get_iface_pppd_conifg(
    State(state): State<LandscapeApp>,
    Path(iface_name): Path<String>,
) -> Result<Json<PPPDServiceConfig>, LandscapeApiError> {
    if let Some(iface_config) = state.pppd_service.get_config_by_name(iface_name).await {
        Ok(Json(iface_config))
    } else {
        Err(LandscapeApiError::NotFound("can not find".into()))
    }
}

async fn handle_iface_pppd_config(
    State(state): State<LandscapeApp>,
    Json(config): Json<PPPDServiceConfig>,
) -> Json<SimpleResult> {
    let result = SimpleResult { success: true };
    state.pppd_service.handle_service_config(config).await;
    Json(result)
}

async fn delete_and_stop_iface_pppd_by_attach_iface_name(
    State(state): State<LandscapeApp>,
    Path(attach_name): Path<String>,
) -> Json<SimpleResult> {
    state.pppd_service.stop_pppds_by_attach_iface_name(attach_name).await;
    Json(SimpleResult { success: true })
}

async fn delete_and_stop_iface_pppd(
    State(state): State<LandscapeApp>,
    Path(iface_name): Path<String>,
) -> Json<Option<DefaultWatchServiceStatus>> {
    Json(state.pppd_service.delete_and_stop_iface_service(iface_name).await)
}
