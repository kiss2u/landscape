use std::collections::HashMap;
use std::fmt;
use std::net::Ipv6Addr;

use serde::{Deserialize, Serialize};

use crate::database::repository::LandscapeDBStore;
use crate::dhcp::v6_server::config::DHCPv6ServerConfig;
use crate::iface::config::{ServiceKind, ZoneAwareConfig, ZoneRequirement};
use crate::ipv6::ra::RouterFlags;
use crate::service::ServiceConfigError;
use crate::store::storev2::LandscapeStore;
use crate::utils::time::get_f64_timestamp;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum IPv6ServiceMode {
    #[default]
    Slaac,
    Stateful,
    SlaacDhcpv6,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum SourceServiceKind {
    Ra,
    Na,
    IaPd,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParentPrefixKey {
    Static(Ipv6Addr),
    Pd(String),
}

impl fmt::Display for ParentPrefixKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParentPrefixKey::Static(addr) => write!(f, "static({})", addr),
            ParentPrefixKey::Pd(iface) => write!(f, "pd({})", iface),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(tag = "t", rename_all = "snake_case")]
pub enum LanIPv6SourceConfig {
    RaStatic {
        #[cfg_attr(feature = "openapi", schema(value_type = String))]
        base_prefix: Ipv6Addr,
        pool_index: u32,
        preferred_lifetime: u32,
        valid_lifetime: u32,
    },
    RaPd {
        depend_iface: String,
        pool_index: u32,
        preferred_lifetime: u32,
        valid_lifetime: u32,
    },
    NaStatic {
        #[cfg_attr(feature = "openapi", schema(value_type = String))]
        base_prefix: Ipv6Addr,
        pool_index: u32,
    },
    NaPd {
        depend_iface: String,
        pool_index: u32,
    },
    PdStatic {
        #[cfg_attr(feature = "openapi", schema(value_type = String))]
        base_prefix: Ipv6Addr,
        base_prefix_len: u8,
        pool_index: u32,
        pool_len: u8,
    },
    PdPd {
        depend_iface: String,
        max_source_prefix_len: u8,
        pool_index: u32,
        pool_len: u8,
    },
}

impl LanIPv6SourceConfig {
    pub fn parent_key(&self) -> ParentPrefixKey {
        match self {
            LanIPv6SourceConfig::RaStatic { base_prefix, .. }
            | LanIPv6SourceConfig::NaStatic { base_prefix, .. } => {
                ParentPrefixKey::Static(*base_prefix)
            }
            LanIPv6SourceConfig::PdStatic { base_prefix, .. } => {
                ParentPrefixKey::Static(*base_prefix)
            }
            LanIPv6SourceConfig::RaPd { depend_iface, .. }
            | LanIPv6SourceConfig::NaPd { depend_iface, .. }
            | LanIPv6SourceConfig::PdPd { depend_iface, .. } => {
                ParentPrefixKey::Pd(depend_iface.clone())
            }
        }
    }

    pub fn service_kind(&self) -> SourceServiceKind {
        match self {
            LanIPv6SourceConfig::RaStatic { .. } | LanIPv6SourceConfig::RaPd { .. } => {
                SourceServiceKind::Ra
            }
            LanIPv6SourceConfig::NaStatic { .. } | LanIPv6SourceConfig::NaPd { .. } => {
                SourceServiceKind::Na
            }
            LanIPv6SourceConfig::PdStatic { .. } | LanIPv6SourceConfig::PdPd { .. } => {
                SourceServiceKind::IaPd
            }
        }
    }

    pub fn pool_index(&self) -> u32 {
        match self {
            LanIPv6SourceConfig::RaStatic { pool_index, .. }
            | LanIPv6SourceConfig::RaPd { pool_index, .. }
            | LanIPv6SourceConfig::NaStatic { pool_index, .. }
            | LanIPv6SourceConfig::NaPd { pool_index, .. }
            | LanIPv6SourceConfig::PdStatic { pool_index, .. }
            | LanIPv6SourceConfig::PdPd { pool_index, .. } => *pool_index,
        }
    }

    pub fn pool_len(&self) -> u8 {
        match self {
            LanIPv6SourceConfig::RaStatic { .. }
            | LanIPv6SourceConfig::RaPd { .. }
            | LanIPv6SourceConfig::NaStatic { .. }
            | LanIPv6SourceConfig::NaPd { .. } => 64,
            LanIPv6SourceConfig::PdStatic { pool_len, .. }
            | LanIPv6SourceConfig::PdPd { pool_len, .. } => *pool_len,
        }
    }

    pub fn is_static(&self) -> bool {
        matches!(
            self,
            LanIPv6SourceConfig::RaStatic { .. }
                | LanIPv6SourceConfig::NaStatic { .. }
                | LanIPv6SourceConfig::PdStatic { .. }
        )
    }

    pub fn base_prefix(&self) -> Option<Ipv6Addr> {
        match self {
            LanIPv6SourceConfig::RaStatic { base_prefix, .. }
            | LanIPv6SourceConfig::NaStatic { base_prefix, .. }
            | LanIPv6SourceConfig::PdStatic { base_prefix, .. } => Some(*base_prefix),
            _ => None,
        }
    }

    pub fn depend_iface(&self) -> Option<&str> {
        match self {
            LanIPv6SourceConfig::RaPd { depend_iface, .. }
            | LanIPv6SourceConfig::NaPd { depend_iface, .. }
            | LanIPv6SourceConfig::PdPd { depend_iface, .. } => Some(depend_iface),
            _ => None,
        }
    }
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
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(required = true))]
    pub mode: IPv6ServiceMode,
    pub ad_interval: u32,
    #[serde(default = "ra_flag_default")]
    #[cfg_attr(feature = "openapi", schema(required = true))]
    pub ra_flag: RouterFlags,
    #[serde(default)]
    pub sources: Vec<LanIPv6SourceConfig>,
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(required = false, nullable = false))]
    pub dhcpv6: Option<DHCPv6ServerConfig>,
}

fn ra_flag_default() -> RouterFlags {
    0xc0.into()
}

fn is_ula(addr: Ipv6Addr) -> bool {
    let first_byte = addr.octets()[0];
    (first_byte & 0xfe) == 0xfc
}

fn blocks_overlap(_parent_prefix_len: u8, idx_a: u32, len_a: u8, idx_b: u32, len_b: u8) -> bool {
    let max_len = len_a.max(len_b);
    let scale_a = 1u64 << (max_len - len_a) as u64;
    let start_a = (idx_a as u64) * scale_a;
    let end_a = start_a + scale_a;
    let scale_b = 1u64 << (max_len - len_b) as u64;
    let start_b = (idx_b as u64) * scale_b;
    let end_b = start_b + scale_b;
    start_a < end_b && start_b < end_a
}

impl LanIPv6Config {
    pub fn validate(&self) -> Result<(), ServiceConfigError> {
        for src in &self.sources {
            validate_source_entry(src)?;
        }

        let active_sources = self.active_sources();
        validate_sources_no_conflict(&active_sources)?;

        match self.mode {
            IPv6ServiceMode::Slaac => self.validate_slaac(),
            IPv6ServiceMode::Stateful => self.validate_stateful(),
            IPv6ServiceMode::SlaacDhcpv6 => self.validate_slaac_dhcpv6(),
        }
    }

    pub fn active_sources(&self) -> Vec<LanIPv6SourceConfig> {
        self.sources
            .iter()
            .filter(|s| match self.mode {
                IPv6ServiceMode::Slaac => s.service_kind() == SourceServiceKind::Ra,
                IPv6ServiceMode::Stateful => {
                    s.service_kind() == SourceServiceKind::Na
                        || s.service_kind() == SourceServiceKind::IaPd
                }
                IPv6ServiceMode::SlaacDhcpv6 => true,
            })
            .cloned()
            .collect()
    }

    fn validate_slaac(&self) -> Result<(), ServiceConfigError> {
        let ra_count =
            self.sources.iter().filter(|s| s.service_kind() == SourceServiceKind::Ra).count();
        if ra_count == 0 {
            return Err(ServiceConfigError::InvalidConfig {
                reason: "Slaac mode requires at least one RA prefix source".to_string(),
            });
        }
        if self.ra_flag.managed_address_config {
            return Err(ServiceConfigError::InvalidConfig {
                reason: "Slaac mode requires M flag to be 0".to_string(),
            });
        }
        if let Some(dhcpv6) = &self.dhcpv6 {
            if dhcpv6.enable {
                return Err(ServiceConfigError::InvalidConfig {
                    reason: "Slaac mode does not allow DHCPv6 to be enabled".to_string(),
                });
            }
        }
        Ok(())
    }

    fn validate_stateful(&self) -> Result<(), ServiceConfigError> {
        if !self.ra_flag.managed_address_config || !self.ra_flag.other_config {
            return Err(ServiceConfigError::InvalidConfig {
                reason: "Stateful mode requires M=1 and O=1".to_string(),
            });
        }
        let na_count =
            self.sources.iter().filter(|s| s.service_kind() == SourceServiceKind::Na).count();
        if na_count == 0 {
            return Err(ServiceConfigError::InvalidConfig {
                reason: "Stateful mode requires at least one DHCPv6 NA prefix source".to_string(),
            });
        }
        let dhcpv6 = self.dhcpv6.as_ref().ok_or(ServiceConfigError::InvalidConfig {
            reason: "Stateful mode requires DHCPv6 configuration".to_string(),
        })?;
        if !dhcpv6.enable {
            return Err(ServiceConfigError::InvalidConfig {
                reason: "Stateful mode requires DHCPv6 to be enabled".to_string(),
            });
        }
        dhcpv6.validate()?;
        Ok(())
    }

    fn validate_slaac_dhcpv6(&self) -> Result<(), ServiceConfigError> {
        if !self.ra_flag.managed_address_config || !self.ra_flag.other_config {
            return Err(ServiceConfigError::InvalidConfig {
                reason: "SlaacDhcpv6 mode requires M=1 and O=1".to_string(),
            });
        }
        let ra_sources: Vec<_> =
            self.sources.iter().filter(|s| s.service_kind() == SourceServiceKind::Ra).collect();
        if ra_sources.is_empty() {
            return Err(ServiceConfigError::InvalidConfig {
                reason: "SlaacDhcpv6 mode requires at least one RA prefix source".to_string(),
            });
        }
        for src in &ra_sources {
            match src {
                LanIPv6SourceConfig::RaStatic { base_prefix, .. } => {
                    if !is_ula(*base_prefix) {
                        return Err(ServiceConfigError::InvalidConfig {
                            reason: format!(
                                "SlaacDhcpv6 mode requires RA sources to be ULA (fc00::/7), got: {}",
                                base_prefix
                            ),
                        });
                    }
                }
                LanIPv6SourceConfig::RaPd { .. } => {
                    return Err(ServiceConfigError::InvalidConfig {
                        reason: "SlaacDhcpv6 mode only allows Static RA sources".to_string(),
                    });
                }
                _ => {}
            }
        }
        let dhcpv6_source_count = self
            .sources
            .iter()
            .filter(|s| {
                s.service_kind() == SourceServiceKind::Na
                    || s.service_kind() == SourceServiceKind::IaPd
            })
            .count();
        if dhcpv6_source_count == 0 {
            return Err(ServiceConfigError::InvalidConfig {
                reason: "SlaacDhcpv6 mode requires at least one DHCPv6 prefix source (Na or Pd)"
                    .to_string(),
            });
        }
        let dhcpv6 = self.dhcpv6.as_ref().ok_or(ServiceConfigError::InvalidConfig {
            reason: "SlaacDhcpv6 mode requires DHCPv6 configuration".to_string(),
        })?;
        if !dhcpv6.enable {
            return Err(ServiceConfigError::InvalidConfig {
                reason: "SlaacDhcpv6 mode requires DHCPv6 to be enabled".to_string(),
            });
        }
        dhcpv6.validate()?;
        Ok(())
    }

    pub fn sources_by_kind(&self, kind: SourceServiceKind) -> Vec<&LanIPv6SourceConfig> {
        self.sources.iter().filter(|s| s.service_kind() == kind).collect()
    }

    pub fn new(depend_iface: String) -> Self {
        let sources = vec![LanIPv6SourceConfig::RaPd {
            depend_iface,
            pool_index: 1,
            preferred_lifetime: 300,
            valid_lifetime: 300,
        }];
        Self {
            mode: IPv6ServiceMode::Slaac,
            sources,
            ra_flag: ra_flag_default(),
            ad_interval: 300,
            dhcpv6: None,
        }
    }
}

fn validate_source_entry(src: &LanIPv6SourceConfig) -> Result<(), ServiceConfigError> {
    match src {
        LanIPv6SourceConfig::PdStatic { base_prefix_len, pool_index, pool_len, .. } => {
            if *pool_len <= *base_prefix_len {
                return Err(ServiceConfigError::InvalidConfig {
                    reason: format!(
                        "PdStatic pool_len ({}) must be > base_prefix_len ({})",
                        pool_len, base_prefix_len
                    ),
                });
            }
            if *pool_len > 128 {
                return Err(ServiceConfigError::InvalidConfig {
                    reason: format!("PdStatic pool_len ({}) must be <= 128", pool_len),
                });
            }
            let max_blocks =
                1u64.checked_shl((*pool_len - *base_prefix_len) as u32).unwrap_or(u64::MAX);
            if (*pool_index as u64) >= max_blocks {
                return Err(ServiceConfigError::InvalidConfig {
                    reason: format!(
                        "PdStatic pool_index ({}) exceeds max blocks ({}) for base_prefix_len={}, pool_len={}",
                        pool_index, max_blocks, base_prefix_len, pool_len
                    ),
                });
            }
        }
        LanIPv6SourceConfig::PdPd { pool_len, .. } => {
            if *pool_len == 0 || *pool_len > 128 {
                return Err(ServiceConfigError::InvalidConfig {
                    reason: format!("PdPd pool_len ({}) must be between 1 and 128", pool_len),
                });
            }
        }
        _ => {}
    }
    Ok(())
}

pub fn validate_sources_no_conflict(
    sources: &[LanIPv6SourceConfig],
) -> Result<(), ServiceConfigError> {
    let mut groups: HashMap<ParentPrefixKey, Vec<&LanIPv6SourceConfig>> = HashMap::new();
    for src in sources {
        groups.entry(src.parent_key()).or_default().push(src);
    }

    for (key, group) in &groups {
        let len = group.len();
        for i in 0..len {
            for j in (i + 1)..len {
                check_pair_conflict(key, group[i], group[j])?;
            }
        }
    }

    Ok(())
}

fn check_pair_conflict(
    key: &ParentPrefixKey,
    a: &LanIPv6SourceConfig,
    b: &LanIPv6SourceConfig,
) -> Result<(), ServiceConfigError> {
    let kind_a = a.service_kind();
    let kind_b = b.service_kind();

    if (kind_a == SourceServiceKind::Ra && kind_b == SourceServiceKind::Na)
        || (kind_a == SourceServiceKind::Na && kind_b == SourceServiceKind::Ra)
    {
        return Ok(());
    }

    let parent_len = get_effective_parent_len(a, b);
    let idx_a = a.pool_index();
    let len_a = a.pool_len();
    let idx_b = b.pool_index();
    let len_b = b.pool_len();

    if blocks_overlap(parent_len, idx_a, len_a, idx_b, len_b) {
        return Err(ServiceConfigError::InvalidConfig {
            reason: format!(
                "Source conflict under parent {}: (pool_index={}, pool_len={}) overlaps with (pool_index={}, pool_len={})",
                key, idx_a, len_a, idx_b, len_b
            ),
        });
    }

    Ok(())
}

fn get_effective_parent_len(a: &LanIPv6SourceConfig, b: &LanIPv6SourceConfig) -> u8 {
    let len_a = match a {
        LanIPv6SourceConfig::PdStatic { base_prefix_len, .. } => Some(*base_prefix_len),
        LanIPv6SourceConfig::PdPd { max_source_prefix_len, .. } => Some(*max_source_prefix_len),
        _ => None,
    };
    let len_b = match b {
        LanIPv6SourceConfig::PdStatic { base_prefix_len, .. } => Some(*base_prefix_len),
        LanIPv6SourceConfig::PdPd { max_source_prefix_len, .. } => Some(*max_source_prefix_len),
        _ => None,
    };

    match (len_a, len_b) {
        (Some(a), Some(b)) => a.max(b),
        (Some(a), None) => a,
        (None, Some(b)) => b,
        (None, None) => 0,
    }
}

pub fn validate_cross_interface(
    new_config: &LanIPv6ServiceConfig,
    other_configs: &[LanIPv6ServiceConfig],
) -> Result<(), ServiceConfigError> {
    let new_iface = &new_config.iface_name;
    let new_sources = new_config.config.active_sources();

    for other in other_configs {
        if other.iface_name == *new_iface {
            continue;
        }
        if !other.enable {
            continue;
        }

        let other_active = other.config.active_sources();

        for new_src in &new_sources {
            for other_src in &other_active {
                if new_src.parent_key() != other_src.parent_key() {
                    continue;
                }
                if new_src.is_static() != other_src.is_static() {
                    continue;
                }
                let kind_new = new_src.service_kind();
                let kind_other = other_src.service_kind();
                if (kind_new == SourceServiceKind::Ra && kind_other == SourceServiceKind::Na)
                    || (kind_new == SourceServiceKind::Na && kind_other == SourceServiceKind::Ra)
                {
                    continue;
                }
                let parent_len = get_effective_parent_len(new_src, other_src);
                if blocks_overlap(
                    parent_len,
                    new_src.pool_index(),
                    new_src.pool_len(),
                    other_src.pool_index(),
                    other_src.pool_len(),
                ) {
                    return Err(ServiceConfigError::InvalidConfig {
                        reason: format!(
                            "Cross-interface conflict: {} source (pool_index={}, pool_len={}) on '{}' overlaps with (pool_index={}, pool_len={}) on '{}'",
                            new_src.parent_key(),
                            new_src.pool_index(),
                            new_src.pool_len(),
                            new_iface,
                            other_src.pool_index(),
                            other_src.pool_len(),
                            other.iface_name,
                        ),
                    });
                }
            }
        }
    }

    Ok(())
}

impl LandscapeStore for LanIPv6ServiceConfig {
    fn get_store_key(&self) -> String {
        self.iface_name.clone()
    }
}

impl ZoneAwareConfig for LanIPv6ServiceConfig {
    fn iface_name(&self) -> &str {
        &self.iface_name
    }
    fn zone_requirement() -> ZoneRequirement {
        ZoneRequirement::LanOnly
    }
    fn service_kind() -> ServiceKind {
        ServiceKind::LanIpv6
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_ra_na_same_index_ok() {
        let sources = vec![
            LanIPv6SourceConfig::RaStatic {
                base_prefix: "fd00::".parse().unwrap(),
                pool_index: 1,
                preferred_lifetime: 300,
                valid_lifetime: 600,
            },
            LanIPv6SourceConfig::NaStatic {
                base_prefix: "fd00::".parse().unwrap(),
                pool_index: 1,
            },
        ];
        assert!(validate_sources_no_conflict(&sources).is_ok());
    }

    #[test]
    fn static_ra_ra_same_index_conflict() {
        let sources = vec![
            LanIPv6SourceConfig::RaStatic {
                base_prefix: "fd00::".parse().unwrap(),
                pool_index: 1,
                preferred_lifetime: 300,
                valid_lifetime: 600,
            },
            LanIPv6SourceConfig::RaStatic {
                base_prefix: "fd00::".parse().unwrap(),
                pool_index: 1,
                preferred_lifetime: 300,
                valid_lifetime: 600,
            },
        ];
        assert!(validate_sources_no_conflict(&sources).is_err());
    }

    #[test]
    fn static_na_na_same_index_conflict() {
        let sources = vec![
            LanIPv6SourceConfig::NaStatic {
                base_prefix: "fd00::".parse().unwrap(),
                pool_index: 1,
            },
            LanIPv6SourceConfig::NaStatic {
                base_prefix: "fd00::".parse().unwrap(),
                pool_index: 1,
            },
        ];
        assert!(validate_sources_no_conflict(&sources).is_err());
    }

    #[test]
    fn static_ra_pd_block_overlap() {
        let sources = vec![
            LanIPv6SourceConfig::RaStatic {
                base_prefix: "fd00::".parse().unwrap(),
                pool_index: 1,
                preferred_lifetime: 300,
                valid_lifetime: 600,
            },
            LanIPv6SourceConfig::PdStatic {
                base_prefix: "fd00::".parse().unwrap(),
                base_prefix_len: 48,
                pool_index: 0,
                pool_len: 62,
            },
        ];
        assert!(validate_sources_no_conflict(&sources).is_err());
    }

    #[test]
    fn static_pd_same_block_conflict() {
        let sources = vec![
            LanIPv6SourceConfig::PdStatic {
                base_prefix: "fd00::".parse().unwrap(),
                base_prefix_len: 48,
                pool_index: 0,
                pool_len: 62,
            },
            LanIPv6SourceConfig::PdStatic {
                base_prefix: "fd00::".parse().unwrap(),
                base_prefix_len: 48,
                pool_index: 0,
                pool_len: 62,
            },
        ];
        assert!(validate_sources_no_conflict(&sources).is_err());
    }

    #[test]
    fn static_pd_adjacent_blocks_ok() {
        let sources = vec![
            LanIPv6SourceConfig::PdStatic {
                base_prefix: "fd00::".parse().unwrap(),
                base_prefix_len: 48,
                pool_index: 0,
                pool_len: 62,
            },
            LanIPv6SourceConfig::PdStatic {
                base_prefix: "fd00::".parse().unwrap(),
                base_prefix_len: 48,
                pool_index: 1,
                pool_len: 62,
            },
        ];
        assert!(validate_sources_no_conflict(&sources).is_ok());
    }

    #[test]
    fn pd_ra_na_same_index_ok() {
        let sources = vec![
            LanIPv6SourceConfig::RaPd {
                depend_iface: "eth0".to_string(),
                pool_index: 0,
                preferred_lifetime: 300,
                valid_lifetime: 600,
            },
            LanIPv6SourceConfig::NaPd { depend_iface: "eth0".to_string(), pool_index: 0 },
        ];
        assert!(validate_sources_no_conflict(&sources).is_ok());
    }

    #[test]
    fn pd_ra_index_inside_pd_block_conflict() {
        let sources = vec![
            LanIPv6SourceConfig::RaPd {
                depend_iface: "eth0".to_string(),
                pool_index: 2,
                preferred_lifetime: 300,
                valid_lifetime: 600,
            },
            LanIPv6SourceConfig::PdPd {
                depend_iface: "eth0".to_string(),
                max_source_prefix_len: 56,
                pool_index: 0,
                pool_len: 62,
            },
        ];
        assert!(validate_sources_no_conflict(&sources).is_err());
    }

    #[test]
    fn pd_pd_adjacent_blocks_ok() {
        let sources = vec![
            LanIPv6SourceConfig::PdPd {
                depend_iface: "eth0".to_string(),
                max_source_prefix_len: 56,
                pool_index: 0,
                pool_len: 62,
            },
            LanIPv6SourceConfig::PdPd {
                depend_iface: "eth0".to_string(),
                max_source_prefix_len: 56,
                pool_index: 1,
                pool_len: 62,
            },
        ];
        assert!(validate_sources_no_conflict(&sources).is_ok());
    }

    #[test]
    fn pd_pd_different_pool_len_overlap() {
        let sources = vec![
            LanIPv6SourceConfig::PdPd {
                depend_iface: "eth0".to_string(),
                max_source_prefix_len: 56,
                pool_index: 0,
                pool_len: 62,
            },
            LanIPv6SourceConfig::PdPd {
                depend_iface: "eth0".to_string(),
                max_source_prefix_len: 56,
                pool_index: 0,
                pool_len: 63,
            },
        ];
        assert!(validate_sources_no_conflict(&sources).is_err());
    }

    #[test]
    fn pd_pd_different_pool_len_no_overlap() {
        let sources = vec![
            LanIPv6SourceConfig::PdPd {
                depend_iface: "eth0".to_string(),
                max_source_prefix_len: 56,
                pool_index: 0,
                pool_len: 62,
            },
            LanIPv6SourceConfig::PdPd {
                depend_iface: "eth0".to_string(),
                max_source_prefix_len: 56,
                pool_index: 4,
                pool_len: 64,
            },
        ];
        assert!(validate_sources_no_conflict(&sources).is_ok());
    }

    #[test]
    fn static_diff_prefix_same_index_ok() {
        let sources = vec![
            LanIPv6SourceConfig::RaStatic {
                base_prefix: "fd00::".parse().unwrap(),
                pool_index: 1,
                preferred_lifetime: 300,
                valid_lifetime: 600,
            },
            LanIPv6SourceConfig::RaStatic {
                base_prefix: "2001:db8::".parse().unwrap(),
                pool_index: 1,
                preferred_lifetime: 300,
                valid_lifetime: 600,
            },
        ];
        assert!(validate_sources_no_conflict(&sources).is_ok());
    }

    #[test]
    fn pd_diff_iface_same_index_ok() {
        let sources = vec![
            LanIPv6SourceConfig::RaPd {
                depend_iface: "eth0".to_string(),
                pool_index: 1,
                preferred_lifetime: 300,
                valid_lifetime: 600,
            },
            LanIPv6SourceConfig::RaPd {
                depend_iface: "eth1".to_string(),
                pool_index: 1,
                preferred_lifetime: 300,
                valid_lifetime: 600,
            },
        ];
        assert!(validate_sources_no_conflict(&sources).is_ok());
    }

    #[test]
    fn empty_sources_ok() {
        assert!(validate_sources_no_conflict(&[]).is_ok());
    }

    #[test]
    fn single_entry_ok() {
        let sources = vec![LanIPv6SourceConfig::RaStatic {
            base_prefix: "fd00::".parse().unwrap(),
            pool_index: 0,
            preferred_lifetime: 300,
            valid_lifetime: 600,
        }];
        assert!(validate_sources_no_conflict(&sources).is_ok());
    }

    #[test]
    fn pool_len_not_greater_than_parent_len() {
        let src = LanIPv6SourceConfig::PdStatic {
            base_prefix: "fd00::".parse().unwrap(),
            base_prefix_len: 48,
            pool_index: 0,
            pool_len: 48,
        };
        assert!(validate_source_entry(&src).is_err());
    }

    #[test]
    fn pool_len_exceeds_128() {
        let src = LanIPv6SourceConfig::PdStatic {
            base_prefix: "fd00::".parse().unwrap(),
            base_prefix_len: 48,
            pool_index: 0,
            pool_len: 129,
        };
        assert!(validate_source_entry(&src).is_err());
    }

    #[test]
    fn pool_index_out_of_range() {
        let src = LanIPv6SourceConfig::PdStatic {
            base_prefix: "fd00::".parse().unwrap(),
            base_prefix_len: 48,
            pool_index: 16384,
            pool_len: 62,
        };
        assert!(validate_source_entry(&src).is_err());
    }

    fn make_service_config(iface: &str, sources: Vec<LanIPv6SourceConfig>) -> LanIPv6ServiceConfig {
        make_service_config_with_mode(iface, sources, IPv6ServiceMode::SlaacDhcpv6)
    }

    fn make_service_config_with_mode(
        iface: &str,
        sources: Vec<LanIPv6SourceConfig>,
        mode: IPv6ServiceMode,
    ) -> LanIPv6ServiceConfig {
        LanIPv6ServiceConfig {
            iface_name: iface.to_string(),
            enable: true,
            config: LanIPv6Config {
                mode,
                ad_interval: 300,
                ra_flag: ra_flag_default(),
                sources,
                dhcpv6: None,
            },
            update_at: 0.0,
        }
    }

    #[test]
    fn cross_iface_same_pd_same_index_conflict() {
        let new = make_service_config(
            "lan1",
            vec![LanIPv6SourceConfig::RaPd {
                depend_iface: "eth0".to_string(),
                pool_index: 0,
                preferred_lifetime: 300,
                valid_lifetime: 600,
            }],
        );
        let others = vec![make_service_config(
            "lan2",
            vec![LanIPv6SourceConfig::RaPd {
                depend_iface: "eth0".to_string(),
                pool_index: 0,
                preferred_lifetime: 300,
                valid_lifetime: 600,
            }],
        )];
        assert!(validate_cross_interface(&new, &others).is_err());
    }

    #[test]
    fn cross_iface_same_pd_diff_block_ok() {
        let new = make_service_config(
            "lan1",
            vec![LanIPv6SourceConfig::PdPd {
                depend_iface: "eth0".to_string(),
                max_source_prefix_len: 56,
                pool_index: 0,
                pool_len: 62,
            }],
        );
        let others = vec![make_service_config(
            "lan2",
            vec![LanIPv6SourceConfig::PdPd {
                depend_iface: "eth0".to_string(),
                max_source_prefix_len: 56,
                pool_index: 1,
                pool_len: 62,
            }],
        )];
        assert!(validate_cross_interface(&new, &others).is_ok());
    }

    #[test]
    fn cross_iface_diff_pd_same_index_ok() {
        let new = make_service_config(
            "lan1",
            vec![LanIPv6SourceConfig::RaPd {
                depend_iface: "eth0".to_string(),
                pool_index: 0,
                preferred_lifetime: 300,
                valid_lifetime: 600,
            }],
        );
        let others = vec![make_service_config(
            "lan2",
            vec![LanIPv6SourceConfig::RaPd {
                depend_iface: "eth1".to_string(),
                pool_index: 0,
                preferred_lifetime: 300,
                valid_lifetime: 600,
            }],
        )];
        assert!(validate_cross_interface(&new, &others).is_ok());
    }

    #[test]
    fn cross_iface_ra_static_vs_pd_static_overlap() {
        let new = make_service_config(
            "lan1",
            vec![LanIPv6SourceConfig::RaStatic {
                base_prefix: "fd00::".parse().unwrap(),
                pool_index: 1,
                preferred_lifetime: 300,
                valid_lifetime: 600,
            }],
        );
        let others = vec![make_service_config(
            "lan2",
            vec![LanIPv6SourceConfig::PdStatic {
                base_prefix: "fd00::".parse().unwrap(),
                base_prefix_len: 48,
                pool_index: 0,
                pool_len: 62,
            }],
        )];
        assert!(validate_cross_interface(&new, &others).is_err());
    }

    #[test]
    fn test_blocks_overlap_same_index_same_len() {
        assert!(blocks_overlap(48, 0, 64, 0, 64));
    }

    #[test]
    fn test_blocks_overlap_adjacent() {
        assert!(!blocks_overlap(48, 0, 64, 1, 64));
    }

    #[test]
    fn test_blocks_overlap_nested() {
        assert!(blocks_overlap(48, 0, 62, 2, 64));
    }

    #[test]
    fn test_blocks_no_overlap_different_sizes() {
        assert!(!blocks_overlap(48, 0, 62, 4, 64));
    }
}
