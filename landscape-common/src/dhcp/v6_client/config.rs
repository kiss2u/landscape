use crate::{
    database::repository::LandscapeDBStore, store::storev2::LandscapeStore,
    utils::time::get_f64_timestamp,
};
use serde::{Deserialize, Serialize};

use crate::net::MacAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct IPV6PDServiceConfig {
    pub iface_name: String,
    pub enable: bool,
    pub config: IPV6PDConfig,
    #[serde(default = "get_f64_timestamp")]
    #[cfg_attr(feature = "openapi", schema(required = false))]
    pub update_at: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct IPV6PDConfig {
    #[cfg_attr(feature = "openapi", schema(value_type = String))]
    pub mac: MacAddr,
}

impl LandscapeDBStore<String> for IPV6PDServiceConfig {
    fn get_id(&self) -> String {
        self.iface_name.clone()
    }
    fn get_update_at(&self) -> f64 {
        self.update_at
    }
    fn set_update_at(&mut self, ts: f64) {
        self.update_at = ts;
    }
}

impl LandscapeStore for IPV6PDServiceConfig {
    fn get_store_key(&self) -> String {
        self.iface_name.clone()
    }
}

impl crate::iface::config::ZoneAwareConfig for IPV6PDServiceConfig {
    fn iface_name(&self) -> &str {
        &self.iface_name
    }
    fn zone_requirement() -> crate::iface::config::ZoneRequirement {
        crate::iface::config::ZoneRequirement::WanOrPpp
    }
    fn service_kind() -> crate::iface::config::ServiceKind {
        crate::iface::config::ServiceKind::Ipv6Pd
    }
}
