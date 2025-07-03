use std::{collections::HashMap, sync::Arc};

use landscape_common::route::{LanRouteInfo, WanRouteInfo};
use landscape_ebpf::map_setting::route::{add_lan_route, add_wan_route, del_lan_route};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct IpRouteService {
    wan_ifaces: Arc<RwLock<HashMap<String, WanRouteInfo>>>,
    lan_ifaces: Arc<RwLock<HashMap<String, LanRouteInfo>>>,
}

impl IpRouteService {
    pub fn new() -> Self {
        IpRouteService {
            wan_ifaces: Arc::new(RwLock::new(HashMap::new())),
            lan_ifaces: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn insert_lan_route(&self, key: &str, info: LanRouteInfo) {
        let mut lock = self.lan_ifaces.write().await;
        add_lan_route(info.clone());
        lock.insert(key.to_string(), info);
    }

    pub async fn remove_lan_route(&self, key: &str) {
        let mut lock = self.lan_ifaces.write().await;
        let result = lock.remove(key);
        if let Some(info) = result {
            del_lan_route(info);
        }
    }

    pub async fn insert_wan_route(&self, key: &str, info: WanRouteInfo) {
        let mut lock = self.wan_ifaces.write().await;
        add_wan_route(0, info.clone());
        lock.insert(key.to_string(), info);
    }

    pub async fn remove_wan_route(&self, key: &str) {
        let mut lock = self.wan_ifaces.write().await;
        let result = lock.remove(key);
        if let Some(_) = result {
            // del_wan_route(info.clone());
        }
    }
}
