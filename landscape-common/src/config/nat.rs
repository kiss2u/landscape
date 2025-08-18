use core::ops::Range;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use ts_rs::TS;
use uuid::Uuid;

use crate::database::repository::LandscapeDBStore;
use crate::store::storev2::LandscapeStore;
use crate::utils::time::get_f64_timestamp;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "common/nat.d.ts")]
pub struct NatServiceConfig {
    pub iface_name: String,
    pub enable: bool,
    #[serde(default)]
    pub nat_config: NatConfig,
    #[serde(default = "get_f64_timestamp")]
    pub update_at: f64,
}

impl LandscapeStore for NatServiceConfig {
    fn get_store_key(&self) -> String {
        self.iface_name.clone()
    }
}

impl LandscapeDBStore<String> for NatServiceConfig {
    fn get_id(&self) -> String {
        self.iface_name.clone()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(export, export_to = "common/nat.d.ts")]
pub struct NatConfig {
    pub tcp_range: Range<u16>,
    pub udp_range: Range<u16>,
    pub icmp_in_range: Range<u16>,
}

impl Default for NatConfig {
    fn default() -> Self {
        Self {
            tcp_range: 32768..65535,
            udp_range: 32768..65535,
            icmp_in_range: 32768..65535,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "common/nat.d.ts")]
pub struct StaticNatMappingConfig {
    #[serde(default)]
    #[ts(as = "Option<_>")]
    pub id: Uuid,
    pub enable: bool,
    pub remark: String,
    pub wan_port: u16,
    pub wan_iface_name: Option<String>,
    pub lan_port: u16,
    /// If set to `UNSPECIFIED` (e.g., 0.0.0.0 or ::), the mapping targets
    /// the router's own address instead of an internal host.
    pub lan_ip: IpAddr,
    /// TCP / UDP
    pub l4_protocol: Vec<u8>,
    #[serde(default = "get_f64_timestamp")]
    pub update_at: f64,
}

impl StaticNatMappingConfig {
    pub fn is_same_config(&self, other: &StaticNatMappingConfig) -> bool {
        self.id == other.id
            && self.wan_port == other.wan_port
            && self.wan_iface_name == other.wan_iface_name
            && self.lan_port == other.lan_port
            && self.lan_ip == other.lan_ip
            && self.l4_protocol == other.l4_protocol
    }
}

impl LandscapeDBStore<Uuid> for StaticNatMappingConfig {
    fn get_id(&self) -> Uuid {
        self.id
    }
}
