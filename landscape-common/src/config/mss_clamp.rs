use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::store::storev2::LandScapeStore;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "common/mss_clamp.d.ts")]
pub struct MSSClampServiceConfig {
    pub iface_name: String,
    pub enable: bool,

    /// clamp size
    /// PPPoE: 1500 - 8 = 1492
    #[serde(default = "default_clamp_size")]
    pub clamp_size: u16,
}

impl LandScapeStore for MSSClampServiceConfig {
    fn get_store_key(&self) -> String {
        self.iface_name.clone()
    }
}

const fn default_clamp_size() -> u16 {
    1492
}
