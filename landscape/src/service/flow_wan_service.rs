use landscape_common::config::flow::FlowWanServiceConfig;
use landscape_common::database::{LandscapeDBTrait, LandscapeServiceDBTrait};
use landscape_common::observer::IfaceObserverAction;
use landscape_common::service::controller_service::ControllerService;
use landscape_common::service::service_manager::ServiceManager;
use landscape_common::service::ServiceStatus;
use landscape_common::service::{
    service_manager::ServiceHandler, DefaultServiceStatus, DefaultWatchServiceStatus,
};
use landscape_database::flow_wan::repository::FlowWanServiceRepository;
use landscape_database::provider::LandscapeDBServiceProvider;
use tokio::sync::{broadcast, oneshot};

use crate::iface::get_iface_by_name;

#[derive(Clone)]
pub struct FlowWanService;

impl ServiceHandler for FlowWanService {
    type Status = DefaultServiceStatus;
    type Config = FlowWanServiceConfig;

    async fn initialize(config: FlowWanServiceConfig) -> DefaultWatchServiceStatus {
        let service_status = DefaultWatchServiceStatus::new();

        if config.enable {
            if let Some(iface) = get_iface_by_name(&config.iface_name).await {
                let status_clone = service_status.clone();
                tokio::spawn(async move {
                    create_mark_service(iface.index as i32, iface.mac.is_some(), status_clone).await
                });
            } else {
                tracing::error!("Interface {} not found", config.iface_name);
            }
        }

        service_status
    }
}

pub async fn create_mark_service(
    ifindex: i32,
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
        tracing::info!("等待外部停止信号");
        let _ = stop_wait.await;
        tracing::info!("接收外部停止信号");
        let _ = tx.send(());
        tracing::info!("向内部发送停止信号");
    });
    std::thread::spawn(move || {
        tracing::info!("启动 verdict_flow 在 ifindex: {:?}", ifindex);
        landscape_ebpf::flow::verdict::attach_verdict_flow(ifindex, has_mac, rx).unwrap();
        tracing::info!("向外部线程发送解除阻塞信号");
        let _ = other_tx.send(());
    });
    let _ = other_rx.await;
    tracing::info!("结束外部线程阻塞");
    service_status.just_change_status(ServiceStatus::Stop);
}

#[derive(Clone)]
pub struct FlowWanServiceManagerService {
    store: FlowWanServiceRepository,
    service: ServiceManager<FlowWanService>,
}

impl ControllerService for FlowWanServiceManagerService {
    type ID = String;
    type Config = FlowWanServiceConfig;
    type DatabseAction = FlowWanServiceRepository;
    type H = FlowWanService;

    fn get_service(&self) -> &ServiceManager<Self::H> {
        &self.service
    }

    fn get_repository(&self) -> &Self::DatabseAction {
        &self.store
    }
}

impl FlowWanServiceManagerService {
    pub async fn new(
        store_service: LandscapeDBServiceProvider,
        mut dev_observer: broadcast::Receiver<IfaceObserverAction>,
    ) -> Self {
        let store = store_service.flow_wan_service_store();
        let service = ServiceManager::init(store.list().await.unwrap()).await;

        let service_clone = service.clone();
        tokio::spawn(async move {
            while let Ok(msg) = dev_observer.recv().await {
                match msg {
                    IfaceObserverAction::Up(iface_name) => {
                        tracing::info!("restart {iface_name} Flow WAN service");
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

        let store = store_service.flow_wan_service_store();
        Self { service, store }
    }
}
