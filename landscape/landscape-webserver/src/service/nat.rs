use std::{collections::HashMap, path::PathBuf, sync::Arc};

use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use landscape::{
    service::{
        nat_service::{NatServiceConfig, NatServiceManager},
        WatchServiceStatus,
    },
    store::StoreFileManager,
};
use serde_json::Value;
use tokio::sync::Mutex;

use crate::{error::LandscapeApiError, SimpleResult};

#[derive(Clone)]
struct LandscapeIfaceNatServices {
    service: NatServiceManager,
    store: Arc<Mutex<StoreFileManager>>,
}

pub async fn get_iface_nat_paths(home_path: PathBuf) -> Router {
    let store = StoreFileManager::new(home_path.clone(), "iface_nat_service".to_string());

    let share_state = LandscapeIfaceNatServices {
        service: NatServiceManager::init(vec![]).await,
        store: Arc::new(Mutex::new(store)),
    };
    Router::new()
        .route("/nats/status", get(get_all_nat_status))
        .route(
            "/nats/:iface_name",
            get(get_iface_nat_conifg)
                .post(handle_iface_nat_status)
                .delete(delete_and_stop_iface_nat),
        )
        .with_state(share_state)
}

async fn get_all_nat_status(State(state): State<LandscapeIfaceNatServices>) -> Json<Value> {
    let read_lock = state.service.services.read().await;
    let mut result = HashMap::new();
    for (key, (iface_status, _)) in read_lock.iter() {
        result.insert(key.clone(), iface_status.clone());
    }
    drop(read_lock);
    let result = serde_json::to_value(result);
    Json(result.unwrap())
}

async fn get_iface_nat_conifg(
    State(state): State<LandscapeIfaceNatServices>,
    Path(iface_name): Path<String>,
) -> Result<Json<Value>, LandscapeApiError> {
    let mut read_lock = state.store.lock().await;
    if let Some(iface_config) = read_lock.get(iface_name) {
        let data: Value = serde_json::from_str(&iface_config)?;
        Ok(Json(data))
    } else {
        Err(LandscapeApiError::NotFound("can not find".into()))
    }
}

async fn handle_iface_nat_status(
    State(state): State<LandscapeIfaceNatServices>,
    Path(iface_name): Path<String>,
    Json(service_config): Json<NatServiceConfig>,
) -> Json<Value> {
    let result = SimpleResult { success: true };
    let write_data = serde_json::to_string(&service_config);

    // TODO 调用 IfaceIpModelConfig 的 check_iface_status 检查当前的 iface 是否能切换这个状态
    if let Ok(()) = state.service.start_new_service(service_config).await {
        let mut write_lock = state.store.lock().await;
        write_lock.set(iface_name, write_data.unwrap());
        drop(write_lock);
    }
    let result = serde_json::to_value(result);
    Json(result.unwrap())
}

async fn delete_and_stop_iface_nat(
    State(state): State<LandscapeIfaceNatServices>,
    Path(iface_name): Path<String>,
) -> Json<Value> {
    let mut write_lock = state.store.lock().await;
    write_lock.del(iface_name.clone());
    drop(write_lock);

    let mut write_lock = state.service.services.write().await;
    let data = if let Some((iface_status, _)) = write_lock.remove(&iface_name) {
        iface_status
    } else {
        WatchServiceStatus::default()
    };
    drop(write_lock);
    // 停止服务
    data.stop().await;
    let result = serde_json::to_value(data);
    Json(result.unwrap())
}
