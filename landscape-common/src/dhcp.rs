use std::net::Ipv4Addr;

use serde::{Deserialize, Serialize};

use crate::net::MacAddr;

/// 持有的数据
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DHCPv4OfferInfo {
    pub relative_boot_time: u64,
    pub offered_ips: Vec<DHCPv4OfferInfoItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DHCPv4OfferInfoItem {
    pub mac: MacAddr,
    pub ip: Ipv4Addr,
    pub relative_active_time: u64,
    pub expire_time: u32,
}
