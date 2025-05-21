use landscape_common::database::{LandscapeDBTrait, LandscapeServiceDBTrait};
use landscape_common::{
    config::mss_clamp::MSSClampServiceConfig,
    observer::IfaceObserverAction,
    service::{
        controller_service::ControllerService,
        service_manager::{ServiceHandler, ServiceManager},
        DefaultServiceStatus, DefaultWatchServiceStatus, ServiceStatus,
    },
};
use landscape_database::{
    mss_clamp::repository::MssClampServiceRepository, provider::LandscapeDBServiceProvider,
};
use tokio::sync::{broadcast, oneshot};

use crate::iface::get_iface_by_name;

#[derive(Clone)]
pub struct MssClampService;

impl ServiceHandler for MssClampService {
    type Status = DefaultServiceStatus;

    type Config = MSSClampServiceConfig;

    async fn initialize(config: MSSClampServiceConfig) -> DefaultWatchServiceStatus {
        let service_status = DefaultWatchServiceStatus::new();

        if config.enable {
            if let Some(iface) = get_iface_by_name(&config.iface_name).await {
                let status_clone = service_status.clone();
                tokio::spawn(async move {
                    run_mss_clamp(
                        iface.index as i32,
                        config.clamp_size,
                        iface.mac.is_some(),
                        status_clone,
                    )
                    .await
                });
            } else {
                tracing::error!("Interface {} not found", config.iface_name);
            }
        }

        service_status
    }
}

pub async fn run_mss_clamp(
    ifindex: i32,
    mtu_size: u16,
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
        landscape_ebpf::mss_clamp::run_mss_clamp(ifindex, mtu_size, has_mac, rx);
        tracing::info!("向外部线程发送解除阻塞信号");
        let _ = other_tx.send(());
    });
    let _ = other_rx.await;
    tracing::info!("结束外部线程阻塞");
    service_status.just_change_status(ServiceStatus::Stop);
}

#[derive(Clone)]
pub struct MssClampServiceManagerService {
    store: MssClampServiceRepository,
    service: ServiceManager<MssClampService>,
}

impl ControllerService for MssClampServiceManagerService {
    type Id = String;
    type Config = MSSClampServiceConfig;
    type DatabseAction = MssClampServiceRepository;
    type H = MssClampService;

    fn get_service(&self) -> &ServiceManager<Self::H> {
        &self.service
    }

    fn get_repository(&self) -> &Self::DatabseAction {
        &self.store
    }
}

impl MssClampServiceManagerService {
    pub async fn new(
        store_service: LandscapeDBServiceProvider,
        mut dev_observer: broadcast::Receiver<IfaceObserverAction>,
    ) -> Self {
        let store = store_service.mss_clamp_service_store();
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

        let store = store_service.mss_clamp_service_store();
        Self { service, store }
    }
}
