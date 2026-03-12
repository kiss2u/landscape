use serde::{Deserialize, Serialize};

use crate::net::MacAddr;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum CallerLookupSource {
    DhcpV4,
    Arp,
    Ipv6Ra,
    DhcpV6,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CallerLookupMatch {
    pub iface_name: String,
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub mac: Option<MacAddr>,
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub hostname: Option<String>,
    pub source: CallerLookupSource,
}
