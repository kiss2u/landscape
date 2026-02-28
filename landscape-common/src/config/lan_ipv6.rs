use std::collections::HashMap;
use std::fmt;
use std::net::Ipv6Addr;

use serde::{Deserialize, Serialize};

use crate::database::repository::LandscapeDBStore;
use crate::dhcp::v6_server::config::DHCPv6ServerConfig;
use crate::service::ServiceConfigError;
use crate::store::storev2::LandscapeStore;
use crate::utils::time::get_f64_timestamp;

use super::ra::RouterFlags;

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

/// Service kind of a source entry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum SourceServiceKind {
    Ra,
    Na,
    IaPd,
}

/// Parent prefix key for grouping sources during conflict detection
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParentPrefixKey {
    /// Static prefix identified by base address
    Static(Ipv6Addr),
    /// PD prefix identified by upstream interface name
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

/// Flat 6-variant enum representing all source configurations for LAN IPv6 services.
/// Combines service type (Ra/Na/Pd) and source type (Static/Pd) into a single flat enum.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(tag = "t", rename_all = "snake_case")]
pub enum LanIPv6SourceConfig {
    /// RA + static prefix (pool_len implicit = 64)
    RaStatic {
        #[cfg_attr(feature = "openapi", schema(value_type = String))]
        base_prefix: Ipv6Addr,
        pool_index: u32,
        preferred_lifetime: u32,
        valid_lifetime: u32,
    },
    /// RA + upstream PD (pool_len implicit = 64)
    RaPd { depend_iface: String, pool_index: u32, preferred_lifetime: u32, valid_lifetime: u32 },
    /// DHCPv6 IA_NA + static prefix (pool_len implicit = 64)
    NaStatic {
        #[cfg_attr(feature = "openapi", schema(value_type = String))]
        base_prefix: Ipv6Addr,
        pool_index: u32,
    },
    /// DHCPv6 IA_NA + upstream PD (pool_len implicit = 64)
    NaPd { depend_iface: String, pool_index: u32 },
    /// DHCPv6 IA_PD + static parent prefix
    PdStatic {
        #[cfg_attr(feature = "openapi", schema(value_type = String))]
        base_prefix: Ipv6Addr,
        base_prefix_len: u8,
        pool_index: u32,
        pool_len: u8,
    },
    /// DHCPv6 IA_PD + upstream PD
    PdPd { depend_iface: String, max_source_prefix_len: u8, pool_index: u32, pool_len: u8 },
}

impl LanIPv6SourceConfig {
    /// Parent prefix key for grouping during conflict detection
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

    /// Service kind of this source entry
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

    /// Pool index within the parent prefix
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

    /// Pool length: RA/NA always 64, PD uses the variant's pool_len
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

    /// Whether this is a static source (vs PD)
    pub fn is_static(&self) -> bool {
        matches!(
            self,
            LanIPv6SourceConfig::RaStatic { .. }
                | LanIPv6SourceConfig::NaStatic { .. }
                | LanIPv6SourceConfig::PdStatic { .. }
        )
    }

    /// Get the base_prefix for static variants
    pub fn base_prefix(&self) -> Option<Ipv6Addr> {
        match self {
            LanIPv6SourceConfig::RaStatic { base_prefix, .. }
            | LanIPv6SourceConfig::NaStatic { base_prefix, .. }
            | LanIPv6SourceConfig::PdStatic { base_prefix, .. } => Some(*base_prefix),
            _ => None,
        }
    }

    /// Get the depend_iface for PD variants
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
    /// All prefix sources (flat list, replaces old source + dhcpv6.source)
    #[serde(default)]
    pub sources: Vec<LanIPv6SourceConfig>,
    /// DHCPv6 server config (parameters only, sources come from `sources` field)
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(required = false, nullable = false))]
    pub dhcpv6: Option<DHCPv6ServerConfig>,
}

fn ra_flag_default() -> RouterFlags {
    0xc0.into()
}

/// Check if an IPv6 address is in the ULA range (fc00::/7)
fn is_ula(addr: Ipv6Addr) -> bool {
    let first_byte = addr.octets()[0];
    (first_byte & 0xfe) == 0xfc
}

/// Check whether two pool blocks overlap.
/// A block at (pool_index, pool_len) within a parent prefix of parent_len
/// occupies a range of /64 slots.
///
/// For RA/NA (pool_len=64): occupies exactly one /64 slot at pool_index.
/// For PD (pool_len < 64): occupies 2^(64 - pool_len) /64 slots starting at pool_index * 2^(64 - pool_len).
///
/// We normalize all blocks to /64 slot ranges and check for overlap.
fn blocks_overlap(_parent_prefix_len: u8, idx_a: u32, len_a: u8, idx_b: u32, len_b: u8) -> bool {
    // Each block occupies a range in the parent prefix's address space.
    // Normalize to a common granularity: the finer of the two pool_lens.
    // Block A starts at idx_a * (2^(max_len - len_a)) and has size 2^(max_len - len_a).
    // Block B starts at idx_b * (2^(max_len - len_b)) and has size 2^(max_len - len_b).
    // We use the parent_prefix_len as the base.

    // The number of slots of size pool_len within the parent:
    // slot_count = 2^(pool_len - parent_prefix_len)
    // A block at pool_index occupies [pool_index, pool_index+1) in units of pool_len.
    // To compare across different pool_lens, normalize to the finest granularity (largest pool_len = 128 theoretical, but practically 64 for RA/NA).

    // Use the finer granularity (larger pool_len value = smaller blocks)
    let max_len = len_a.max(len_b);

    // Block A in units of max_len
    let scale_a = 1u64 << (max_len - len_a) as u64;
    let start_a = (idx_a as u64) * scale_a;
    let end_a = start_a + scale_a;

    // Block B in units of max_len
    let scale_b = 1u64 << (max_len - len_b) as u64;
    let start_b = (idx_b as u64) * scale_b;
    let end_b = start_b + scale_b;

    // Overlap check
    start_a < end_b && start_b < end_a
}

impl LanIPv6Config {
    pub fn validate(&self) -> Result<(), ServiceConfigError> {
        // Validate individual source entries
        for src in &self.sources {
            validate_source_entry(src)?;
        }

        // Only validate conflicts among sources that are ACTIVE in the current mode.
        // Inactive sources (e.g. RA sources in stateful mode) are dormant and should
        // not trigger conflicts — this allows users to switch modes without reconfiguring.
        let active_sources = self.active_sources();
        validate_sources_no_conflict(&active_sources)?;

        match self.mode {
            IPv6ServiceMode::Slaac => self.validate_slaac(),
            IPv6ServiceMode::Stateful => self.validate_stateful(),
            IPv6ServiceMode::SlaacDhcpv6 => self.validate_slaac_dhcpv6(),
        }
    }

    /// Returns only the sources that are active under the current mode.
    /// - Slaac: Ra sources only
    /// - Stateful: Na + IaPd sources only
    /// - SlaacDhcpv6: Ra + Na + IaPd (all)
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
        // Must have at least one RA source
        let ra_count =
            self.sources.iter().filter(|s| s.service_kind() == SourceServiceKind::Ra).count();
        if ra_count == 0 {
            return Err(ServiceConfigError::InvalidConfig {
                reason: "Slaac mode requires at least one RA prefix source".to_string(),
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
        Ok(())
    }

    fn validate_stateful(&self) -> Result<(), ServiceConfigError> {
        // M=1, O=1
        if !self.ra_flag.managed_address_config || !self.ra_flag.other_config {
            return Err(ServiceConfigError::InvalidConfig {
                reason: "Stateful mode requires M=1 and O=1".to_string(),
            });
        }
        // Must have at least one Na source
        let na_count =
            self.sources.iter().filter(|s| s.service_kind() == SourceServiceKind::Na).count();
        if na_count == 0 {
            return Err(ServiceConfigError::InvalidConfig {
                reason: "Stateful mode requires at least one DHCPv6 NA prefix source".to_string(),
            });
        }
        // DHCPv6 must be enabled
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
        // M=1, O=1
        if !self.ra_flag.managed_address_config || !self.ra_flag.other_config {
            return Err(ServiceConfigError::InvalidConfig {
                reason: "SlaacDhcpv6 mode requires M=1 and O=1".to_string(),
            });
        }
        // Must have at least one RA source (must be static ULA)
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
        // Must have at least one Na or Pd source for DHCPv6
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
        // DHCPv6 must be enabled
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

    /// Filter sources by service kind
    pub fn sources_by_kind(&self, kind: SourceServiceKind) -> Vec<&LanIPv6SourceConfig> {
        self.sources.iter().filter(|s| s.service_kind() == kind).collect()
    }
}

/// Validate a single source entry for parameter correctness
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
            // pool_len must be > 0 and <= 128
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

/// Validate that sources within a single interface do not conflict
pub fn validate_sources_no_conflict(
    sources: &[LanIPv6SourceConfig],
) -> Result<(), ServiceConfigError> {
    // Group by parent key
    let mut groups: HashMap<ParentPrefixKey, Vec<&LanIPv6SourceConfig>> = HashMap::new();
    for src in sources {
        groups.entry(src.parent_key()).or_default().push(src);
    }

    // Within each group, check for conflicts
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

/// Check if two sources under the same parent key conflict.
///
/// RA and NA are allowed to share the same /64 subnet (same pool_index):
/// RA advertises the prefix for SLAAC, while NA assigns addresses via DHCPv6
/// within the same subnet. This is the expected usage in slaac_dhcpv6 mode.
///
/// Real conflicts are:
/// - RA vs RA (duplicate prefix announcement)
/// - NA vs NA (duplicate address pool)
/// - RA/NA /64 slot falling inside a PD delegation pool range
/// - PD vs PD pool block overlap
fn check_pair_conflict(
    key: &ParentPrefixKey,
    a: &LanIPv6SourceConfig,
    b: &LanIPv6SourceConfig,
) -> Result<(), ServiceConfigError> {
    let kind_a = a.service_kind();
    let kind_b = b.service_kind();

    // RA and NA can share the same /64 subnet — skip conflict check
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

/// Get effective parent prefix len for overlap calculation.
/// For PdStatic, use base_prefix_len.
/// For PdPd, use max_source_prefix_len as lower bound.
/// For RA/NA static or PD sources, the parent_prefix_len doesn't
/// matter for /64 vs /64 comparisons (just comparing pool_index),
/// but matters for /64 vs PD block comparisons.
fn get_effective_parent_len(a: &LanIPv6SourceConfig, b: &LanIPv6SourceConfig) -> u8 {
    // Try to extract a concrete parent len from PdStatic
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

    // Use the most specific (largest) parent prefix len, or default to 0
    // for pure RA/NA comparisons where we just compare /64 indices
    match (len_a, len_b) {
        (Some(a), Some(b)) => a.max(b),
        (Some(a), None) => a,
        (None, Some(b)) => b,
        (None, None) => 0, // Both are RA/NA with pool_len=64, overlap check still works
    }
}

/// Validate cross-interface conflicts: check new config against all other interfaces
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
                // Only compare sources with the same parent key
                if new_src.parent_key() != other_src.parent_key() {
                    continue;
                }
                // Only detect conflicts between same-type sources (static vs static, pd vs pd)
                // Cross-type (static vs pd) conflicts are detected at runtime
                if new_src.is_static() != other_src.is_static() {
                    continue;
                }
                // RA and NA can share the same /64 subnet (even across interfaces)
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
                            "Cross-interface conflict: {} source (pool_index={}, pool_len={}) on '{}' \
                             overlaps with (pool_index={}, pool_len={}) on '{}'",
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

impl LanIPv6Config {
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

#[cfg(test)]
mod tests {
    use super::*;

    // === Same interface — static prefix conflicts ===

    #[test]
    fn static_ra_na_same_index_ok() {
        // RA and NA can share the same /64 subnet:
        // RA advertises the prefix for SLAAC, NA assigns addresses via DHCPv6
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
        // Two RA sources on the same prefix + index is a real conflict
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
        // Two NA sources on the same prefix + index is a real conflict
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
        // RaStatic pool_index=1 (/64 slot 1) vs PdStatic pool_index=0, pool_len=62
        // A /62 block at index 0 within a /48 covers /64 slots 0..3
        // So slot 1 falls within block 0
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

    // === Same interface — dynamic PD conflicts ===

    #[test]
    fn pd_ra_na_same_index_ok() {
        // RA and NA can share the same /64 subnet from PD source
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
        // RaPd pool_index=2 (/64 slot 2) vs PdPd pool_index=0, pool_len=62
        // /62 block 0 covers /64 slots 0..3, so slot 2 is inside
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

    // === Different pool_len overlap / non-overlap ===

    #[test]
    fn pd_pd_different_pool_len_overlap() {
        // /62 block 0 covers slots 0..3; /63 block 0 covers slots 0..1 → overlap
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
        // /62 block 0 covers /64 slots 0..3; /64 slot 4 is outside
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

    // === Different parent prefixes — no conflict ===

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

    // === Edge cases ===

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

    // === Parameter validation ===

    #[test]
    fn pool_len_not_greater_than_parent_len() {
        let src = LanIPv6SourceConfig::PdStatic {
            base_prefix: "fd00::".parse().unwrap(),
            base_prefix_len: 48,
            pool_index: 0,
            pool_len: 48, // must be > 48
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
        // /48 parent, /62 pool_len → 2^(62-48) = 16384 blocks max
        let src = LanIPv6SourceConfig::PdStatic {
            base_prefix: "fd00::".parse().unwrap(),
            base_prefix_len: 48,
            pool_index: 16384, // exactly at limit, should fail
            pool_len: 62,
        };
        assert!(validate_source_entry(&src).is_err());
    }

    // === Cross-interface tests ===

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
        // Both are static, same parent key, slot 1 falls in /62 block 0
        assert!(validate_cross_interface(&new, &others).is_err());
    }

    // === blocks_overlap unit tests ===

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
        // /62 block 0 = slots 0..3, /64 slot 2 is inside
        assert!(blocks_overlap(48, 0, 62, 2, 64));
    }

    #[test]
    fn test_blocks_no_overlap_different_sizes() {
        // /62 block 0 = slots 0..3, /64 slot 4 is outside
        assert!(!blocks_overlap(48, 0, 62, 4, 64));
    }
}
