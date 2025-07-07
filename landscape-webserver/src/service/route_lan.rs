use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use landscape_common::service::DefaultWatchServiceStatus;
use landscape_common::{
    config::route_lan::RouteLanServiceConfig, service::controller_service_v2::ControllerService,
};
use serde_json::Value;

use crate::{error::LandscapeApiError, LandscapeApp, SimpleResult};

pub async fn get_route_lan_paths() -> Router<LandscapeApp> {
    Router::new()
        .route("/route_lans/status", get(get_all_route_lan_status))
        .route("/route_lans", post(handle_route_lan_status))
        .route(
            "/route_lans/:iface_name",
            get(get_route_lan_conifg).delete(delete_and_stop_route_lan),
        )
}

async fn get_all_route_lan_status(State(state): State<LandscapeApp>) -> Json<Value> {
    let result = serde_json::to_value(state.route_lan_service.get_all_status().await);
    Json(result.unwrap())
}

async fn get_route_lan_conifg(
    State(state): State<LandscapeApp>,
    Path(iface_name): Path<String>,
) -> Result<Json<RouteLanServiceConfig>, LandscapeApiError> {
    if let Some(iface_config) = state.route_lan_service.get_config_by_name(iface_name).await {
        Ok(Json(iface_config))
    } else {
        Err(LandscapeApiError::NotFound("can not find".into()))
    }
}

async fn handle_route_lan_status(
    State(state): State<LandscapeApp>,
    Json(config): Json<RouteLanServiceConfig>,
) -> Json<SimpleResult> {
    let result = SimpleResult { success: true };
    state.route_lan_service.handle_service_config(config).await;
    Json(result)
}

async fn delete_and_stop_route_lan(
    State(state): State<LandscapeApp>,
    Path(iface_name): Path<String>,
) -> Json<Option<DefaultWatchServiceStatus>> {
    Json(state.route_lan_service.delete_and_stop_iface_service(iface_name).await)
}
