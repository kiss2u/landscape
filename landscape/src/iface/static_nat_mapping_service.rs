use std::net::{Ipv4Addr, Ipv6Addr};

use landscape_common::error::LdError;
use landscape_common::{
    iface::nat::{StaticMapPair, StaticNatMappingConfig, StaticNatTarget},
    service::controller::ConfigController,
    utils::time::get_f64_timestamp,
    LANDSCAPE_DEFAULE_DHCP_V4_CLIENT_PORT, LANDSCAPE_DEFAULE_DHCP_V6_CLIENT_PORT,
};
use landscape_database::{
    provider::LandscapeDBServiceProvider,
    static_nat_mapping::repository::StaticNatMappingConfigRepository,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct StaticNatMappingService {
    store: StaticNatMappingConfigRepository,
}

impl StaticNatMappingService {
    pub async fn new(store: LandscapeDBServiceProvider) -> Self {
        let static_nat_config_service = Self { store: store.static_nat_mapping_store() };

        if static_nat_config_service.list().await.is_empty() {
            static_nat_config_service.set_list(default_static_mapping_rule()).await;
        }

        static_nat_config_service.refresh_runtime_rules().await;
        static_nat_config_service
    }
}

#[async_trait::async_trait]
impl ConfigController for StaticNatMappingService {
    type Id = Uuid;

    type Config = StaticNatMappingConfig;

    type DatabseAction = StaticNatMappingConfigRepository;

    fn get_repository(&self) -> &Self::DatabseAction {
        &self.store
    }

    async fn after_update_config(
        &self,
        _new_rules: Vec<Self::Config>,
        _old_rules: Vec<Self::Config>,
    ) {
        self.refresh_runtime_rules().await;
    }
}

impl StaticNatMappingService {
    pub async fn refresh_runtime_rules(&self) {
        let runtime_configs = match self.store.list_runtime_configs().await {
            Ok(runtime_configs) => runtime_configs,
            Err(error) => {
                tracing::error!("failed to load static NAT runtime configs: {error:?}");
                return;
            }
        };

        if let Err(error) =
            landscape_ebpf::map_setting::nat::reconcile_static_nat4_map(&runtime_configs)
        {
            tracing::error!("failed to reconcile static NAT v4 map: {error:?}");
        }

        if let Err(error) =
            landscape_ebpf::map_setting::nat::reconcile_static_nat6_map(&runtime_configs)
        {
            tracing::error!("failed to reconcile static NAT v6 map: {error:?}");
        }
    }

    pub async fn validate_runtime_target(
        &self,
        config: &StaticNatMappingConfig,
    ) -> Result<(), LdError> {
        self.store.validate_runtime_target(config).await
    }
}

pub fn default_static_mapping_rule() -> Vec<StaticNatMappingConfig> {
    let mut result = Vec::with_capacity(5);
    // DHCPv4 Clinet
    result.push(StaticNatMappingConfig {
        wan_iface_name: None,
        lan_target: Some(StaticNatTarget::address(Some(Ipv4Addr::UNSPECIFIED), None)),
        ipv4_l4_protocol: vec![17],
        ipv6_l4_protocol: vec![],
        id: Uuid::new_v4(),
        enable: true,
        remark: "Default DHCPv4 Client Port".to_string(),
        update_at: get_f64_timestamp(),
        mapping_pair_ports: vec![StaticMapPair {
            wan_port: LANDSCAPE_DEFAULE_DHCP_V4_CLIENT_PORT,
            lan_port: LANDSCAPE_DEFAULE_DHCP_V4_CLIENT_PORT,
        }],
    });
    // DHCPv6 Clinet
    result.push(StaticNatMappingConfig {
        wan_iface_name: None,
        lan_target: Some(StaticNatTarget::address(None, Some(Ipv6Addr::UNSPECIFIED))),
        ipv4_l4_protocol: vec![],
        ipv6_l4_protocol: vec![17],
        id: Uuid::new_v4(),
        enable: true,
        remark: "Default DHCPv6 Client Port".to_string(),
        update_at: get_f64_timestamp(),
        mapping_pair_ports: vec![StaticMapPair {
            wan_port: LANDSCAPE_DEFAULE_DHCP_V6_CLIENT_PORT,
            lan_port: LANDSCAPE_DEFAULE_DHCP_V6_CLIENT_PORT,
        }],
    });
    #[cfg(debug_assertions)]
    {
        result.push(StaticNatMappingConfig {
            wan_iface_name: None,
            lan_target: Some(StaticNatTarget::address(Some(Ipv4Addr::UNSPECIFIED), None)),
            ipv4_l4_protocol: vec![6, 17],
            ipv6_l4_protocol: vec![],
            id: Uuid::new_v4(),
            enable: true,
            remark: "For Test".to_string(),
            update_at: get_f64_timestamp(),
            mapping_pair_ports: vec![StaticMapPair { wan_port: 8080, lan_port: 8081 }],
        });

        result.push(StaticNatMappingConfig {
            wan_iface_name: None,
            lan_target: Some(StaticNatTarget::address(Some(Ipv4Addr::UNSPECIFIED), None)),
            ipv4_l4_protocol: vec![6],
            ipv6_l4_protocol: vec![],
            id: Uuid::new_v4(),
            enable: true,
            remark: "".to_string(),
            update_at: get_f64_timestamp(),
            mapping_pair_ports: vec![StaticMapPair { wan_port: 5173, lan_port: 5173 }],
        });

        result.push(StaticNatMappingConfig {
            wan_iface_name: None,
            lan_target: Some(StaticNatTarget::address(Some(Ipv4Addr::UNSPECIFIED), None)),
            ipv4_l4_protocol: vec![6],
            ipv6_l4_protocol: vec![],
            id: Uuid::new_v4(),
            enable: true,
            remark: "".to_string(),
            update_at: get_f64_timestamp(),
            mapping_pair_ports: vec![StaticMapPair { wan_port: 22, lan_port: 22 }],
        });
    }
    result
}
