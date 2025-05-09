use axum::{extract::State, routing::get, Json, Router};
use landscape::metric::MetricService;
use landscape_common::metric::firewall::FrontEndFirewallMetricServiceData;
use serde_json::Value;

pub async fn get_metric_service_paths() -> Router {
    let metric_service = MetricService::new().await;

    metric_service.start_service().await;
    Router::new()
        .route("/status", get(get_metric_status))
        .route("/firewall", get(get_firewall_metric_info))
        .with_state(metric_service)
}

pub async fn get_metric_status(State(state): State<MetricService>) -> Json<Value> {
    Json(serde_json::to_value(&state.status).unwrap())
}

pub async fn get_firewall_metric_info(
    State(state): State<MetricService>,
) -> Json<FrontEndFirewallMetricServiceData> {
    let data = state.data.firewall.convert_to_front_formart().await;
    Json(data)
}
