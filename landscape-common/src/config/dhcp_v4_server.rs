use std::net::Ipv4Addr;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::net::MacAddr;
use crate::utils::time::get_f64_timestamp;
use crate::{store::storev2::LandscapeStore, LANDSCAPE_DEFAULT_LAN_NAME};

use crate::{
    LANDSCAPE_DEFAULE_LAN_DHCP_RANGE_START, LANDSCAPE_DEFAULE_LAN_DHCP_SERVER_IP,
    LANDSCAPE_DEFAULT_LAN_DHCP_SERVER_NETMASK, LANDSCAPE_DHCP_DEFAULT_ADDRESS_LEASE_TIME,
};

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "common/dhcp_v4_server.d.ts")]
pub struct DHCPv4ServiceConfig {
    pub iface_name: String,
    pub enable: bool,
    #[serde(default)]
    pub config: DHCPv4ServerConfig,
    /// 最近一次编译时间
    #[serde(default = "get_f64_timestamp")]
    pub update_at: f64,
}

impl Default for DHCPv4ServiceConfig {
    fn default() -> Self {
        Self {
            iface_name: LANDSCAPE_DEFAULT_LAN_NAME.into(),
            enable: true,
            config: DHCPv4ServerConfig::default(),
            update_at: get_f64_timestamp(),
        }
    }
}

impl LandscapeStore for DHCPv4ServiceConfig {
    fn get_store_key(&self) -> String {
        self.iface_name.clone()
    }
}

/// DHCP Server IPv4 Config
#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(export, export_to = "common/dhcp_v4_server.d.ts")]
pub struct DHCPv4ServerConfig {
    /// dhcp options
    // #[serde(default)]
    // options: Vec<DhcpOptions>,
    /// range start
    pub ip_range_start: Ipv4Addr,
    /// range end [not include]
    #[serde(default)]
    pub ip_range_end: Option<Ipv4Addr>,

    /// DHCP Server Addr e.g. 192.168.1.1
    pub server_ip_addr: Ipv4Addr,
    /// network mask e.g. 255.255.255.0 = 24
    pub network_mask: u8,

    #[serde(default)]
    pub address_lease_time: Option<u32>,

    #[serde(default)]
    /// Static MAC --> IP address binding
    pub mac_binding_records: Vec<MacBindingRecord>,
}

impl Default for DHCPv4ServerConfig {
    fn default() -> Self {
        Self {
            ip_range_start: LANDSCAPE_DEFAULE_LAN_DHCP_RANGE_START,
            ip_range_end: None,
            server_ip_addr: LANDSCAPE_DEFAULE_LAN_DHCP_SERVER_IP,
            network_mask: LANDSCAPE_DEFAULT_LAN_DHCP_SERVER_NETMASK,
            address_lease_time: Some(LANDSCAPE_DHCP_DEFAULT_ADDRESS_LEASE_TIME),
            mac_binding_records: vec![],
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(export, export_to = "common/dhcp_v4_server.d.ts")]
pub struct MacBindingRecord {
    pub mac: MacAddr,
    pub ip: Ipv4Addr,
    #[serde(default = "default_binding_record")]
    pub expire_time: u32,
}

const fn default_binding_record() -> u32 {
    // 24 小时
    86400
}
