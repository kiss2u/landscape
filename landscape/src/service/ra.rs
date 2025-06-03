use landscape_common::database::LandscapeDBTrait;
use landscape_common::database::LandscapeServiceDBTrait;
use landscape_common::observer::IfaceObserverAction;
use landscape_common::service::controller_service::ControllerService;
use landscape_common::service::service_manager::ServiceManager;
use landscape_common::{
    config::ra::IPV6RAServiceConfig,
    service::{service_manager::ServiceHandler, DefaultServiceStatus, DefaultWatchServiceStatus},
};
use landscape_database::provider::LandscapeDBServiceProvider;
use landscape_database::ra::repository::IPV6RAServiceRepository;
use tokio::sync::broadcast;

use crate::iface::get_iface_by_name;

/// 控制进行路由通告
#[derive(Clone)]
pub struct IPV6RAService;

impl ServiceHandler for IPV6RAService {
    type Status = DefaultServiceStatus;
    type Config = IPV6RAServiceConfig;

    async fn initialize(config: IPV6RAServiceConfig) -> DefaultWatchServiceStatus {
        let service_status = DefaultWatchServiceStatus::new();
        if config.enable {
            let status_clone = service_status.clone();
            if let Some(iface) = get_iface_by_name(&config.iface_name).await {
                if let Some(mac) = iface.mac {
                    tokio::spawn(async move {
                        let _ = crate::icmp::v6::icmp_ra_server(
                            config.config,
                            mac,
                            config.iface_name,
                            status_clone,
                        )
                        .await;
                    });
                }
            }
        }

        service_status
    }
}

#[derive(Clone)]
pub struct IPV6RAManagerService {
    store: IPV6RAServiceRepository,
    service: ServiceManager<IPV6RAService>,
}

impl ControllerService for IPV6RAManagerService {
    type Id = String;
    type Config = IPV6RAServiceConfig;
    type DatabseAction = IPV6RAServiceRepository;
    type H = IPV6RAService;

    fn get_service(&self) -> &ServiceManager<Self::H> {
        &self.service
    }

    fn get_repository(&self) -> &Self::DatabseAction {
        &self.store
    }
}

impl IPV6RAManagerService {
    pub async fn new(
        store_service: LandscapeDBServiceProvider,
        mut dev_observer: broadcast::Receiver<IfaceObserverAction>,
    ) -> Self {
        let store = store_service.ra_service_store();
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

        let store = store_service.ra_service_store();
        Self { service, store }
    }
}
