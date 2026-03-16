use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::process::Command;
use std::process::Stdio;
use std::time::{Duration, Instant};

use landscape_common::route::LanRouteInfo;
use landscape_common::route::LanRouteMode;
use landscape_common::route::RouteTargetInfo;
use landscape_common::SYSCTL_IPV6_RA_ACCEPT_PATTERN;
use sysctl::Sysctl as _;
use tokio::sync::{oneshot, watch};

use landscape_common::database::LandscapeStore;
use landscape_common::global_const::default_router::RouteInfo;
use landscape_common::global_const::default_router::RouteType;
use landscape_common::global_const::default_router::LD_ALL_ROUTERS;
use landscape_common::iface::ppp::PPPDConfig;
use landscape_common::service::controller::ControllerService;
use landscape_common::service::manager::ServiceManager;
use landscape_common::service::ServiceStatus;
use landscape_common::{
    concurrency::{
        short_thread_name, spawn_named_thread, spawn_task_with_resource, task_label, thread_name,
    },
    iface::ppp::PPPDServiceConfig,
    service::{manager::ServiceStarterTrait, WatchService},
};
use landscape_database::pppd::repository::PPPDServiceRepository;
use landscape_database::provider::LandscapeDBServiceProvider;

use crate::iface::get_iface_by_name;
use crate::route::IpRouteService;

const PPPD_RETRY_BASE_SECS: u64 = 4;
const PPPD_RETRY_MAX_SECS: u64 = 10 * 60;

fn calc_pppd_retry_backoff_secs(failure_count: u32) -> u64 {
    let exp = failure_count.saturating_sub(1).min(31);
    let secs = PPPD_RETRY_BASE_SECS.saturating_mul(1u64 << exp);
    secs.min(PPPD_RETRY_MAX_SECS)
}

fn wait_stop_or_timeout(rx: &mut oneshot::Receiver<()>, duration: Duration) -> bool {
    let deadline = Instant::now() + duration;
    loop {
        match rx.try_recv() {
            Err(tokio::sync::oneshot::error::TryRecvError::Empty) => {}
            Ok(_) | Err(tokio::sync::oneshot::error::TryRecvError::Closed) => return true,
        }

        let now = Instant::now();
        if now >= deadline {
            return false;
        }

        let remain = deadline.saturating_duration_since(now);
        std::thread::sleep(remain.min(Duration::from_millis(200)));
    }
}

#[derive(Clone)]
pub struct PPPDService {
    route_service: IpRouteService,
}

impl PPPDService {
    pub fn new(route_service: IpRouteService) -> Self {
        PPPDService { route_service }
    }
}

#[async_trait::async_trait]
impl ServiceStarterTrait for PPPDService {
    type Config = PPPDServiceConfig;

    async fn start(&self, config: PPPDServiceConfig) -> WatchService {
        let service_status = WatchService::new();
        let route_service = self.route_service.clone();
        if config.enable {
            if let Some(_) = get_iface_by_name(&config.attach_iface_name).await {
                let status_clone = service_status.clone();
                let iface_name = config.iface_name.clone();

                spawn_task_with_resource(
                    task_label::task::PPPD_RUN,
                    iface_name.clone(),
                    async move {
                        create_pppd_thread(
                            config.attach_iface_name,
                            config.iface_name,
                            config.pppd_config,
                            status_clone,
                            route_service,
                        )
                        .await
                    },
                );
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
    service_status: WatchService,
    route_service: IpRouteService,
) {
    service_status.just_change_status(ServiceStatus::Staring);

    let (tx, mut rx) = oneshot::channel::<()>();
    let (other_tx, other_rx) = oneshot::channel::<()>();

    service_status.just_change_status(ServiceStatus::Running);
    let service_status_clone = service_status.clone();
    spawn_task_with_resource(task_label::task::PPPD_STOP, ppp_iface_name.clone(), async move {
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
    let route_service_clone = route_service.clone();
    spawn_task_with_resource(
        task_label::task::PPPD_IP_WATCH,
        ppp_iface_name_clone.clone(),
        async move {
            let mut ip4addr: Option<(u32, Option<Ipv4Addr>, Option<Ipv4Addr>)> = None;
            while let Ok(_) = updata_ip_rx.changed().await {
                let new_ip4addr = crate::get_ppp_address(&ppp_iface_name_clone).await;
                if let Some(new_ip4addr) = new_ip4addr {
                    let update = if let Some(data) = ip4addr { data != new_ip4addr } else { true };
                    if update {
                        if let (Some(ip), Some(peer_ip)) = (new_ip4addr.1, new_ip4addr.2) {
                            set_iface_ipv6_ra_accept_to_2(&ppp_iface_name_clone);
                            landscape_ebpf::map_setting::add_ipv4_wan_ip(
                                new_ip4addr.0,
                                ip.clone(),
                                Some(peer_ip.clone()),
                                32,
                                None,
                            );

                            let info = RouteTargetInfo {
                                ifindex: new_ip4addr.0,
                                weight: 1,
                                mac: None,
                                is_docker: false,
                                iface_name: ppp_iface_name_clone.clone(),
                                iface_ip: IpAddr::V4(ip.clone()),
                                default_route: as_router,
                                gateway_ip: IpAddr::V4(peer_ip),
                            };
                            route_service_clone
                                .insert_ipv4_wan_route(&ppp_iface_name_clone, info)
                                .await;

                            route_service_clone
                                .insert_ipv4_lan_route(
                                    &ppp_iface_name_clone,
                                    LanRouteInfo {
                                        ifindex: new_ip4addr.0,
                                        iface_name: ppp_iface_name_clone.clone(),
                                        iface_ip: IpAddr::V4(ip.clone()),
                                        mac: None,
                                        prefix: 32,
                                        mode: LanRouteMode::Reachable,
                                    },
                                )
                                .await;
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
        },
    );

    tracing::info!("pppd 配置写入成功");
    let iface_name = ppp_iface_name.clone();
    spawn_named_thread(short_thread_name(thread_name::prefix::PPPD, &ppp_iface_name), move || {
        let mut connect_failure_count: u32 = 0;
        let mut should_stop = false;

        'restart: loop {
            if wait_stop_or_timeout(&mut rx, Duration::from_secs(0)) {
                break;
            }

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
                    connect_failure_count = connect_failure_count.saturating_add(1);
                    let backoff = calc_pppd_retry_backoff_secs(connect_failure_count);
                    tracing::error!(
                        "启动 pppd 失败: {}, {} 秒后重试 (failure_count={})",
                        e,
                        backoff,
                        connect_failure_count
                    );
                    if wait_stop_or_timeout(&mut rx, Duration::from_secs(backoff)) {
                        break;
                    }
                    continue 'restart;
                }
            };
            let mut check_error_times = 0;
            let mut healthy_once = false;
            loop {
                std::thread::sleep(Duration::from_secs(1));
                updata_ip.send_replace(());
                match child.try_wait() {
                    Ok(Some(status)) => {
                        tracing::warn!("pppd 退出， 状态码： {:?}", status);
                        break;
                    }
                    Ok(None) => {
                        check_error_times = 0;
                        if !healthy_once {
                            healthy_once = true;
                            connect_failure_count = 0;
                        }
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
                        tracing::info!("收到停止 pppd 信号");
                        should_stop = true;
                        break;
                    }
                }
            }
            let _ = child.kill();
            if should_stop {
                break;
            }

            connect_failure_count = connect_failure_count.saturating_add(1);
            let backoff = calc_pppd_retry_backoff_secs(connect_failure_count);
            tracing::warn!(
                "pppd 连接中断，{} 秒后重试 (failure_count={})",
                backoff,
                connect_failure_count
            );
            if wait_stop_or_timeout(&mut rx, Duration::from_secs(backoff)) {
                break;
            }
        }

        tracing::info!("向外部线程发送解除阻塞信号");
        let _ = other_tx.send(());
        pppd_conf.delete_config(&ppp_iface_name);
    })
    .expect("failed to spawn pppd worker thread");

    let _ = other_rx.await;
    tracing::info!("结束外部线程阻塞");
    if as_router {
        LD_ALL_ROUTERS.del_route_by_iface(&iface_name).await;
    }
    route_service.remove_ipv4_wan_route(&iface_name).await;
    route_service.remove_ipv4_lan_route(&iface_name).await;
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
    pub async fn new(
        store_service: LandscapeDBServiceProvider,
        route_service: IpRouteService,
    ) -> Self {
        let store = store_service.pppd_service_store();
        let server_starter = PPPDService::new(route_service);
        let service = ServiceManager::init(store.list().await.unwrap(), server_starter).await;

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

fn set_iface_ipv6_ra_accept_to_2(iface_name: &str) {
    if let Ok(ctl) = sysctl::Ctl::new(&SYSCTL_IPV6_RA_ACCEPT_PATTERN.replace("{}", iface_name)) {
        match ctl.set_value_string("2") {
            Ok(value) => {
                if value != "2" {
                    tracing::error!("modify value error: {:?}", value)
                }
            }
            Err(e) => {
                tracing::error!("err: {e:?}")
            }
        }
    }
}
