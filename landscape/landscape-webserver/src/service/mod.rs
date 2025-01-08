use std::{path::PathBuf, sync::Arc};

use axum::{
    extract::{Path, State},
    routing::{delete, get},
    Json, Router,
};
use ipconfig::get_iface_service_paths;
use landscape_common::store::storev2::StoreFileManager;
use nat::get_iface_nat_paths;
use packet_mark::get_iface_packet_mark_paths;
use pppd::get_iface_pppd_paths;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::Mutex;

use landscape_dns::{rule::DNSRuleConfig, LandscapeDnsService};

mod ipconfig;
mod nat;
mod packet_mark;
mod pppd;

#[derive(Clone)]
struct LandscapeServices {
    dns_service: LandscapeDnsService,
    store: Arc<Mutex<StoreFileManager<DNSRuleConfig>>>,
}

pub async fn get_service_paths(home_path: PathBuf) -> Router {
    let mut store = StoreFileManager::new(home_path.clone(), "dns_rule".to_string());
    let mut rules = store.list();
    if rules.is_empty() {
        store.set(DNSRuleConfig::default());
        rules = store.list();
    }
    let share_state = LandscapeServices {
        dns_service: LandscapeDnsService::new(home_path.clone()).await,
        store: Arc::new(Mutex::new(store)),
    };
    share_state.dns_service.start(53, Some(53), rules).await;

    Router::new()
        .route("/dns", get(get_dns_service_status).post(start_dns_service).delete(stop_dns_service))
        .route("/dns/rules", get(get_dns_rules).post(add_dns_rules))
        .route("/dns/rules/:index", delete(del_dns_rules))
        .with_state(share_state)
        .merge(get_iface_pppd_paths(home_path.clone()).await)
        .merge(get_iface_service_paths(home_path.clone()).await)
        .merge(get_iface_packet_mark_paths(home_path.clone()).await)
        .merge(get_iface_nat_paths(home_path).await)
}

async fn get_dns_rules(State(state): State<LandscapeServices>) -> Json<Value> {
    let mut get_store = state.store.lock().await;
    let dns_rules = get_store.list();
    let result = serde_json::to_value(dns_rules);
    Json(result.unwrap())
}

async fn add_dns_rules(
    State(state): State<LandscapeServices>,
    Json(dns_rule): Json<DNSRuleConfig>,
) -> Json<Value> {
    let mut get_store = state.store.lock().await;
    get_store.set(dns_rule);
    let dns_rules = get_store.list();
    let result = serde_json::to_value(dns_rules);
    Json(result.unwrap())
}

async fn del_dns_rules(
    State(state): State<LandscapeServices>,
    Path(index): Path<String>,
) -> Json<Value> {
    let mut get_store = state.store.lock().await;
    get_store.del(&index);
    let dns_rules = get_store.list();
    let result = serde_json::to_value(dns_rules);
    Json(result.unwrap())
}

async fn get_dns_service_status(State(state): State<LandscapeServices>) -> Json<Value> {
    let result = serde_json::to_value(state.dns_service);
    Json(result.unwrap())
}

async fn start_dns_service(
    State(state): State<LandscapeServices>,
    Json(DNSStartRequest { udp_port, tcp_port }): Json<DNSStartRequest>,
) -> Json<Value> {
    let mut get_store = state.store.lock().await;
    let dns_rules = get_store.list();
    drop(get_store);

    state.dns_service.start(udp_port, tcp_port, dns_rules).await;
    let result = serde_json::to_value(state.dns_service);
    Json(result.unwrap())
}

async fn stop_dns_service(State(state): State<LandscapeServices>) -> Json<Value> {
    state.dns_service.stop();
    let result = serde_json::to_value(state.dns_service);
    Json(result.unwrap())
}

#[derive(Clone, Serialize, Deserialize)]
struct DNSStartRequest {
    udp_port: u16,
    tcp_port: Option<u16>,
}
