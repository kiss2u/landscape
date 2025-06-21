use std::collections::HashMap;
use std::sync::Arc;

use landscape_common::database::LandscapeDBTrait;
use landscape_common::database::LandscapeServiceDBTrait;
use landscape_common::dhcp::DHCPv4OfferInfo;
use landscape_common::service::controller_service_v2::ControllerService;
use landscape_common::store::storev2::LandscapeStore;
use landscape_common::{
    config::dhcp_v4_server::DHCPv4ServiceConfig,
    observer::IfaceObserverAction,
    service::{
        dhcp::{DHCPv4ServiceStatus, DHCPv4ServiceWatchStatus},
        service_manager_v2::{ServiceManager, ServiceStarterTrait},
    },
};
use landscape_database::dhcp_v4_server::repository::DHCPv4ServerRepository;
use landscape_database::provider::LandscapeDBServiceProvider;
use tokio::sync::broadcast;
use tokio::sync::RwLock;

use crate::iface::get_iface_by_name;

#[derive(Clone)]
pub struct DHCPv4ServerStarter {
    // TODO: 例如 Router Service
    pub iface_lease_map: Arc<RwLock<HashMap<String, Arc<RwLock<DHCPv4OfferInfo>>>>>,
}

impl DHCPv4ServerStarter {
    pub fn new() -> DHCPv4ServerStarter {
        DHCPv4ServerStarter {
            iface_lease_map: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl ServiceStarterTrait for DHCPv4ServerStarter {
    type Status = DHCPv4ServiceStatus;
    type Config = DHCPv4ServiceConfig;

    async fn start(&self, config: DHCPv4ServiceConfig) -> DHCPv4ServiceWatchStatus {
        let service_status = DHCPv4ServiceWatchStatus::new();

        if config.enable {
            if let Some(_) = get_iface_by_name(&config.iface_name).await {
                let assigned_ips = {
                    let mut write = self.iface_lease_map.write().await;
                    let key = config.get_store_key();
                    write
                        .entry(key.clone())
                        .or_insert_with(|| Arc::new(RwLock::new(DHCPv4OfferInfo::default())))
                        .clone()
                };
                let status = service_status.clone();
                tokio::spawn(async move {
                    crate::dhcp_server::dhcp_server_new::dhcp_v4_server(
                        config.iface_name,
                        config.config,
                        status,
                        assigned_ips,
                    )
                    .await;
                });
            } else {
                tracing::error!("Interface {} not found", config.iface_name);
            }
        }

        service_status
    }
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct DHCPv4ServerManagerService {
    service: ServiceManager<DHCPv4ServerStarter>,
    store: DHCPv4ServerRepository,
    server_booter: DHCPv4ServerStarter,
}

impl ControllerService for DHCPv4ServerManagerService {
    type Id = String;

    type Config = DHCPv4ServiceConfig;

    type DatabseAction = DHCPv4ServerRepository;

    type H = DHCPv4ServerStarter;

    fn get_service(&self) -> &ServiceManager<Self::H> {
        &self.service
    }

    fn get_repository(&self) -> &Self::DatabseAction {
        &self.store
    }
}

impl DHCPv4ServerManagerService {
    pub async fn new(
        store_service: LandscapeDBServiceProvider,
        mut dev_observer: broadcast::Receiver<IfaceObserverAction>,
    ) -> Self {
        let store = store_service.dhcp_v4_server_store();
        let server_booter = DHCPv4ServerStarter::new();
        let service =
            ServiceManager::init(store.list().await.unwrap(), server_booter.clone()).await;

        let service_clone = service.clone();
        tokio::spawn(async move {
            while let Ok(msg) = dev_observer.recv().await {
                match msg {
                    IfaceObserverAction::Up(iface_name) => {
                        tracing::info!("restart {iface_name} Firewall service");
                        let service_config = if let Some(service_config) =
                            store.find_by_iface_name(iface_name.clone()).await.unwrap()
                        {
                            service_config
                        } else {
                            continue;
                        };

                        let _ = service_clone.update_service(service_config).await;
                    }
                    IfaceObserverAction::Down(_) => {}
                }
            }
        });

        let store = store_service.dhcp_v4_server_store();
        Self { service, store, server_booter }
    }

    pub async fn check_ip_range_conflict(
        &self,
        new_config: &DHCPv4ServiceConfig,
    ) -> Result<(), String> {
        // 获取所有现有配置
        let Ok(all_configs) = self.get_repository().list().await else {
            return Err("read all config error".to_string());
        };

        for existing_config in all_configs {
            // 跳过同一个接口的配置
            if existing_config.iface_name == new_config.iface_name {
                continue;
            }

            // 只检查启用的配置
            if !existing_config.enable {
                continue;
            }

            // 检查IP范围是否重叠
            if new_config.config.has_ip_range_overlap(&existing_config.config) {
                return Err(format!(
                    "IP range conflict detected with interface '{}'. New range: {}-{}, Existing range: {}-{}",
                    existing_config.iface_name,
                    new_config.config.ip_range_start,
                    new_config.config.get_ip_range().1,
                    existing_config.config.ip_range_start,
                    existing_config.config.get_ip_range().1
                ));
            }
        }

        Ok(())
    }
}
