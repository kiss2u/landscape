use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::store::storev2::LandscapeStore;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "common/config.d.ts")]
pub struct WifiServiceConfig {
    pub iface_name: String,
    pub enable: bool,
    /// hostapd config file
    pub config: String,
}

impl LandscapeStore for WifiServiceConfig {
    fn get_store_key(&self) -> String {
        self.iface_name.clone()
    }
}
