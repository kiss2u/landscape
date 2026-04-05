use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::cert::order::DnsProviderConfig;
use crate::database::repository::LandscapeDBStore;
use crate::utils::id::gen_database_uuid;
use crate::utils::time::get_f64_timestamp;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DnsProviderProfile {
    #[serde(default = "gen_database_uuid")]
    #[cfg_attr(feature = "openapi", schema(required = false))]
    pub id: Uuid,
    pub name: String,
    #[serde(default)]
    pub provider_config: DnsProviderConfig,
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(required = false, nullable = false))]
    pub remark: Option<String>,
    #[serde(default = "get_f64_timestamp")]
    #[cfg_attr(feature = "openapi", schema(required = false))]
    pub update_at: f64,
}

impl LandscapeDBStore<Uuid> for DnsProviderProfile {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_update_at(&self) -> f64 {
        self.update_at
    }

    fn set_update_at(&mut self, ts: f64) {
        self.update_at = ts;
    }
}
