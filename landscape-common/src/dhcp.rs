use std::{collections::HashMap, net::Ipv4Addr};

use serde::{Deserialize, Serialize};

use crate::{
    net::MacAddr, LANDSCAPE_DEFAULE_LAN_DHCP_RANGE_START, LANDSCAPE_DEFAULE_LAN_DHCP_SERVER_IP,
    LANDSCAPE_DEFAULT_LAN_DHCP_SERVER_NETMASK,
};

/// 持有的数据
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DHCPv4OfferInfo {
    pub relate_time: u64,
    pub offered_ip: HashMap<MacAddr, (Ipv4Addr, u64, u64)>,
}

/// DHCP Server IPv4 Config
#[derive(Debug, Serialize, Deserialize, Clone)]
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
}

impl Default for DHCPv4ServerConfig {
    fn default() -> Self {
        Self {
            ip_range_start: LANDSCAPE_DEFAULE_LAN_DHCP_RANGE_START,
            ip_range_end: None,
            server_ip_addr: LANDSCAPE_DEFAULE_LAN_DHCP_SERVER_IP,
            network_mask: LANDSCAPE_DEFAULT_LAN_DHCP_SERVER_NETMASK,
        }
    }
}
