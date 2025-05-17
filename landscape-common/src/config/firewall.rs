use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::store::storev2::LandscapeStore;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "common/firewall.d.ts")]
pub struct FirewallServiceConfig {
    pub iface_name: String,
    pub enable: bool,
}

impl LandscapeStore for FirewallServiceConfig {
    fn get_store_key(&self) -> String {
        self.iface_name.clone()
    }
}
