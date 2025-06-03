use tokio::sync::broadcast;

use landscape_common::database::LandscapeDBTrait;
use landscape_common::database::LandscapeServiceDBTrait;
use landscape_common::{
    config::dhcp_v6_client::IPV6PDServiceConfig,
    observer::IfaceObserverAction,
    service::{
        controller_service::ControllerService,
        service_manager::{ServiceHandler, ServiceManager},
        DefaultServiceStatus, DefaultWatchServiceStatus,
    },
    LANDSCAPE_DEFAULE_DHCP_V6_CLIENT_PORT,
};
use landscape_database::{
    dhcp_v6_client::repository::DHCPv6ClientRepository, provider::LandscapeDBServiceProvider,
};

use crate::iface::get_iface_by_name;

#[derive(Clone)]
pub struct IPV6PDService;

impl ServiceHandler for IPV6PDService {
    type Status = DefaultServiceStatus;
    type Config = IPV6PDServiceConfig;

    async fn initialize(config: IPV6PDServiceConfig) -> DefaultWatchServiceStatus {
        let service_status = DefaultWatchServiceStatus::new();
        // service_status.just_change_status(ServiceStatus::Staring);
        if config.enable {
            if let Some(_) = get_iface_by_name(&config.iface_name).await {
                let status_clone = service_status.clone();
                tokio::spawn(async move {
                    crate::dhcp_client::v6::dhcp_v6_pd_client(
                        config.iface_name,
                        config.config.mac,
                        LANDSCAPE_DEFAULE_DHCP_V6_CLIENT_PORT,
                        status_clone,
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
pub struct DHCPv6ClientManagerService {
    store: DHCPv6ClientRepository,
    service: ServiceManager<IPV6PDService>,
}

impl ControllerService for DHCPv6ClientManagerService {
    type Id = String;
    type Config = IPV6PDServiceConfig;
    type DatabseAction = DHCPv6ClientRepository;
    type H = IPV6PDService;

    fn get_service(&self) -> &ServiceManager<Self::H> {
        &self.service
    }

    fn get_repository(&self) -> &Self::DatabseAction {
        &self.store
    }
}

impl DHCPv6ClientManagerService {
    pub async fn new(
        store_service: LandscapeDBServiceProvider,
        mut dev_observer: broadcast::Receiver<IfaceObserverAction>,
    ) -> Self {
        let store = store_service.dhcp_v6_client_store();
        let service = ServiceManager::init(store.list().await.unwrap()).await;

        let service_clone = service.clone();
        tokio::spawn(async move {
            while let Ok(msg) = dev_observer.recv().await {
                match msg {
                    IfaceObserverAction::Up(iface_name) => {
                        tracing::info!("restart {iface_name} IPv6PD service");
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

        let store = store_service.dhcp_v6_client_store();
        Self { service, store }
    }
}
