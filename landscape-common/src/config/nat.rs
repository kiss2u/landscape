use core::ops::Range;
use serde::{Deserialize, Serialize};

use crate::store::storev2::LandScapeStore;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatServiceConfig {
    pub iface_name: String,
    pub enable: bool,
    #[serde(default)]
    pub nat_config: NatConfig,
}

impl LandScapeStore for NatServiceConfig {
    fn get_store_key(&self) -> String {
        self.iface_name.clone()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
