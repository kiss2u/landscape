use landscape_common::database::LandscapeDBTrait;
use landscape_common::database::LandscapeServiceDBTrait;
use landscape_common::service::controller_service::ControllerService;
use landscape_common::{
    config::dhcp_v4_server::DHCPv4ServiceConfig,
    observer::IfaceObserverAction,
    service::{
        dhcp::{DHCPv4ServiceStatus, DHCPv4ServiceWatchStatus},
        service_manager::{ServiceHandler, ServiceManager},
    },
};
use landscape_database::dhcp_v4_server::repository::DHCPv4ServerRepository;
use landscape_database::provider::LandscapeDBServiceProvider;
use tokio::sync::broadcast;

use crate::iface::get_iface_by_name;

#[derive(Clone)]
pub struct DHCPv4Service;

impl ServiceHandler for DHCPv4Service {
    type Status = DHCPv4ServiceStatus;
    type Config = DHCPv4ServiceConfig;

    async fn initialize(config: DHCPv4ServiceConfig) -> DHCPv4ServiceWatchStatus {
        let service_status = DHCPv4ServiceWatchStatus::new();

        if config.enable {
            if let Some(_) = get_iface_by_name(&config.iface_name).await {
                let status = service_status.clone();
                tokio::spawn(async move {
                    crate::dhcp_server::dhcp_server_new::dhcp_v4_server(
                        config.iface_name,
                        config.config,
                        status,
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
pub struct DHCPv4ServerManagerService {
    // store_service: LandscapeDBServiceProvider,
    service: ServiceManager<DHCPv4Service>,
    store: DHCPv4ServerRepository,
}

impl ControllerService for DHCPv4ServerManagerService {
    type Id = String;

    type Config = DHCPv4ServiceConfig;

    type DatabseAction = DHCPv4ServerRepository;

    type H = DHCPv4Service;

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
        let service = ServiceManager::init(store.list().await.unwrap()).await;

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
        Self { service, store }
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

    // pub async fn get_all_status(&self) -> HashMap<String, DHCPv4ServiceWatchStatus> {
    //     self.service.get_all_status().await
    // }

    // pub async fn get_config_by_name(&self, iface_name: String) -> Option<DHCPv4ServiceConfig> {
    //     self.store_service.dhcp_v4_server_store().find_by_iface_name(iface_name).await.unwrap()
    // }

    // pub async fn handle_service_config(&self, config: DHCPv4ServiceConfig) {
    //     if let Ok(()) = self.service.update_service(config.clone()).await {
    //         self.store_service.dhcp_v4_server_store().set(config).await.unwrap();
    //     }
    // }

    // pub async fn delete_and_stop_iface_service(
    //     &self,
    //     iface_name: String,
    // ) -> Option<DHCPv4ServiceWatchStatus> {
    //     self.store_service.dhcp_v4_server_store().delete(iface_name.clone()).await.unwrap();
    //     self.service.stop_service(iface_name).await
    // }
}
