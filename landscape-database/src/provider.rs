use std::time::Duration;

use landscape_common::{
    config::{InitConfig, StoreRuntimeConfig},
    database::repository::Repository,
};
use sea_orm::{Database, DatabaseConnection};

use migration::{Migrator, MigratorTrait};

use crate::{
    dhcp_v4_server::repository::DHCPv4ServerRepository,
    dhcp_v6_client::repository::DHCPv6ClientRepository, dns_rule::repository::DNSRuleRepository,
    dst_ip_rule::repository::DstIpRuleRepository, firewall::repository::FirewallServiceRepository,
    firewall_rule::repository::FirewallRuleRepository, flow_rule::repository::FlowConfigRepository,
    flow_wan::repository::FlowWanServiceRepository,
    geo_ip::repository::GeoIpSourceConfigRepository, geo_site::repository::GeoSiteConfigRepository,
    iface::repository::NetIfaceRepository, iface_ip::repository::IfaceIpServiceRepository,
    mss_clamp::repository::MssClampServiceRepository, nat::repository::NatServiceRepository,
    pppd::repository::PPPDServiceRepository, ra::repository::IPV6RAServiceRepository,
    route_lan::repository::RouteLanServiceRepository,
    route_wan::repository::RouteWanServiceRepository,
    static_nat_mapping::repository::StaticNatMappingConfigRepository,
    wifi::repository::WifiServiceRepository,
};

/// 存储提供者  
/// 后续有需要再进行抽象
#[derive(Clone)]
pub struct LandscapeDBServiceProvider {
    database: DatabaseConnection,
}

impl LandscapeDBServiceProvider {
    pub async fn new(config: &StoreRuntimeConfig) -> Self {
        let mut opt: migration::sea_orm::ConnectOptions = config.database_path.clone().into();
        let (lever, _) = opt.get_sqlx_slow_statements_logging_settings();
        opt.sqlx_slow_statements_logging_settings(lever, Duration::from_secs(10));

        let database = Database::connect(opt).await.expect("Database connection failed");
        Migrator::up(&database, None).await.unwrap();
        Self { database }
    }

    pub async fn mem_test_db() -> Self {
        let database =
            Database::connect("sqlite::memory:").await.expect("Database connection failed");
        Migrator::up(&database, None).await.unwrap();
        Self { database }
    }

    /// 清空数据并且从配置从初始化
    pub async fn truncate_and_fit_from(&self, config: Option<InitConfig>) {
        tracing::info!("init config: {config:?}");
        if let Some(InitConfig {
            config: _,
            ifaces,
            ipconfigs,
            nats,
            marks,
            pppds,
            flow_rules,
            dns_rules,
            dhcpv6pds,
            icmpras,
            firewalls,
            firewall_rules,
            wifi_configs,
            mss_clamps,
            dst_ip_mark,
            dhcpv4_services,
            geo_ips,
            geo_sites,
            route_lans,
            route_wans,
            static_nat_mappings,
        }) = config
        {
            let iface_store = self.iface_store();
            iface_store.truncate_table().await.unwrap();
            for each_config in ifaces {
                iface_store.set_model(each_config).await.unwrap();
            }

            let dhcp_v4_server_store = self.dhcp_v4_server_store();
            dhcp_v4_server_store.truncate_table().await.unwrap();
            for each_config in dhcpv4_services {
                dhcp_v4_server_store.set_model(each_config).await.unwrap();
            }

            let wifi_config_store = self.wifi_service_store();
            wifi_config_store.truncate_table().await.unwrap();
            for each_config in wifi_configs {
                wifi_config_store.set_model(each_config).await.unwrap();
            }

            let firewall_service_store = self.firewall_service_store();
            firewall_service_store.truncate_table().await.unwrap();
            for each_config in firewalls {
                firewall_service_store.set_model(each_config).await.unwrap();
            }

            let firewall_rules_store = self.firewall_rule_store();
            firewall_rules_store.truncate_table().await.unwrap();
            for each_config in firewall_rules {
                firewall_rules_store.set_model(each_config).await.unwrap();
            }

            let iface_ipconfig_store = self.iface_ip_service_store();
            iface_ipconfig_store.truncate_table().await.unwrap();
            for each_config in ipconfigs {
                iface_ipconfig_store.set_model(each_config).await.unwrap();
            }

            let iface_nat_store = self.nat_service_store();
            iface_nat_store.truncate_table().await.unwrap();
            for each_config in nats {
                iface_nat_store.set_model(each_config).await.unwrap();
            }

            let flow_store = self.flow_rule_store();
            flow_store.truncate_table().await.unwrap();
            for each_config in flow_rules {
                flow_store.set_model(each_config).await.unwrap();
            }

            let iface_mark_store = self.flow_wan_service_store();
            iface_mark_store.truncate_table().await.unwrap();
            for each_config in marks {
                iface_mark_store.set_model(each_config).await.unwrap();
            }

            let dst_ip_rule_store = self.dst_ip_rule_store();
            dst_ip_rule_store.truncate_table().await.unwrap();
            for each_config in dst_ip_mark {
                dst_ip_rule_store.set_model(each_config).await.unwrap();
            }

            let iface_pppd_store = self.pppd_service_store();
            iface_pppd_store.truncate_table().await.unwrap();
            for each_config in pppds {
                iface_pppd_store.set_model(each_config).await.unwrap();
            }

            let dns_store = self.dns_rule_store();
            dns_store.truncate_table().await.unwrap();
            for each_config in dns_rules {
                dns_store.set_model(each_config).await.unwrap();
            }

            let ipv6pd_store = self.dhcp_v6_client_store();
            ipv6pd_store.truncate_table().await.unwrap();
            for each_config in dhcpv6pds {
                ipv6pd_store.set_model(each_config).await.unwrap();
            }

            let icmpv6ra_store = self.ra_service_store();
            icmpv6ra_store.truncate_table().await.unwrap();
            for each_config in icmpras {
                icmpv6ra_store.set_model(each_config).await.unwrap();
            }

            let mss_clamp_store = self.mss_clamp_service_store();
            mss_clamp_store.truncate_table().await.unwrap();
            for each_config in mss_clamps {
                mss_clamp_store.set_model(each_config).await.unwrap();
            }

            let geo_ip_store = self.geo_ip_rule_store();
            geo_ip_store.truncate_table().await.unwrap();
            for each_config in geo_ips {
                geo_ip_store.set_model(each_config).await.unwrap();
            }

            let geo_site_store = self.geo_site_rule_store();
            geo_site_store.truncate_table().await.unwrap();
            for each_config in geo_sites {
                geo_site_store.set_model(each_config).await.unwrap();
            }

            let rooute_lan_store = self.route_lan_service_store();
            rooute_lan_store.truncate_table().await.unwrap();
            for each_config in route_lans {
                rooute_lan_store.set_model(each_config).await.unwrap();
            }

            let rooute_wan_store = self.route_wan_service_store();
            rooute_wan_store.truncate_table().await.unwrap();
            for each_config in route_wans {
                rooute_wan_store.set_model(each_config).await.unwrap();
            }

            let static_nat_mapping_store = self.static_nat_mapping_store();
            static_nat_mapping_store.truncate_table().await.unwrap();
            for each_config in static_nat_mappings {
                static_nat_mapping_store.set_model(each_config).await.unwrap();
            }
        }
    }

    // config

    pub fn dns_rule_store(&self) -> DNSRuleRepository {
        DNSRuleRepository::new(self.database.clone())
    }

    pub fn firewall_rule_store(&self) -> FirewallRuleRepository {
        FirewallRuleRepository::new(self.database.clone())
    }

    pub fn flow_rule_store(&self) -> FlowConfigRepository {
        FlowConfigRepository::new(self.database.clone())
    }

    pub fn dst_ip_rule_store(&self) -> DstIpRuleRepository {
        DstIpRuleRepository::new(self.database.clone())
    }

    pub fn geo_site_rule_store(&self) -> GeoSiteConfigRepository {
        GeoSiteConfigRepository::new(self.database.clone())
    }

    pub fn geo_ip_rule_store(&self) -> GeoIpSourceConfigRepository {
        GeoIpSourceConfigRepository::new(self.database.clone())
    }

    // service

    pub fn iface_store(&self) -> NetIfaceRepository {
        NetIfaceRepository::new(self.database.clone())
    }

    pub fn dhcp_v4_server_store(&self) -> DHCPv4ServerRepository {
        DHCPv4ServerRepository::new(self.database.clone())
    }

    pub fn dhcp_v6_client_store(&self) -> DHCPv6ClientRepository {
        DHCPv6ClientRepository::new(self.database.clone())
    }

    pub fn firewall_service_store(&self) -> FirewallServiceRepository {
        FirewallServiceRepository::new(self.database.clone())
    }

    pub fn flow_wan_service_store(&self) -> FlowWanServiceRepository {
        FlowWanServiceRepository::new(self.database.clone())
    }

    pub fn mss_clamp_service_store(&self) -> MssClampServiceRepository {
        MssClampServiceRepository::new(self.database.clone())
    }

    pub fn nat_service_store(&self) -> NatServiceRepository {
        NatServiceRepository::new(self.database.clone())
    }

    pub fn pppd_service_store(&self) -> PPPDServiceRepository {
        PPPDServiceRepository::new(self.database.clone())
    }

    pub fn ra_service_store(&self) -> IPV6RAServiceRepository {
        IPV6RAServiceRepository::new(self.database.clone())
    }

    pub fn iface_ip_service_store(&self) -> IfaceIpServiceRepository {
        IfaceIpServiceRepository::new(self.database.clone())
    }

    pub fn wifi_service_store(&self) -> WifiServiceRepository {
        WifiServiceRepository::new(self.database.clone())
    }

    pub fn route_wan_service_store(&self) -> RouteWanServiceRepository {
        RouteWanServiceRepository::new(self.database.clone())
    }

    pub fn route_lan_service_store(&self) -> RouteLanServiceRepository {
        RouteLanServiceRepository::new(self.database.clone())
    }

    pub fn static_nat_mapping_store(&self) -> StaticNatMappingConfigRepository {
        StaticNatMappingConfigRepository::new(self.database.clone())
    }
}

#[cfg(test)]
mod tests {
    use landscape_common::config::StoreRuntimeConfig;

    use crate::provider::LandscapeDBServiceProvider;

    #[tokio::test]
    pub async fn test_run_database() {
        landscape_common::init_tracing!();

        let config = StoreRuntimeConfig {
            database_path: "sqlite://../db.sqlite?mode=rwc".to_string(),
        };
        let _provider = LandscapeDBServiceProvider::new(&config).await;
    }
}
