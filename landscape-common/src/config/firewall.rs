use serde::{Deserialize, Serialize};

use crate::store::storev2::LandScapeStore;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallServiceConfig {
    pub iface_name: String,
    pub enable: bool,
}

impl LandScapeStore for FirewallServiceConfig {
    fn get_store_key(&self) -> String {
        self.iface_name.clone()
    }
}
