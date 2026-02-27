use std::collections::HashSet;
use std::net::Ipv6Addr;

use serde::{Deserialize, Serialize};

use crate::database::repository::LandscapeDBStore;
use crate::dhcp::v6_server::config::DHCPv6ServerConfig;
use crate::service::ServiceConfigError;
use crate::store::storev2::LandscapeStore;
use crate::utils::time::get_f64_timestamp;

use super::ra::{IPV6RaConfigSource, RouterFlags};

/// IPv6 service preset mode
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum IPv6ServiceMode {
    /// Pure RA: sends prefix info (A=1), no DHCPv6
    #[default]
    Slaac,
    /// Pure DHCPv6: RA sends M=1 without PrefixInformation, DHCPv6 assigns addresses
    Stateful,
    /// RA + DHCPv6: RA sends ULA static prefixes, DHCPv6 uses its own source (GUA PD/static)
    SlaacDhcpv6,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LanIPv6ServiceConfig {
    pub iface_name: String,
    pub enable: bool,
    pub config: LanIPv6Config,

    #[serde(default = "get_f64_timestamp")]
    #[cfg_attr(feature = "openapi", schema(required = false))]
    pub update_at: f64,
}

impl LandscapeDBStore<String> for LanIPv6ServiceConfig {
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LanIPv6Config {
    /// IPv6 service mode
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(required = true))]
    pub mode: IPv6ServiceMode,
    /// Router Advertisement Interval
    pub ad_interval: u32,
    /// Router Advertisement Flag
    #[serde(default = "ra_flag_default")]
    #[cfg_attr(feature = "openapi", schema(required = true))]
    pub ra_flag: RouterFlags,
    /// RA prefix sources
    pub source: Vec<IPV6RaConfigSource>,
    /// DHCPv6 server config (nested within LAN IPv6)
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(required = false, nullable = false))]
    pub dhcpv6: Option<DHCPv6ServerConfig>,
}

fn ra_flag_default() -> RouterFlags {
    0xc0.into()
}

impl LanIPv6Config {
    pub fn validate(&self) -> Result<(), ServiceConfigError> {
        match self.mode {
            IPv6ServiceMode::Slaac => self.validate_slaac(),
            IPv6ServiceMode::Stateful => self.validate_stateful(),
            IPv6ServiceMode::SlaacDhcpv6 => self.validate_slaac_dhcpv6(),
        }
    }

    fn validate_slaac(&self) -> Result<(), ServiceConfigError> {
        // source must not be empty
        if self.source.is_empty() {
            return Err(ServiceConfigError::InvalidConfig {
                reason: "Slaac mode requires at least one prefix source".to_string(),
            });
        }
        // M must be 0
        if self.ra_flag.managed_address_config {
            return Err(ServiceConfigError::InvalidConfig {
                reason: "Slaac mode requires M flag to be 0".to_string(),
            });
        }
        // DHCPv6 must be None or disabled
        if let Some(dhcpv6) = &self.dhcpv6 {
            if dhcpv6.enable {
                return Err(ServiceConfigError::InvalidConfig {
                    reason: "Slaac mode does not allow DHCPv6 to be enabled".to_string(),
                });
            }
        }
        // Validate source uniqueness
        self.validate_sources(&self.source)?;
        Ok(())
    }

    fn validate_stateful(&self) -> Result<(), ServiceConfigError> {
        // M=1, O=1
        if !self.ra_flag.managed_address_config || !self.ra_flag.other_config {
            return Err(ServiceConfigError::InvalidConfig {
                reason: "Stateful mode requires M=1 and O=1".to_string(),
            });
        }
        // DHCPv6 must be enabled with non-empty source
        let dhcpv6 = self.dhcpv6.as_ref().ok_or(ServiceConfigError::InvalidConfig {
            reason: "Stateful mode requires DHCPv6 configuration".to_string(),
        })?;
        if !dhcpv6.enable {
            return Err(ServiceConfigError::InvalidConfig {
                reason: "Stateful mode requires DHCPv6 to be enabled".to_string(),
            });
        }
        if dhcpv6.source.is_empty() {
            return Err(ServiceConfigError::InvalidConfig {
                reason: "Stateful mode requires at least one DHCPv6 prefix source".to_string(),
            });
        }
        // Validate DHCPv6 source uniqueness
        self.validate_sources(&dhcpv6.source)?;
        // Validate DHCPv6 config
        dhcpv6.validate()?;
        Ok(())
    }

    fn validate_slaac_dhcpv6(&self) -> Result<(), ServiceConfigError> {
        // M=1, O=1
        if !self.ra_flag.managed_address_config || !self.ra_flag.other_config {
            return Err(ServiceConfigError::InvalidConfig {
                reason: "SlaacDhcpv6 mode requires M=1 and O=1".to_string(),
            });
        }
        // RA source must be all Static and in ULA range (fc00::/7)
        if self.source.is_empty() {
            return Err(ServiceConfigError::InvalidConfig {
                reason: "SlaacDhcpv6 mode requires at least one RA prefix source".to_string(),
            });
        }
        for src in &self.source {
            match src {
                IPV6RaConfigSource::Static(cfg) => {
                    if !is_ula(cfg.base_prefix) {
                        return Err(ServiceConfigError::InvalidConfig {
                            reason: format!(
                                "SlaacDhcpv6 mode requires RA sources to be ULA (fc00::/7), got: {}",
                                cfg.base_prefix
                            ),
                        });
                    }
                }
                IPV6RaConfigSource::Pd(_) => {
                    return Err(ServiceConfigError::InvalidConfig {
                        reason: "SlaacDhcpv6 mode only allows Static RA sources".to_string(),
                    });
                }
            }
        }
        // DHCPv6 must be enabled with non-empty source
        let dhcpv6 = self.dhcpv6.as_ref().ok_or(ServiceConfigError::InvalidConfig {
            reason: "SlaacDhcpv6 mode requires DHCPv6 configuration".to_string(),
        })?;
        if !dhcpv6.enable {
            return Err(ServiceConfigError::InvalidConfig {
                reason: "SlaacDhcpv6 mode requires DHCPv6 to be enabled".to_string(),
            });
        }
        if dhcpv6.source.is_empty() {
            return Err(ServiceConfigError::InvalidConfig {
                reason: "SlaacDhcpv6 mode requires at least one DHCPv6 prefix source".to_string(),
            });
        }
        // Cross-validate: sub_index/subnet_index must not overlap between source and dhcpv6.source
        self.validate_cross_sources(&self.source, &dhcpv6.source)?;
        // Validate DHCPv6 config
        dhcpv6.validate()?;
        Ok(())
    }

    /// Validate uniqueness within a single source list
    fn validate_sources(&self, source: &[IPV6RaConfigSource]) -> Result<(), ServiceConfigError> {
        let mut base_prefixes = HashSet::<Ipv6Addr>::new();
        let mut depend_ifaces = HashSet::<String>::new();
        let mut sub_indices = HashSet::<u32>::new();

        for src in source {
            match src {
                IPV6RaConfigSource::Static(cfg) => {
                    if !base_prefixes.insert(cfg.base_prefix) {
                        return Err(ServiceConfigError::InvalidConfig {
                            reason: format!("Duplicate base_prefix found: {}", cfg.base_prefix),
                        });
                    }
                    if !sub_indices.insert(cfg.sub_index) {
                        return Err(ServiceConfigError::InvalidConfig {
                            reason: format!(
                                "Duplicate sub_index/subnet_index found: {}",
                                cfg.sub_index
                            ),
                        });
                    }
                }
                IPV6RaConfigSource::Pd(cfg) => {
                    if !depend_ifaces.insert(cfg.depend_iface.clone()) {
                        return Err(ServiceConfigError::InvalidConfig {
                            reason: format!("Duplicate depend_iface found: {}", cfg.depend_iface),
                        });
                    }
                    if !sub_indices.insert(cfg.subnet_index) {
                        return Err(ServiceConfigError::InvalidConfig {
                            reason: format!(
                                "Duplicate sub_index/subnet_index found: {}",
                                cfg.subnet_index
                            ),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    /// Cross-validate sub_index/subnet_index between two source lists
    fn validate_cross_sources(
        &self,
        ra_source: &[IPV6RaConfigSource],
        dhcpv6_source: &[IPV6RaConfigSource],
    ) -> Result<(), ServiceConfigError> {
        let mut sub_indices = HashSet::<u32>::new();

        // Collect from RA sources
        for src in ra_source {
            let idx = match src {
                IPV6RaConfigSource::Static(cfg) => cfg.sub_index,
                IPV6RaConfigSource::Pd(cfg) => cfg.subnet_index,
            };
            sub_indices.insert(idx);
        }

        // Check DHCPv6 sources
        for src in dhcpv6_source {
            let idx = match src {
                IPV6RaConfigSource::Static(cfg) => cfg.sub_index,
                IPV6RaConfigSource::Pd(cfg) => cfg.subnet_index,
            };
            if !sub_indices.insert(idx) {
                return Err(ServiceConfigError::InvalidConfig {
                    reason: format!(
                        "sub_index/subnet_index {} overlaps between RA source and DHCPv6 source",
                        idx
                    ),
                });
            }
        }

        // Also validate each list individually
        self.validate_sources(ra_source)?;
        self.validate_sources(dhcpv6_source)?;

        Ok(())
    }
}

/// Check if an IPv6 address is in the ULA range (fc00::/7)
fn is_ula(addr: Ipv6Addr) -> bool {
    let first_byte = addr.octets()[0];
    (first_byte & 0xfe) == 0xfc
}

impl LanIPv6Config {
    pub fn new(depend_iface: String) -> Self {
        use super::ra::IPv6RaPdConfig;
        let source = vec![IPV6RaConfigSource::Pd(IPv6RaPdConfig {
            depend_iface,
            ra_preferred_lifetime: 300,
            ra_valid_lifetime: 300,
            prefix_len: 64,
            subnet_index: 1,
        })];
        Self {
            mode: IPv6ServiceMode::Slaac,
            source,
            ra_flag: ra_flag_default(),
            ad_interval: 300,
            dhcpv6: None,
        }
    }
}

impl LandscapeStore for LanIPv6ServiceConfig {
    fn get_store_key(&self) -> String {
        self.iface_name.clone()
    }
}

impl super::iface::ZoneAwareConfig for LanIPv6ServiceConfig {
    fn iface_name(&self) -> &str {
        &self.iface_name
    }
    fn zone_requirement() -> super::iface::ZoneRequirement {
        super::iface::ZoneRequirement::LanOnly
    }
    fn service_kind() -> super::iface::ServiceKind {
        super::iface::ServiceKind::LanIpv6
    }
}
