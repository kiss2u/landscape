use std::{
    collections::{HashMap, HashSet},
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
};

use landscape_common::{
    config::nat::StaticNatMappingConfig, service::controller_service::ConfigController,
    utils::time::get_f64_timestamp, LANDSCAPE_DEFAULE_DHCP_V4_CLIENT_PORT,
    LANDSCAPE_DEFAULE_DHCP_V6_CLIENT_PORT,
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
    let rules = mapping_rule_into_hash(rules);
    let old_rules = mapping_rule_into_hash(old_rules);

    tracing::debug!("rules: {:?}", rules);
    tracing::debug!("old_rules: {:?}", old_rules);

    let new_ids: HashSet<_> = rules.keys().collect();
    let old_ids: HashSet<_> = old_rules.keys().collect();

    let remove_ids: HashSet<_> = &old_ids - &new_ids;
    let remove_rules: Vec<_> =
        old_rules.values().filter(|v| remove_ids.contains(&v.id)).cloned().collect();

    let mut update_list = vec![];
    for id in new_ids {
        match (rules.get(id), old_rules.get(id)) {
            (Some(rule), Some(old_rule)) => {
                if !rule.is_same_config(&old_rule) {
                    update_list.push(rule.clone());
                }
            }
            (Some(rule), None) => {
                update_list.push(rule.clone());
            }
            _ => {}
        }
    }

    tracing::debug!("update_config: {:?}", update_list);
    tracing::debug!("delete_keys: {:?}", remove_rules);

    landscape_ebpf::map_setting::nat::add_static_nat_mapping(update_list);
    landscape_ebpf::map_setting::nat::del_static_nat_mapping(remove_rules);
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
    let mut result = vec![
        // DHCPv4 Clinet
        StaticNatMappingConfig {
            wan_port: LANDSCAPE_DEFAULE_DHCP_V4_CLIENT_PORT,
            wan_iface_name: None,
            lan_port: LANDSCAPE_DEFAULE_DHCP_V4_CLIENT_PORT,
            lan_ip: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            l4_protocol: 17,
            id: Uuid::new_v4(),
            enable: true,
            update_at: get_f64_timestamp(),
        },
        // DHCPv6 Clinet
        StaticNatMappingConfig {
            wan_port: LANDSCAPE_DEFAULE_DHCP_V6_CLIENT_PORT,
            wan_iface_name: None,
            lan_port: LANDSCAPE_DEFAULE_DHCP_V6_CLIENT_PORT,
            lan_ip: IpAddr::V6(Ipv6Addr::UNSPECIFIED),
            l4_protocol: 17,
            id: Uuid::new_v4(),
            enable: true,
            update_at: get_f64_timestamp(),
        },
    ];
    #[cfg(debug_assertions)]
    {
        result.push(StaticNatMappingConfig {
            wan_port: 8080,
            wan_iface_name: None,
            lan_port: 8081,
            lan_ip: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            l4_protocol: 6,
            id: Uuid::new_v4(),
            enable: true,
            update_at: get_f64_timestamp(),
        });

        result.push(StaticNatMappingConfig {
            wan_port: 5173,
            wan_iface_name: None,
            lan_port: 5173,
            lan_ip: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            l4_protocol: 6,
            id: Uuid::new_v4(),
            enable: true,
            update_at: get_f64_timestamp(),
        });

        result.push(StaticNatMappingConfig {
            wan_port: 22,
            wan_iface_name: None,
            lan_port: 22,
            lan_ip: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            l4_protocol: 6,
            id: Uuid::new_v4(),
            enable: true,
            update_at: get_f64_timestamp(),
        });
    }
    result
}
