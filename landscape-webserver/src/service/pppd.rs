use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use landscape::service::pppd_service::PPPDService;
use landscape_common::{
    config::ppp::PPPDServiceConfig,
    service::{service_manager::ServiceManager, DefaultWatchServiceStatus},
    store::storev2::StoreFileManager,
};
use serde_json::Value;
use tokio::sync::Mutex;

use crate::{error::LandscapeApiError, SimpleResult};

#[derive(Clone)]
struct LandscapeIfacePPPDServices {
    service: ServiceManager<PPPDService>,
    store: Arc<Mutex<StoreFileManager<PPPDServiceConfig>>>,
}

pub async fn get_iface_pppd_paths(mut store: StoreFileManager<PPPDServiceConfig>) -> Router {
    let share_state = LandscapeIfacePPPDServices {
        service: ServiceManager::init(store.list()).await,
        store: Arc::new(Mutex::new(store)),
    };
    Router::new()
        .route("/pppds/status", get(get_all_pppd_status))
        .route("/pppds", post(handle_iface_pppd_status))
        .route(
            "/pppds/attach/:iface_name",
            get(get_iface_pppd_conifg_by_attach_iface_name)
                .delete(delete_and_stop_iface_pppd_by_attach_iface_name),
        )
        .route("/pppds/:iface_name", get(get_iface_pppd_conifg).delete(delete_and_stop_iface_pppd))
        .with_state(share_state)
}

async fn get_all_pppd_status(State(state): State<LandscapeIfacePPPDServices>) -> Json<Value> {
    let read_lock = state.service.services.read().await;
    let mut result = HashMap::new();
    for (key, (iface_status, _)) in read_lock.iter() {
        result.insert(key.clone(), iface_status.clone());
    }
    drop(read_lock);
    let result = serde_json::to_value(result);
    Json(result.unwrap())
}

async fn get_iface_pppd_conifg_by_attach_iface_name(
    State(state): State<LandscapeIfacePPPDServices>,
    Path(iface_name): Path<String>,
) -> Result<Json<Value>, LandscapeApiError> {
    let mut read_lock = state.store.lock().await;

    let mut stop_iface_name = vec![];
    for each_conf in read_lock.list() {
        if each_conf.attach_iface_name == iface_name {
            stop_iface_name.push(each_conf);
        }
    }
    Ok(Json(serde_json::to_value(stop_iface_name).unwrap()))
}

async fn get_iface_pppd_conifg(
    State(state): State<LandscapeIfacePPPDServices>,
    Path(iface_name): Path<String>,
) -> Result<Json<PPPDServiceConfig>, LandscapeApiError> {
    let mut read_lock = state.store.lock().await;
    if let Some(iface_config) = read_lock.get(&iface_name) {
        Ok(Json(iface_config))
    } else {
        Err(LandscapeApiError::NotFound("can not find".into()))
    }
}

async fn handle_iface_pppd_status(
    State(state): State<LandscapeIfacePPPDServices>,
    Json(service_config): Json<PPPDServiceConfig>,
) -> Json<Value> {
    let result = SimpleResult { success: true };

    // TODO 调用 IfaceIpModelConfig 的 check_iface_status 检查当前的 iface 是否能切换这个状态
    if let Ok(()) = state.service.update_service(service_config.clone()).await {
        let mut write_lock = state.store.lock().await;
        write_lock.set(service_config);
        drop(write_lock);
    }
    let result = serde_json::to_value(result);
    Json(result.unwrap())
}

async fn delete_and_stop_iface_pppd_by_attach_iface_name(
    State(state): State<LandscapeIfacePPPDServices>,
    Path(iface_name): Path<String>,
) -> Json<SimpleResult> {
    let mut write_lock = state.store.lock().await;
    let mut stop_iface_name = vec![];
    for each_conf in write_lock.list() {
        if each_conf.attach_iface_name == iface_name {
            stop_iface_name.push(each_conf.iface_name);
        }
    }

    for name in stop_iface_name.iter() {
        write_lock.del(name);
    }
    drop(write_lock);

    let mut write_lock = state.service.services.write().await;

    for name in stop_iface_name.iter() {
        let data = if let Some((iface_status, _)) = write_lock.remove(name) {
            iface_status
        } else {
            DefaultWatchServiceStatus::new()
        };
        data.wait_stop().await;
    }

    drop(write_lock);
    Json(SimpleResult { success: true })
}

async fn delete_and_stop_iface_pppd(
    State(state): State<LandscapeIfacePPPDServices>,
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
