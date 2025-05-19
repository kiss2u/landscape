use std::time::Duration;

use landscape_common::{config::InitConfig, database::repository::Repository};
use sea_orm::{Database, DatabaseConnection};

use migration::{Migrator, MigratorTrait};

use crate::{
    dhcp_v4_server::repository::DHCPv4ServerRepository,
    dhcp_v6_client::repository::DHCPv6ClientRepository, dns::repository::DNSRepository,
    firewall::repository::FirewallServiceRepository,
    flow_wan::repository::FlowWanServiceRepository, iface::repository::NetIfaceRepository,
    mss_clamp::repository::MssClampServiceRepository,
};

/// 存储提供者  
/// 后续有需要再进行抽象
#[derive(Clone)]
pub struct LandscapeDBServiceProvider {
    database: DatabaseConnection,
}

impl LandscapeDBServiceProvider {
    pub async fn new(db_url: String) -> Self {
        let mut opt: migration::sea_orm::ConnectOptions = db_url.into();
        let (lever, _) = opt.get_sqlx_slow_statements_logging_settings();
        opt.sqlx_slow_statements_logging_settings(lever, Duration::from_secs(10));

        let database = Database::connect(opt).await.expect("Database connection failed");
        Migrator::up(&database, None).await.unwrap();
        Self { database }
    }

    /// 清空数据并且从配置从初始化
    pub async fn truncate_and_fit_from(&self, config: Option<InitConfig>) {
        if let Some(config) = config {
            let iface_store = self.iface_store();
            iface_store.truncate().await.unwrap();
            for each_config in config.ifaces {
                iface_store.set(each_config).await.unwrap();
            }
            let dhcp_v4_server_store = self.dhcp_v4_server_store();
            dhcp_v4_server_store.truncate_table().await.unwrap();
            for each_config in config.dhcpv4_services {
                dhcp_v4_server_store.set_model(each_config).await.unwrap();
            }
        }
    }

    pub fn dns_store(&self) -> DNSRepository {
        DNSRepository::new(self.database.clone())
    }

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
}

#[cfg(test)]
mod tests {
    use crate::provider::LandscapeDBServiceProvider;

    #[tokio::test]
    pub async fn test_run_database() {
        landscape_common::init_tracing!();

        let _provider =
            LandscapeDBServiceProvider::new("sqlite://../db.sqlite?mode=rwc".to_string()).await;
    }
}
