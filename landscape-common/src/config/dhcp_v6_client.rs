use serde::{Deserialize, Serialize};

use crate::{net::MacAddr, store::storev2::LandScapeStore};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IPV6PDServiceConfig {
    pub iface_name: String,
    pub enable: bool,
    pub config: IPV6PDConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IPV6PDConfig {
    pub mac: MacAddr,
}

impl LandScapeStore for IPV6PDServiceConfig {
    fn get_store_key(&self) -> String {
        self.iface_name.clone()
    }
}
