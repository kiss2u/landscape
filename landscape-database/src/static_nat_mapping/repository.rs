use std::collections::{HashMap, HashSet};
use std::net::{Ipv4Addr, Ipv6Addr};

use landscape_common::enrolled_device::EnrolledDevice;
use landscape_common::error::LdError;
use landscape_common::iface::nat::{
    RuntimeStaticNatMappingConfig, StaticNatMappingConfig, StaticNatTarget,
};
use landscape_common::ipv6::lan::{
    LanIPv6ServiceConfigV2, LanPrefixGroupConfig, PrefixParentSource,
};
use landscape_common::ipv6::{checked_allocate_subnet, checked_combine_ipv6_prefix_suffix};
use sea_orm::DatabaseConnection;

use super::entity::{
    StaticNatMappingConfigActiveModel, StaticNatMappingConfigEntity, StaticNatMappingConfigModel,
};
use crate::enrolled_device::repository::EnrolledDeviceRepository;
use crate::lan_ipv6_v2::repository::LanIPv6V2ServiceRepository;
use crate::repository::Repository;
use crate::DBId;

#[derive(Clone)]
pub struct StaticNatMappingConfigRepository {
    db: DatabaseConnection,
}

impl StaticNatMappingConfigRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn list_runtime_configs(
        &self,
    ) -> Result<Vec<RuntimeStaticNatMappingConfig>, LdError> {
        let configs = self.list_all().await?;
        let devices = self.load_devices_for_configs(&configs).await?;

        let has_device_target = configs.iter().any(|config| {
            matches!(config.lan_target.as_ref(), Some(StaticNatTarget::Device { .. }))
        });
        let lan_ipv6_configs = if has_device_target {
            LanIPv6V2ServiceRepository::new(self.db.clone())
                .list_all()
                .await?
                .into_iter()
                .map(|config| (config.iface_name.clone(), config))
                .collect()
        } else {
            HashMap::new()
        };

        Ok(configs
            .into_iter()
            .filter(|config| config.enable)
            .map(|config| resolve_static_nat_mapping_config(config, &devices, &lan_ipv6_configs))
            .collect())
    }

    async fn load_devices_for_configs(
        &self,
        configs: &[StaticNatMappingConfig],
    ) -> Result<HashMap<DBId, EnrolledDevice>, LdError> {
        let mut device_ids = HashSet::new();
        for config in configs {
            if let Some(StaticNatTarget::Device { device_id }) = config.lan_target.as_ref() {
                device_ids.insert(*device_id);
            }
        }

        let devices = EnrolledDeviceRepository::new(self.db.clone())
            .find_by_ids(device_ids.into_iter().collect())
            .await;
        Ok(devices.into_iter().map(|device| (device.id, device)).collect())
    }

    pub async fn validate_runtime_target(
        &self,
        config: &StaticNatMappingConfig,
    ) -> Result<(), LdError> {
        let devices = self.load_devices_for_configs(std::slice::from_ref(config)).await?;
        let lan_ipv6_configs = if matches!(config.lan_target, Some(StaticNatTarget::Device { .. }))
        {
            LanIPv6V2ServiceRepository::new(self.db.clone())
                .list_all()
                .await?
                .into_iter()
                .map(|lan_config| (lan_config.iface_name.clone(), lan_config))
                .collect()
        } else {
            HashMap::new()
        };

        let (lan_ipv4, lan_ipv6) = resolve_static_nat_target(config, &devices, &lan_ipv6_configs);

        if config.enable && !config.ipv4_l4_protocol.is_empty() && lan_ipv4.is_none() {
            return Err(LdError::ConfigError(
                "enabled IPv4 static NAT mapping must resolve to an IPv4 target".to_string(),
            ));
        }

        if config.enable && !config.ipv6_l4_protocol.is_empty() && lan_ipv6.is_none() {
            return Err(LdError::ConfigError(
                "enabled IPv6 static NAT mapping must resolve to an IPv6 target".to_string(),
            ));
        }

        Ok(())
    }
}

fn resolve_static_nat_mapping_config(
    config: StaticNatMappingConfig,
    devices: &HashMap<DBId, EnrolledDevice>,
    lan_ipv6_configs: &HashMap<String, LanIPv6ServiceConfigV2>,
) -> RuntimeStaticNatMappingConfig {
    let (lan_ipv4, lan_ipv6) = resolve_static_nat_target(&config, devices, lan_ipv6_configs);
    RuntimeStaticNatMappingConfig {
        mapping_pair_ports: config.mapping_pair_ports,
        lan_ipv4,
        lan_ipv6,
        ipv4_l4_protocol: config.ipv4_l4_protocol,
        ipv6_l4_protocol: config.ipv6_l4_protocol,
    }
}

fn resolve_static_nat_target(
    config: &StaticNatMappingConfig,
    devices: &HashMap<DBId, EnrolledDevice>,
    lan_ipv6_configs: &HashMap<String, LanIPv6ServiceConfigV2>,
) -> (Option<Ipv4Addr>, Option<Ipv6Addr>) {
    match config.lan_target.as_ref() {
        Some(StaticNatTarget::Address { ipv4, ipv6 }) => (*ipv4, *ipv6),
        Some(StaticNatTarget::Local) => (Some(Ipv4Addr::UNSPECIFIED), Some(Ipv6Addr::UNSPECIFIED)),
        Some(StaticNatTarget::Device { device_id }) => {
            let Some(device) = devices.get(device_id) else {
                tracing::warn!("static NAT device target unresolved: device {device_id} not found");
                return (None, None);
            };
            let ipv6 = resolve_device_ipv6(device, lan_ipv6_configs);
            (device.ipv4, ipv6)
        }
        None => (None, None),
    }
}

fn resolve_device_ipv6(
    device: &EnrolledDevice,
    lan_ipv6_configs: &HashMap<String, LanIPv6ServiceConfigV2>,
) -> Option<Ipv6Addr> {
    let device_ipv6 = device.ipv6?;
    let iface_name = device.iface_name.as_ref()?;
    let config = lan_ipv6_configs.get(iface_name)?;
    let group = select_device_ipv6_group(&config.config.prefix_groups)?;
    // Static NA groups reuse the device host part under the current /64.
    // PD-backed groups treat device.ipv6 as the final runtime address.
    match &group.parent {
        PrefixParentSource::Static { base_prefix, parent_prefix_len } => {
            let pool_index = group.na.as_ref().map(|na| na.pool_index)?;
            let (prefix, _) =
                checked_allocate_subnet(*base_prefix, *parent_prefix_len, 64, pool_index as u128)?;
            checked_combine_ipv6_prefix_suffix(prefix, 64, device_ipv6)
        }
        PrefixParentSource::Pd { .. } => Some(device_ipv6),
    }
}

fn select_device_ipv6_group(groups: &[LanPrefixGroupConfig]) -> Option<&LanPrefixGroupConfig> {
    let mut candidates = groups.iter().filter(|group| group.na.is_some());
    let candidate = candidates.next()?;
    if candidates.next().is_some() {
        tracing::warn!("static NAT device target unresolved: multiple IPv6 NA pools found");
        return None;
    }
    Some(candidate)
}

crate::impl_repository!(
    StaticNatMappingConfigRepository,
    StaticNatMappingConfigModel,
    StaticNatMappingConfigEntity,
    StaticNatMappingConfigActiveModel,
    StaticNatMappingConfig,
    DBId
);
