use std::{collections::HashMap, fs::OpenOptions, io::Write, path::PathBuf, sync::Arc};

use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use landscape::service::{
    pppd_service::{PPPDServiceConfig, PPPDServiceManager},
    WatchServiceStatus,
};
use landscape_common::args::LAND_ARGS;
use landscape_common::store::storev2::StoreFileManager;
use serde_json::Value;
use tokio::sync::Mutex;

use crate::{error::LandscapeApiError, SimpleResult};

#[derive(Clone)]
struct LandscapeIfacePPPDServices {
    service: PPPDServiceManager,
    store: Arc<Mutex<StoreFileManager<PPPDServiceConfig>>>,
}
// 755
const PPPD_IF_UP_FILE_PATH: &str = "/etc/ppp/ip-up.d/0land_pppd_up";
const PPPD_IF_DOWN_FILE_PATH: &str = "/etc/ppp/ip-down.d/0land_pppd_down";

pub fn write_context(path: PathBuf, context: &str) -> Result<(), ()> {
    let Ok(mut file) = OpenOptions::new().write(true).truncate(true).create(true).open(&path)
    else {
        return Err(());
    };

    let Ok(_) = file.write_all(context.as_bytes()) else {
        return Err(());
    };

    // 设置文件权限为 755
    let permissions = <std::fs::Permissions as std::os::unix::fs::PermissionsExt>::from_mode(0o755); // 755 权限
    if let Err(e) = std::fs::set_permissions(path, permissions) {
        println!("set permissions error: {:?}", e);
    };

    Ok(())
}

pub fn check_pppd_sh() {
    let args = LAND_ARGS.clone();
    let server_address = format!("http://127.0.0.1:{}", args.port);
    let ip_up_file_context = format!(
        r#"#!/bin/sh
# Enable MSS clamping (autogenerated by landscape)

iptables -t mangle -o "$PPP_IFACE" --insert FORWARD 1 -p tcp --tcp-flags SYN,RST SYN -m tcpmss --mss 1400:65495 -j TCPMSS --clamp-mss-to-pmtu
curl -X POST {server_address}/api/services/nats/"$PPP_IFACE"/restart
curl -X POST {server_address}/api/services/packet_marks/"$PPP_IFACE"/restart
"#
    );
    let _ = write_context(PathBuf::from(PPPD_IF_UP_FILE_PATH), &ip_up_file_context);
    let ip_down_file_context = r#"#!/bin/sh
# Disable MSS clamping (autogenerated by landscape)

iptables -t mangle -L -n -v --line-numbers | grep "TCPMSS.*$PPP_IFACE.*clamp" | cut -f1 -d " " | sort -r | xargs -n1 -r iptables -t mangle -D FORWARD
"#;

    let _ = write_context(PathBuf::from(PPPD_IF_DOWN_FILE_PATH), ip_down_file_context);
}

pub async fn get_iface_pppd_paths(home_path: PathBuf) -> Router {
    check_pppd_sh();
    let mut store = StoreFileManager::new(home_path.clone(), "iface_pppd_service".to_string());

    let share_state = LandscapeIfacePPPDServices {
        service: PPPDServiceManager::init(store.list()).await,
        store: Arc::new(Mutex::new(store)),
    };
    Router::new()
        .route("/pppds/status", get(get_all_pppd_status))
        .route(
            "/pppds/attach/:iface_name",
            get(get_iface_pppd_conifg_by_attach_iface_name)
                .delete(delete_and_stop_iface_pppd_by_attach_iface_name),
        )
        .route(
            "/pppds/:iface_name",
            get(get_iface_pppd_conifg)
                .post(handle_iface_pppd_status)
                .delete(delete_and_stop_iface_pppd),
        )
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
    Path(iface_name): Path<String>,
    Json(service_config): Json<PPPDServiceConfig>,
) -> Json<Value> {
    let result = SimpleResult { success: true };

    // TODO 调用 IfaceIpModelConfig 的 check_iface_status 检查当前的 iface 是否能切换这个状态
    if let Ok(()) = state.service.start_new_service(service_config.clone()).await {
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
            WatchServiceStatus::default()
        };
        data.stop().await;
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
        WatchServiceStatus::default()
    };
    drop(write_lock);
    // 停止服务
    data.stop().await;
    let result = serde_json::to_value(data);
    Json(result.unwrap())
}
