use core::ops::Range;
use landscape_macro::LdApiError;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use uuid::Uuid;

use crate::config::ConfigId;
use crate::database::repository::LandscapeDBStore;
use crate::iface::config::{ServiceKind, ZoneAwareConfig, ZoneRequirement};
use crate::service::ServiceConfigError;
use crate::store::storev2::LandscapeStore;
use crate::utils::id::gen_database_uuid;
use crate::utils::time::get_f64_timestamp;

#[derive(thiserror::Error, Debug, LdApiError)]
#[api_error(crate_path = "crate")]
pub enum StaticNatError {
    #[error("Static NAT mapping '{0}' not found")]
    #[api_error(id = "static_nat.not_found", status = 404)]
    NotFound(ConfigId),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct NatServiceConfig {
    pub iface_name: String,
    pub enable: bool,
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(required = true))]
    pub nat_config: NatConfig,
    #[serde(default = "get_f64_timestamp")]
    #[cfg_attr(feature = "openapi", schema(required = false))]
    pub update_at: f64,
}

impl LandscapeStore for NatServiceConfig {
    fn get_store_key(&self) -> String {
        self.iface_name.clone()
    }
}

impl LandscapeDBStore<String> for NatServiceConfig {
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

impl ZoneAwareConfig for NatServiceConfig {
    fn iface_name(&self) -> &str {
        &self.iface_name
    }
    fn zone_requirement() -> ZoneRequirement {
        ZoneRequirement::WanOrPpp
    }
    fn service_kind() -> ServiceKind {
        ServiceKind::NAT
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct NatConfig {
    #[cfg_attr(feature = "openapi", schema(value_type = Object))]
    pub tcp_range: Range<u16>,
    #[cfg_attr(feature = "openapi", schema(value_type = Object))]
    pub udp_range: Range<u16>,
    #[cfg_attr(feature = "openapi", schema(value_type = Object))]
    pub icmp_in_range: Range<u16>,
}

impl NatConfig {
    fn validate_range(name: &str, range: &Range<u16>) -> Result<(), ServiceConfigError> {
        if range.start == 0 {
            return Err(ServiceConfigError::InvalidConfig {
                reason: format!("{name} start port must be > 0"),
            });
        }
        if range.start >= range.end {
            return Err(ServiceConfigError::InvalidConfig {
                reason: format!(
                    "{name} start ({}) must be less than end ({})",
                    range.start, range.end
                ),
            });
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<(), ServiceConfigError> {
        Self::validate_range("tcp_range", &self.tcp_range)?;
        Self::validate_range("udp_range", &self.udp_range)?;
        Self::validate_range("icmp_in_range", &self.icmp_in_range)?;
        Ok(())
    }
}

impl Default for NatConfig {
    fn default() -> Self {
        Self {
            tcp_range: 32768..65535,
            udp_range: 32768..65535,
            icmp_in_range: 32768..65535,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct StaticMapPair {
    pub wan_port: u16,
    pub lan_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct StaticNatMappingConfig {
    #[serde(default = "gen_database_uuid")]
    #[cfg_attr(feature = "openapi", schema(required = false))]
    pub id: Uuid,
    pub enable: bool,
    pub remark: String,
    #[cfg_attr(feature = "openapi", schema(required = true, nullable = true))]
    pub wan_iface_name: Option<String>,
    pub mapping_pair_ports: Vec<StaticMapPair>,
    #[cfg_attr(feature = "openapi", schema(required = true, value_type = Option<String>))]
    pub lan_ipv4: Option<Ipv4Addr>,
    #[cfg_attr(feature = "openapi", schema(required = true, value_type = Option<String>))]
    pub lan_ipv6: Option<Ipv6Addr>,
    pub ipv4_l4_protocol: Vec<u8>,
    pub ipv6_l4_protocol: Vec<u8>,
    #[serde(default = "get_f64_timestamp")]
    #[cfg_attr(feature = "openapi", schema(required = false))]
    pub update_at: f64,
}

impl StaticNatMappingConfig {
    pub fn validate(&self) -> Result<(), ServiceConfigError> {
        if self.enable && self.mapping_pair_ports.is_empty() {
            return Err(ServiceConfigError::InvalidConfig {
                reason: "mapping_pair_ports must not be empty when enabled".to_string(),
            });
        }

        for (i, pair) in self.mapping_pair_ports.iter().enumerate() {
            if pair.wan_port == 0 {
                return Err(ServiceConfigError::InvalidConfig {
                    reason: format!("mapping_pair_ports[{i}].wan_port must not be 0"),
                });
            }
            if pair.lan_port == 0 {
                return Err(ServiceConfigError::InvalidConfig {
                    reason: format!("mapping_pair_ports[{i}].lan_port must not be 0"),
                });
            }
        }

        for (i, &proto) in self.ipv4_l4_protocol.iter().enumerate() {
            if proto != 6 && proto != 17 {
                return Err(ServiceConfigError::InvalidConfig {
                    reason: format!("ipv4_l4_protocol[{i}] ({proto}) must be 6 (TCP) or 17 (UDP)"),
                });
            }
        }
        for (i, &proto) in self.ipv6_l4_protocol.iter().enumerate() {
            if proto != 6 && proto != 17 {
                return Err(ServiceConfigError::InvalidConfig {
                    reason: format!("ipv6_l4_protocol[{i}] ({proto}) must be 6 (TCP) or 17 (UDP)"),
                });
            }
        }

        Ok(())
    }

    pub fn convert_to_item(&self) -> Vec<StaticNatMappingItem> {
        let mut result = Vec::with_capacity(4);
        for l4_protocol in &self.ipv4_l4_protocol {
            if let Some(ipv4) = self.lan_ipv4 {
                let items = self.mapping_pair_ports.iter().map(|pair_port| StaticNatMappingItem {
                    wan_port: pair_port.wan_port,
                    wan_iface_name: self.wan_iface_name.clone(),
                    lan_port: pair_port.lan_port,
                    lan_ip: IpAddr::V4(ipv4),
                    l4_protocol: *l4_protocol,
                });
                result.extend(items);
            }
        }

        for l4_protocol in &self.ipv6_l4_protocol {
            if let Some(ipv6) = self.lan_ipv6 {
                let items = self.mapping_pair_ports.iter().map(|pair_port| StaticNatMappingItem {
                    wan_port: pair_port.wan_port,
                    wan_iface_name: self.wan_iface_name.clone(),
                    lan_port: pair_port.lan_port,
                    lan_ip: IpAddr::V6(ipv6),
                    l4_protocol: *l4_protocol,
                });

                result.extend(items);
            }
        }
        result
    }
}

impl LandscapeDBStore<Uuid> for StaticNatMappingConfig {
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

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct StaticNatMappingItem {
    pub wan_port: u16,
    pub wan_iface_name: Option<String>,
    pub lan_port: u16,
    pub lan_ip: IpAddr,
    pub l4_protocol: u8,
}
