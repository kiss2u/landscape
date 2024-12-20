use std::{collections::HashMap, path::PathBuf, sync::Arc};

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use landscape::{
    iface::{
        config::{IfaceZoneType, NetworkIfaceConfig},
        IfaceTopology,
    },
    store::StoreFileManager,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::Mutex;

use crate::SimpleResult;

#[derive(Clone)]
struct NetworkState {
    store: Arc<Mutex<StoreFileManager>>,
}

pub async fn get_network_paths(home_path: PathBuf) -> Router {
    let mut store = StoreFileManager::new(home_path, "network".to_string());
    let interface_configs = if let Some(datas) = store.list() {
        datas.into_iter().filter_map(|j| serde_json::from_str(&j).ok()).collect()
    } else {
        vec![]
    };
    // 从配置初始化当前网络布局环境
    let nedd_update_config = landscape::init_devs(interface_configs).await;
    for c in nedd_update_config.into_iter() {
        store.set(c.name.clone(), serde_json::to_string(&c).unwrap());
    }
    // println!("==> {:?}", devs);
    let store = Arc::new(Mutex::new(store));
    let share_state = NetworkState { store };
    Router::new()
        .route("/", get(get_ifaces))
        .route("/bridge", post(create_bridge))
        .route("/controller", post(set_controller))
        .route("/zone", post(change_zone))
        .route("/:iface_name/status/:status", post(change_dev_status))
        .with_state(share_state)
}

async fn get_ifaces(State(state): State<NetworkState>) -> Json<Value> {
    let all_alive_devs = landscape::get_all_devices().await;
    let mut store_lock = state.store.lock().await;
    let all_config = store_lock.pair_list();
    drop(store_lock);

    let mut comfig_map: HashMap<String, NetworkIfaceConfig> = HashMap::new();
    for (key, config) in all_config.unwrap().into_iter() {
        comfig_map.insert(key, serde_json::from_str(&config).unwrap());
    }

    let mut info = vec![];
    for each in all_alive_devs.into_iter() {
        if each.is_lo() {
            continue;
        }
        let config = if let Some(config) = comfig_map.remove(&each.name) {
            config
        } else {
            NetworkIfaceConfig::from_phy_dev(&each)
        };
        info.push(IfaceTopology { config, status: each });
    }

    let result = serde_json::to_value(&info);
    Json(result.unwrap())
}

async fn create_bridge(
    State(state): State<NetworkState>,
    Json(bridge_create_request): Json<BridgeCreate>,
) -> Json<SimpleResult> {
    let mut result = SimpleResult { success: false };
    if landscape::create_bridge(bridge_create_request.name.clone()).await {
        let bridge_info = NetworkIfaceConfig::crate_bridge(bridge_create_request.name.clone());
        let value = serde_json::to_string(&bridge_info).unwrap();
        let mut store_lock = state.store.lock().await;
        store_lock.set(bridge_create_request.name, value);
        drop(store_lock);
        result.success = true;
    }

    Json(result)
}

async fn set_controller(
    State(state): State<NetworkState>,
    Json(AddController {
        link_name,
        link_ifindex,
        master_name,
        master_ifindex,
    }): Json<AddController>,
) -> Json<SimpleResult> {
    let iface_info = landscape::set_controller(&link_name, master_ifindex).await;

    let mut success = false;
    if let Some(iface_info) = iface_info {
        let mut store_lock = state.store.lock().await;
        let mut link_config = if let Some(link_config) = store_lock.get(link_name) {
            serde_json::from_str(&link_config).unwrap()
        } else {
            NetworkIfaceConfig::from_phy_dev(&iface_info)
        };
        link_config.controller_name = master_name;
        store_lock.set(link_config.name.clone(), serde_json::to_string(&link_config).unwrap());
        drop(store_lock);
        success = true;
    }
    Json(SimpleResult { success })
}

// 切换 网卡 所属区域
async fn change_zone(
    State(state): State<NetworkState>,
    Json(ChangeZone { iface_name, zone }): Json<ChangeZone>,
) -> Json<SimpleResult> {
    let success = false;
    let mut store_lock = state.store.lock().await;
    if let Some(link_config) = store_lock.get(iface_name.clone()) {
        let mut link_config: NetworkIfaceConfig = serde_json::from_str(&link_config).unwrap();
        if matches!(zone, IfaceZoneType::Wan) {
            landscape::set_controller(&iface_name, None).await;
            link_config.controller_name = None;
        }
        link_config.zone_type = zone;
        store_lock.set(link_config.name.clone(), serde_json::to_string(&link_config).unwrap());
        drop(store_lock);
    }

    Json(SimpleResult { success })
}

async fn change_dev_status(
    State(state): State<NetworkState>,
    Path((iface_name, enable_in_boot)): Path<(String, bool)>,
) -> Json<SimpleResult> {
    let iface_info = landscape::change_dev_status(&iface_name, enable_in_boot).await;

    let mut success = false;
    if let Some(iface_info) = iface_info {
        let mut store_lock = state.store.lock().await;
        let mut link_config = if let Some(link_config) = store_lock.get(iface_name) {
            serde_json::from_str(&link_config).unwrap()
        } else {
            NetworkIfaceConfig::from_phy_dev(&iface_info)
        };
        link_config.enable_in_boot = enable_in_boot;
        store_lock.set(link_config.name.clone(), serde_json::to_string(&link_config).unwrap());
        drop(store_lock);
        success = true;
    }
    Json(SimpleResult { success })
}

#[derive(Clone, Serialize, Deserialize)]
struct BridgeCreate {
    name: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct AddController {
    link_name: String,
    link_ifindex: u32,
    #[serde(default)]
    master_name: Option<String>,
    #[serde(default)]
    master_ifindex: Option<u32>,
}

#[derive(Clone, Serialize, Deserialize)]
struct ChangeZone {
    iface_name: String,
    zone: IfaceZoneType,
}
