use std::{borrow::Borrow, path::PathBuf, sync::Arc};

use axum::{
    extract::{Path, State},
    routing::{delete, get, post},
    Json, Router,
};
use ipconfig::get_iface_service_paths;
use landscape::store::StoreFileManager;
use nat::get_iface_nat_paths;
use packet_mark::get_iface_packet_mark_paths;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::Mutex;

use landscape_dns::{rule::DNSRuleConfig, LandscapeDnsService};

mod ipconfig;
mod nat;
mod packet_mark;

#[derive(Clone)]
struct LandscapeServices {
    dns_service: LandscapeDnsService,
    store: Arc<Mutex<StoreFileManager>>,
}

pub async fn get_service_paths(home_path: PathBuf) -> Router {
    let mut store = StoreFileManager::new(home_path.clone(), "dns_rule".to_string());
    let rules = store.list().unwrap();
    if rules.is_empty() {
        let config = DNSRuleConfig::default();
        store.set(config.get_store_key(), serde_json::to_string(&config).unwrap());
    }
    let share_state = LandscapeServices {
        dns_service: LandscapeDnsService::new(home_path.clone()).await,
        store: Arc::new(Mutex::new(store)),
    };
    Router::new()
        .route("/dns", get(get_dns_service_status).post(start_dns_service).delete(stop_dns_service))
        .route("/dns/rules", get(get_dns_rules).post(add_dns_rules))
        .route("/dns/rules/:index", delete(del_dns_rules))
        .with_state(share_state)
        .merge(get_iface_service_paths(home_path.clone()).await)
        .merge(get_iface_packet_mark_paths(home_path.clone()).await)
        .merge(get_iface_nat_paths(home_path).await)
}
async fn get_dns_rules(State(state): State<LandscapeServices>) -> Json<Value> {
    let mut get_store = state.store.lock().await;
    let dns_rules: Vec<DNSRuleConfig> = if let Some(datas) = get_store.list() {
        datas.into_iter().filter_map(|j| serde_json::from_str(&j).ok()).collect()
    } else {
        vec![]
    };
    let result = serde_json::to_value(dns_rules);
    Json(result.unwrap())
}

async fn add_dns_rules(
    State(state): State<LandscapeServices>,
    Json(dns_rule): Json<DNSRuleConfig>,
) -> Json<Value> {
    let mut get_store = state.store.lock().await;
    get_store.set(dns_rule.get_store_key(), serde_json::to_string(&dns_rule).unwrap());
    let dns_rules: Vec<DNSRuleConfig> = if let Some(datas) = get_store.list() {
        datas.into_iter().filter_map(|j| serde_json::from_str(&j).ok()).collect()
    } else {
        vec![]
    };
    let result = serde_json::to_value(dns_rules);
    Json(result.unwrap())
}

async fn del_dns_rules(
    State(state): State<LandscapeServices>,
    Path(index): Path<String>,
) -> Json<Value> {
    let mut get_store = state.store.lock().await;
    get_store.del(index);
    let dns_rules: Vec<DNSRuleConfig> = if let Some(datas) = get_store.list() {
        datas.into_iter().filter_map(|j| serde_json::from_str(&j).ok()).collect()
    } else {
        vec![]
    };
    let result = serde_json::to_value(dns_rules);
    Json(result.unwrap())
}

async fn get_dns_service_status(State(state): State<LandscapeServices>) -> Json<Value> {
    // let dns_lock = state.dns_service.lock().await;
    // let dns_data = dns_lock.clone();
    // drop(dns_lock);

    let result = serde_json::to_value(state.dns_service);
    Json(result.unwrap())
}

async fn start_dns_service(
    State(state): State<LandscapeServices>,
    Json(DNSStartRequest { udp_port, tcp_port }): Json<DNSStartRequest>,
) -> Json<Value> {
    let mut get_store = state.store.lock().await;
    let dns_rules: Vec<DNSRuleConfig> = if let Some(datas) = get_store.list() {
        datas.into_iter().filter_map(|j| serde_json::from_str(&j).ok()).collect()
    } else {
        vec![]
    };

    drop(get_store);
    // let dns_lock = state.dns_service.lock().await;
    state.dns_service.start(udp_port, tcp_port, dns_rules).await;
    // let dns_data = dns_lock.clone();
    // drop(dns_lock);
    let result = serde_json::to_value(state.dns_service);
    Json(result.unwrap())
}

async fn stop_dns_service(State(state): State<LandscapeServices>) -> Json<Value> {
    // let dns_lock = state.dns_service.lock().await;
    state.dns_service.stop();
    // let dns_data = dns_lock.clone();
    // drop(dns_lock);
    let result = serde_json::to_value(state.dns_service);
    Json(result.unwrap())
}

#[derive(Clone, Serialize, Deserialize)]
struct DNSStartRequest {
    udp_port: u16,
    tcp_port: Option<u16>,
}
