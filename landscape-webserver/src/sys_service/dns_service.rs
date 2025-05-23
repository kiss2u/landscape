use axum::{extract::State, routing::get, Json, Router};
use landscape_common::service::DefaultWatchServiceStatus;

use crate::LandscapeApp;

pub async fn get_dns_paths() -> Router<LandscapeApp> {
    Router::new()
        .route("/dns", get(get_dns_service_status).post(start_dns_service).delete(stop_dns_service))
}

async fn get_dns_service_status(
    State(state): State<LandscapeApp>,
) -> Json<DefaultWatchServiceStatus> {
    Json(state.dns_service.get_status().await)
}

async fn start_dns_service(State(state): State<LandscapeApp>) {
    state.dns_service.start_dns_service().await;
}

async fn stop_dns_service(State(state): State<LandscapeApp>) {
    state.dns_service.stop().await;
}
