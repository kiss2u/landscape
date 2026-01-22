use std::{
    collections::{HashMap, HashSet},
    net::{Ipv4Addr, Ipv6Addr},
};

use landscape_common::{
    config::nat::{StaticMapPair, StaticNatMappingConfig, StaticNatMappingItem},
    service::controller_service_v2::ConfigController,
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
        let store = store.static_nat_mapping_store();
        let static_nat_config_service = Self { store };

        let mut rules = static_nat_config_service.list().await;

        if rules.is_empty() {
            static_nat_config_service.set_list(default_static_mapping_rule()).await;
            rules = static_nat_config_service.list().await;
        }

        update_mapping_rules(rules, vec![]);

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
        new_rules: Vec<Self::Config>,
        old_rules: Vec<Self::Config>,
    ) {
        update_mapping_rules(new_rules, old_rules);
    }
}

pub fn update_mapping_rules(
    rules: Vec<StaticNatMappingConfig>,
    old_rules: Vec<StaticNatMappingConfig>,
) {
    let new_rules: HashSet<StaticNatMappingItem> =
        rules.into_iter().filter(|e| e.enable).flat_map(|e| e.convert_to_item()).collect();
    let old_rules: HashSet<StaticNatMappingItem> =
        old_rules.into_iter().filter(|e| e.enable).flat_map(|e| e.convert_to_item()).collect();

    tracing::debug!("rules: {:?}", new_rules);
    tracing::debug!("old_rules: {:?}", old_rules);

    // 需要删除的
    let to_delete: HashSet<_> = old_rules.difference(&new_rules).cloned().collect();

    // 需要添加的
    let to_add: HashSet<_> = new_rules.difference(&old_rules).cloned().collect();

    tracing::debug!("delete static mapping items: {:?}", to_delete);
    tracing::debug!("add static mapping items: {:?}", to_add);

    landscape_ebpf::map_setting::nat::del_static_nat_mapping(to_delete.into_iter());
    landscape_ebpf::map_setting::nat::add_static_nat_mapping(to_add.into_iter());
}

pub fn mapping_rule_into_hash(
    mappings: Vec<StaticNatMappingConfig>,
) -> HashMap<Uuid, StaticNatMappingConfig> {
    let mut result = HashMap::new();

    for mapping in mappings {
        if mapping.enable {
            result.insert(mapping.id, mapping);
        }
    }

    result
}

pub fn default_static_mapping_rule() -> Vec<StaticNatMappingConfig> {
    let mut result = Vec::with_capacity(5);
    // DHCPv4 Clinet
    result.push(StaticNatMappingConfig {
        wan_iface_name: None,
        lan_ipv4: Some(Ipv4Addr::UNSPECIFIED),
        lan_ipv6: None,
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
        lan_ipv4: None,
        lan_ipv6: Some(Ipv6Addr::UNSPECIFIED),
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
            lan_ipv4: Some(Ipv4Addr::UNSPECIFIED),
            lan_ipv6: None,
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
            lan_ipv4: Some(Ipv4Addr::UNSPECIFIED),
            lan_ipv6: None,
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
            lan_ipv4: Some(Ipv4Addr::UNSPECIFIED),
            lan_ipv6: None,
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
