use std::{collections::HashSet, net::IpAddr, sync::Arc};

use landscape_common::net::MacAddr;
use tokio::sync::RwLock;

#[derive(Eq, Hash, PartialEq, Debug)]
pub struct WanRouteInfo {
    pub weight: u32,
    pub iface_name: String,
    pub iface_ip: IpAddr,
    pub iface_mac: Option<MacAddr>,

    pub gateway_ip: IpAddr,
    pub gateway_mac: MacAddr,
}

#[derive(Eq, Hash, PartialEq, Debug)]
pub struct LanRouteInfo {
    pub iface_name: String,
    pub iface_ip: IpAddr,
    pub prefix: u8,
    pub iface_mac: Option<MacAddr>,
}

#[derive(Clone)]
pub struct IpRouteService {
    wan_ifaces: Arc<RwLock<HashSet<WanRouteInfo>>>,
    lan_ifaces: Arc<RwLock<HashSet<LanRouteInfo>>>,
}

impl IpRouteService {
    pub fn new() -> Self {
        IpRouteService {
            wan_ifaces: Arc::new(RwLock::new(HashSet::new())),
            lan_ifaces: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    pub async fn insert_lan_route(&self, info: LanRouteInfo) {
        let mut lock = self.lan_ifaces.write().await;
        lock.insert(info);
        // TODO 刷新 EBPF 中 LAN map
    }

    pub async fn remove_lan_route(&self, info: LanRouteInfo) {
        let mut lock = self.lan_ifaces.write().await;
        lock.insert(info);
    }

    pub async fn insert_wan_route(&self, info: WanRouteInfo) {
        let mut lock = self.wan_ifaces.write().await;
        lock.insert(info);
        // TODO 刷新 EBPF 中 WAN map
    }

    pub async fn remove_wan_route(&self, info: WanRouteInfo) {
        let mut lock = self.wan_ifaces.write().await;
        lock.insert(info);
    }
}
