use std::collections::HashSet;
use std::net::Ipv4Addr;
use std::process::Command;
use std::process::Stdio;

use tokio::sync::{oneshot, watch};

use landscape_common::config::ppp::PPPDConfig;
use landscape_common::database::LandscapeDBTrait;
use landscape_common::global_const::default_router::RouteInfo;
use landscape_common::global_const::default_router::RouteType;
use landscape_common::global_const::default_router::LD_ALL_ROUTERS;
use landscape_common::service::controller_service::ControllerService;
use landscape_common::service::service_manager::ServiceManager;
use landscape_common::service::ServiceStatus;
use landscape_common::{
    config::ppp::PPPDServiceConfig,
    service::{service_manager::ServiceHandler, DefaultServiceStatus, DefaultWatchServiceStatus},
};
use landscape_database::pppd::repository::PPPDServiceRepository;
use landscape_database::provider::LandscapeDBServiceProvider;

use crate::iface::get_iface_by_name;

#[derive(Clone)]
pub struct PPPDService;

impl ServiceHandler for PPPDService {
    type Status = DefaultServiceStatus;
    type Config = PPPDServiceConfig;

    async fn initialize(config: PPPDServiceConfig) -> DefaultWatchServiceStatus {
        let service_status = DefaultWatchServiceStatus::new();
        if config.enable {
            if let Some(_) = get_iface_by_name(&config.attach_iface_name).await {
                let status_clone = service_status.clone();

                tokio::spawn(async move {
                    create_pppd_thread(
                        config.attach_iface_name,
                        config.iface_name,
                        config.pppd_config,
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

pub async fn create_pppd_thread(
    attach_iface_name: String,
    ppp_iface_name: String,
    pppd_conf: PPPDConfig,
    service_status: DefaultWatchServiceStatus,
) {
    service_status.just_change_status(ServiceStatus::Staring);

    let (tx, mut rx) = oneshot::channel::<()>();
    let (other_tx, other_rx) = oneshot::channel::<()>();

    service_status.just_change_status(ServiceStatus::Running);
    let service_status_clone = service_status.clone();
    tokio::spawn(async move {
        let stop_wait = service_status_clone.wait_to_stopping();
        tracing::debug!("等待外部停止信号");
        let _ = stop_wait.await;
        tracing::info!("接收外部停止信号");
        let _ = tx.send(());
        tracing::info!("向内部发送停止信号");
    });

    let Ok(_) = pppd_conf.write_config(&attach_iface_name, &ppp_iface_name) else {
        tracing::error!("pppd config write error");
        service_status.just_change_status(ServiceStatus::Stop);
        return;
    };

    let as_router = pppd_conf.default_route;

    let (updata_ip, mut updata_ip_rx) = watch::channel(());
    let ppp_iface_name_clone = ppp_iface_name.clone();
    tokio::spawn(async move {
        let mut ip4addr: Option<(u32, HashSet<Ipv4Addr>)> = None;
        while let Ok(_) = updata_ip_rx.changed().await {
            let new_ip4addr = crate::get_address(&ppp_iface_name_clone).await;
            if let Some(new_ip4addr) = new_ip4addr {
                let update = if let Some(data) = ip4addr { data != new_ip4addr } else { true };
                if update {
                    for ip in new_ip4addr.1.iter() {
                        landscape_ebpf::map_setting::add_wan_ip(new_ip4addr.0, ip.clone());

                        if as_router {
                            LD_ALL_ROUTERS
                                .add_route(RouteInfo {
                                    iface_name: ppp_iface_name_clone.clone(),
                                    weight: 1,
                                    route: RouteType::PPP,
                                })
                                .await;
                        } else {
                            LD_ALL_ROUTERS.del_route_by_iface(&ppp_iface_name_clone).await;
                        }
                    }
                }
                ip4addr = Some(new_ip4addr);
            }
        }
    });

    tracing::info!("pppd 配置写入成功");
    let iface_name = ppp_iface_name.clone();
    std::thread::spawn(move || {
        tracing::info!("pppd 启动中");
        let mut child = match Command::new("pppd")
            .arg("nodetach")
            .arg("call")
            .arg(&ppp_iface_name)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(e) => {
                tracing::error!("启动 pppd 失败: {}", e);
                return;
            }
        };
        let mut check_error_times = 0;
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
            updata_ip.send_replace(());
            match child.try_wait() {
                Ok(Some(status)) => {
                    tracing::warn!("pppd 退出， 状态码： {:?}", status);
                    break;
                }
                Ok(None) => {
                    check_error_times = 0;
                }
                Err(e) => {
                    tracing::error!("pppd error: {e:?}");
                    if check_error_times > 3 {
                        break;
                    }
                    check_error_times += 1;
                }
            }

            match rx.try_recv() {
                Err(tokio::sync::oneshot::error::TryRecvError::Empty) => {}
                Ok(_) | Err(tokio::sync::oneshot::error::TryRecvError::Closed) => {
                    tracing::error!("rx, 通知错误");
                    break;
                }
            }
        }
        let _ = child.kill();
        tracing::info!("向外部线程发送解除阻塞信号");
        let _ = other_tx.send(());
        pppd_conf.delete_config(&ppp_iface_name);
    });

    let _ = other_rx.await;
    tracing::info!("结束外部线程阻塞");
    if as_router {
        LD_ALL_ROUTERS.del_route_by_iface(&iface_name).await;
    }
    service_status.just_change_status(ServiceStatus::Stop);
}

#[derive(Clone)]
pub struct PPPDServiceConfigManagerService {
    store: PPPDServiceRepository,
    service: ServiceManager<PPPDService>,
}

impl ControllerService for PPPDServiceConfigManagerService {
    type Id = String;
    type Config = PPPDServiceConfig;
    type DatabseAction = PPPDServiceRepository;
    type H = PPPDService;

    fn get_service(&self) -> &ServiceManager<Self::H> {
        &self.service
    }

    fn get_repository(&self) -> &Self::DatabseAction {
        &self.store
    }
}

impl PPPDServiceConfigManagerService {
    pub async fn new(store_service: LandscapeDBServiceProvider) -> Self {
        let store = store_service.pppd_service_store();
        let service = ServiceManager::init(store.list().await.unwrap()).await;

        Self { service, store }
    }

    pub async fn get_pppd_configs_by_attach_iface_name(
        &self,
        attach_name: String,
    ) -> Vec<PPPDServiceConfig> {
        self.store.get_pppd_configs_by_attach_iface_name(attach_name).await.unwrap()
    }

    pub async fn stop_pppds_by_attach_iface_name(&self, attach_name: String) {
        let configs = self.get_pppd_configs_by_attach_iface_name(attach_name).await;
        for each in configs {
            self.service.stop_service(each.iface_name.clone()).await;
            self.get_repository().delete(each.iface_name).await.unwrap();
        }
    }
}
