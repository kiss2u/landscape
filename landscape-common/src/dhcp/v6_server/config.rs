use serde::{Deserialize, Serialize};

use crate::config::ra::IPV6RaConfigSource;
use crate::service::ServiceConfigError;

/// DHCPv6 server config — nested inside IPV6RAConfig.
/// DHCPv6 does NOT reference RA sources by index. Instead, it defines filter criteria.
/// At runtime, the server checks each resolved RA prefix against the filter.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DHCPv6ServerConfig {
    pub enable: bool,

    /// Independent prefix sources for DHCPv6 (used in Stateful/SlaacDhcpv6 modes)
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(required = false))]
    pub source: Vec<IPV6RaConfigSource>,

    /// IA_NA: stateful address assignment
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(required = false, nullable = false))]
    pub ia_na: Option<DHCPv6IANAConfig>,

    /// IA_PD: prefix delegation to downstream routers
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(required = false, nullable = false))]
    pub ia_pd: Option<DHCPv6IAPDConfig>,
}

impl Default for DHCPv6ServerConfig {
    fn default() -> Self {
        Self {
            enable: false,
            source: vec![],
            ia_na: None,
            ia_pd: None,
        }
    }
}

/// IA_NA config — uses runtime-resolved RA prefixes that match the filter.
/// Only RA prefixes with prefix_len <= max_prefix_len qualify for address assignment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DHCPv6IANAConfig {
    /// Max prefix length to qualify (e.g., 64 means prefixes with len <= 64 are used).
    /// Shorter prefixes have more host space, so they also qualify.
    pub max_prefix_len: u8,

    /// Host part range start (suffix value, e.g., 0x100 = 256)
    pub pool_start: u64,

    /// Host part range end (optional, defaults to subnet max)
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(required = false, nullable = false))]
    pub pool_end: Option<u64>,

    /// Preferred lifetime (seconds)
    pub preferred_lifetime: u32,

    /// Valid lifetime (seconds)
    pub valid_lifetime: u32,
}

/// IA_PD config — delegates sub-prefixes from qualifying RA base prefixes.
/// Only base prefixes with prefix_len <= max_source_prefix_len qualify.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DHCPv6IAPDConfig {
    /// Max source prefix length to qualify (e.g., 56 means only prefixes with len <= 56).
    /// Sources with prefix_len > this are too small to delegate from.
    pub max_source_prefix_len: u8,

    /// Prefix length to delegate to clients (e.g., 60, 64).
    /// Must be > source prefix length.
    pub delegate_prefix_len: u8,

    /// Sub-prefix pool range start index (within the delegatable space)
    pub pool_start_index: u32,

    /// Sub-prefix pool range end index (optional)
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(required = false, nullable = false))]
    pub pool_end_index: Option<u32>,

    /// Preferred lifetime (seconds)
    pub preferred_lifetime: u32,

    /// Valid lifetime (seconds)
    pub valid_lifetime: u32,
}

impl DHCPv6ServerConfig {
    pub fn validate(&self) -> Result<(), ServiceConfigError> {
        if !self.enable {
            return Ok(());
        }

        // At least one of ia_na or ia_pd must be set if enabled
        if self.ia_na.is_none() && self.ia_pd.is_none() {
            return Err(ServiceConfigError::InvalidConfig {
                reason: "DHCPv6 enabled but neither IA_NA nor IA_PD is configured".to_string(),
            });
        }

        if let Some(ia_na) = &self.ia_na {
            if ia_na.max_prefix_len == 0 || ia_na.max_prefix_len > 127 {
                return Err(ServiceConfigError::InvalidConfig {
                    reason: format!(
                        "IA_NA max_prefix_len ({}) must be between 1 and 127",
                        ia_na.max_prefix_len
                    ),
                });
            }

            if let Some(pool_end) = ia_na.pool_end {
                if pool_end <= ia_na.pool_start {
                    return Err(ServiceConfigError::InvalidConfig {
                        reason: format!(
                            "IA_NA pool_end ({}) must be > pool_start ({})",
                            pool_end, ia_na.pool_start
                        ),
                    });
                }
            }

            if ia_na.valid_lifetime == 0 {
                return Err(ServiceConfigError::InvalidConfig {
                    reason: "IA_NA valid_lifetime must be > 0".to_string(),
                });
            }

            if ia_na.preferred_lifetime > ia_na.valid_lifetime {
                return Err(ServiceConfigError::InvalidConfig {
                    reason: "IA_NA preferred_lifetime must be <= valid_lifetime".to_string(),
                });
            }
        }

        if let Some(ia_pd) = &self.ia_pd {
            if ia_pd.max_source_prefix_len == 0 || ia_pd.max_source_prefix_len > 126 {
                return Err(ServiceConfigError::InvalidConfig {
                    reason: format!(
                        "IA_PD max_source_prefix_len ({}) must be between 1 and 126",
                        ia_pd.max_source_prefix_len
                    ),
                });
            }

            if ia_pd.delegate_prefix_len <= ia_pd.max_source_prefix_len {
                return Err(ServiceConfigError::InvalidConfig {
                    reason: format!(
                        "IA_PD delegate_prefix_len ({}) must be > max_source_prefix_len ({})",
                        ia_pd.delegate_prefix_len, ia_pd.max_source_prefix_len
                    ),
                });
            }

            if ia_pd.delegate_prefix_len > 128 {
                return Err(ServiceConfigError::InvalidConfig {
                    reason: format!(
                        "IA_PD delegate_prefix_len ({}) must be <= 128",
                        ia_pd.delegate_prefix_len
                    ),
                });
            }

            if let Some(pool_end_index) = ia_pd.pool_end_index {
                if pool_end_index <= ia_pd.pool_start_index {
                    return Err(ServiceConfigError::InvalidConfig {
                        reason: format!(
                            "IA_PD pool_end_index ({}) must be > pool_start_index ({})",
                            pool_end_index, ia_pd.pool_start_index
                        ),
                    });
                }
            }

            if ia_pd.valid_lifetime == 0 {
                return Err(ServiceConfigError::InvalidConfig {
                    reason: "IA_PD valid_lifetime must be > 0".to_string(),
                });
            }

            if ia_pd.preferred_lifetime > ia_pd.valid_lifetime {
                return Err(ServiceConfigError::InvalidConfig {
                    reason: "IA_PD preferred_lifetime must be <= valid_lifetime".to_string(),
                });
            }
        }

        Ok(())
    }
}
