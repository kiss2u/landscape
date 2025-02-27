use std::sync::Arc;

use axum::{
    extract::{Path, State},
    routing::{delete, get},
    Json, Router,
};
use landscape_common::{dns::DNSRuleConfig, store::storev2::StoreFileManager};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::Mutex;

use landscape_dns::{DNSCache, LandscapeDnsService};

#[derive(Clone)]
struct LandscapeServices {
    dns_service: LandscapeDnsService,
    store: Arc<Mutex<StoreFileManager<DNSRuleConfig>>>,
    prev_cache: Arc<Mutex<Option<Arc<Mutex<DNSCache>>>>>,
}

pub async fn get_dns_paths(mut store: StoreFileManager<DNSRuleConfig>) -> Router {
    let mut rules = store.list();
    if rules.is_empty() {
        store.set(DNSRuleConfig::default());
        rules = store.list();
    }
    let share_state = LandscapeServices {
        dns_service: LandscapeDnsService::new().await,
        store: Arc::new(Mutex::new(store)),
        prev_cache: Arc::new(Mutex::new(None)),
    };

    let cache = share_state.dns_service.start(53, Some(53), rules, None).await;
    {
        let mut lock = share_state.prev_cache.lock().await;
        lock.replace(cache);
    }

    Router::new()
        .route("/", get(get_dns_service_status).post(start_dns_service).delete(stop_dns_service))
        .route("/rules", get(get_dns_rules).post(add_dns_rules))
        .route("/rules/:index", delete(del_dns_rules))
        .with_state(share_state)
}

async fn get_dns_rules(State(state): State<LandscapeServices>) -> Json<Value> {
    let mut get_store = state.store.lock().await;
    let mut dns_rules = get_store.list();
    dns_rules.sort_by(|a, b| a.index.cmp(&b.index));
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

    let old_cache = {
        let mut lock = state.prev_cache.lock().await;
        lock.take()
    };
    let cache = state.dns_service.start(udp_port, tcp_port, dns_rules, old_cache).await;
    {
        let mut lock = state.prev_cache.lock().await;
        lock.replace(cache)
    };
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
