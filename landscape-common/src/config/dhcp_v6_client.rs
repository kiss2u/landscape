use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{net::MacAddr, store::storev2::LandscapeStore};

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "common/dhcp_v6_server.d.ts")]
pub struct IPV6PDServiceConfig {
    pub iface_name: String,
    pub enable: bool,
    pub config: IPV6PDConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(export, export_to = "common/dhcp_v6_server.d.ts")]
pub struct IPV6PDConfig {
    pub mac: MacAddr,
}

impl LandscapeStore for IPV6PDServiceConfig {
    fn get_store_key(&self) -> String {
        self.iface_name.clone()
    }
}
