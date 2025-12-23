use landscape_common::config::route_lan::RouteLanServiceConfig;
use landscape_common::database::{LandscapeDBTrait, LandscapeServiceDBTrait};
use landscape_common::{
    observer::IfaceObserverAction,
    service::{
        controller_service_v2::ControllerService,
        service_manager_v2::{ServiceManager, ServiceStarterTrait},
        DefaultServiceStatus, DefaultWatchServiceStatus, ServiceStatus,
    },
};
use landscape_database::provider::LandscapeDBServiceProvider;
use landscape_database::route_lan::repository::RouteLanServiceRepository;
use tokio::sync::{broadcast, oneshot};

use crate::iface::get_iface_by_name;
use crate::route::IpRouteService;

#[derive(Clone)]
#[allow(dead_code)]
pub struct RouteLanService {
    route_service: IpRouteService,
}

impl RouteLanService {
    pub fn new(route_service: IpRouteService) -> Self {
        RouteLanService { route_service }
    }
}

#[async_trait::async_trait]
impl ServiceStarterTrait for RouteLanService {
    type Status = DefaultServiceStatus;

    type Config = RouteLanServiceConfig;

    async fn start(&self, config: RouteLanServiceConfig) -> DefaultWatchServiceStatus {
        let service_status = DefaultWatchServiceStatus::new();

        if config.enable {
            if let Some(iface) = get_iface_by_name(&config.iface_name).await {
                let route_name = format!("{}-static", &config.iface_name);
                let route_service = self.route_service.clone();
                let static_routes: Vec<_> = config
                    .static_routes
                    .into_iter()
                    .flat_map(|routes| routes.into_iter())
                    .map(|e2| e2.to_lan_info(iface.index, &iface.name))
                    .collect();

                for info in static_routes {
                    route_service.insert_ipv4_lan_route(&route_name, info).await;
                }

                let status_clone = service_status.clone();
                tokio::spawn(async move {
                    create_route_lan_service(iface.index, iface.mac.is_some(), status_clone).await;
                    route_service.remove_ipv4_lan_route(&route_name).await;
                });
            } else {
                tracing::error!("Interface {} not found", config.iface_name);
            }
        }

        service_status
    }
}

pub async fn create_route_lan_service(
    ifindex: u32,
    has_mac: bool,
    service_status: DefaultWatchServiceStatus,
) {
    service_status.just_change_status(ServiceStatus::Staring);
    let (tx, rx) = oneshot::channel::<()>();
    let (other_tx, other_rx) = oneshot::channel::<()>();
    service_status.just_change_status(ServiceStatus::Running);
    let service_status_clone = service_status.clone();
    tokio::spawn(async move {
        let stop_wait = service_status_clone.wait_to_stopping();
        tracing::info!("Waiting for external stop signal");
        let _ = stop_wait.await;
        tracing::info!("Receiving external stop signal");
        let _ = tx.send(());
        tracing::info!("Send a stop signal internally");
    });
    std::thread::spawn(move || {
        tracing::info!("start attach_match_flow at ifindex: {:?}", ifindex);
        landscape_ebpf::route::lan_v2::route_lan(ifindex, has_mac, rx).unwrap();
        tracing::info!("Send an unblocking signal to an external thread");
        let _ = other_tx.send(());
    });
    let _ = other_rx.await;
    tracing::info!("End external thread blocking");
    service_status.just_change_status(ServiceStatus::Stop);
}

#[derive(Clone)]
pub struct RouteLanServiceManagerService {
    store: RouteLanServiceRepository,
    service: ServiceManager<RouteLanService>,
}

impl ControllerService for RouteLanServiceManagerService {
    type Id = String;
    type Config = RouteLanServiceConfig;
    type DatabseAction = RouteLanServiceRepository;
    type H = RouteLanService;

    fn get_service(&self) -> &ServiceManager<Self::H> {
        &self.service
    }

    fn get_repository(&self) -> &Self::DatabseAction {
        &self.store
    }
}

impl RouteLanServiceManagerService {
    pub async fn new(
        store_service: LandscapeDBServiceProvider,
        route_service: IpRouteService,
        mut dev_observer: broadcast::Receiver<IfaceObserverAction>,
    ) -> Self {
        let store = store_service.route_lan_service_store();
        let server_starter = RouteLanService::new(route_service);
        let service =
            ServiceManager::init(store.list().await.unwrap(), server_starter.clone()).await;

        let service_clone = service.clone();
        tokio::spawn(async move {
            while let Ok(msg) = dev_observer.recv().await {
                match msg {
                    IfaceObserverAction::Up(iface_name) => {
                        tracing::info!("restart {iface_name} RouteLan service");
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

        let store = store_service.route_lan_service_store();
        Self { service, store }
    }
}
