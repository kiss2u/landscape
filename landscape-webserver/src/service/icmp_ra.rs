use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use landscape::service::ra::{IPV6RAService, IPV6RAServiceConfig};
use landscape_common::{
    service::{service_manager::ServiceManager, DefaultWatchServiceStatus},
    store::storev2::StoreFileManager,
};
use serde_json::Value;
use tokio::sync::Mutex;

use crate::{error::LandscapeApiError, SimpleResult};

#[derive(Clone)]
struct LandscapeICMPv6RAServices {
    service: ServiceManager<IPV6RAService>,
    store: Arc<Mutex<StoreFileManager<IPV6RAServiceConfig>>>,
}

pub async fn get_iface_icmpv6ra_paths(mut store: StoreFileManager<IPV6RAServiceConfig>) -> Router {
    let share_state = LandscapeICMPv6RAServices {
        service: ServiceManager::init(store.list()).await,
        store: Arc::new(Mutex::new(store)),
    };

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

async fn get_all_status(State(state): State<LandscapeICMPv6RAServices>) -> Json<Value> {
    let read_lock = state.service.services.read().await;
    let mut result = HashMap::new();
    for (key, (iface_status, _)) in read_lock.iter() {
        result.insert(key.clone(), iface_status.clone());
    }
    drop(read_lock);
    let result = serde_json::to_value(result);
    Json(result.unwrap())
}

async fn get_iface_icmpv6_conifg(
    State(state): State<LandscapeICMPv6RAServices>,
    Path(iface_name): Path<String>,
) -> Result<Json<IPV6RAServiceConfig>, LandscapeApiError> {
    let mut read_lock = state.store.lock().await;
    if let Some(iface_config) = read_lock.get(&iface_name) {
        Ok(Json(iface_config))
    } else {
        Err(LandscapeApiError::NotFound("can not find".into()))
    }
}

async fn handle_iface_icmpv6(
    State(state): State<LandscapeICMPv6RAServices>,
    Json(service_config): Json<IPV6RAServiceConfig>,
) -> Json<Value> {
    let result = SimpleResult { success: true };

    if let Ok(()) = state.service.update_service(service_config.clone()).await {
        let mut write_lock = state.store.lock().await;
        write_lock.set(service_config);
        drop(write_lock);
    }
    let result = serde_json::to_value(result);
    Json(result.unwrap())
}

async fn delete_and_stop_iface_icmpv6(
    State(state): State<LandscapeICMPv6RAServices>,
    Path(iface_name): Path<String>,
) -> Json<Value> {
    let mut write_lock = state.store.lock().await;
    write_lock.del(&iface_name);
    drop(write_lock);

    let mut write_lock = state.service.services.write().await;
    let data = if let Some((iface_status, _)) = write_lock.remove(&iface_name) {
        iface_status
    } else {
        DefaultWatchServiceStatus::new()
    };
    drop(write_lock);
    // 停止服务
    data.wait_stop().await;
    let result = serde_json::to_value(data);
    Json(result.unwrap())
}
