use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use landscape::service::ra::IPV6RAManagerService;
use landscape_common::service::{controller_service::ControllerService, DefaultWatchServiceStatus};
use landscape_common::{config::ra::IPV6RAServiceConfig, observer::IfaceObserverAction};
use landscape_database::provider::LandscapeDBServiceProvider;
use serde_json::Value;
use tokio::sync::broadcast;

use crate::{error::LandscapeApiError, SimpleResult};

pub async fn get_iface_icmpv6ra_paths(
    store: LandscapeDBServiceProvider,
    dev_observer: broadcast::Receiver<IfaceObserverAction>,
) -> Router {
    let share_state = IPV6RAManagerService::new(store, dev_observer).await;

    Router::new()
        .route("/icmpv6ra/status", get(get_all_status))
        .route("/icmpv6ra", post(handle_iface_icmpv6))
        .route(
            "/icmpv6ra/:iface_name",
            get(get_iface_icmpv6_conifg).delete(delete_and_stop_iface_icmpv6),
        )
        // .route("/nats/:iface_name/restart", post(restart_nat_service_status))
        .with_state(share_state)
}

async fn get_all_status(State(state): State<IPV6RAManagerService>) -> Json<Value> {
    let result = serde_json::to_value(state.get_all_status().await);
    Json(result.unwrap())
}

async fn get_iface_icmpv6_conifg(
    State(state): State<IPV6RAManagerService>,
    Path(iface_name): Path<String>,
) -> Result<Json<IPV6RAServiceConfig>, LandscapeApiError> {
    if let Some(iface_config) = state.get_config_by_name(iface_name).await {
        Ok(Json(iface_config))
    } else {
        Err(LandscapeApiError::NotFound("can not find".into()))
    }
}

async fn handle_iface_icmpv6(
    State(state): State<IPV6RAManagerService>,
    Json(config): Json<IPV6RAServiceConfig>,
) -> Json<SimpleResult> {
    let result = SimpleResult { success: true };
    state.handle_service_config(config).await;
    Json(result)
}

async fn delete_and_stop_iface_icmpv6(
    State(state): State<IPV6RAManagerService>,
    Path(iface_name): Path<String>,
) -> Json<Option<DefaultWatchServiceStatus>> {
    Json(state.delete_and_stop_iface_service(iface_name).await)
}
