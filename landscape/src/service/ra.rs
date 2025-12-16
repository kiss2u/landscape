use std::collections::HashMap;
use std::net::IpAddr;
use std::net::Ipv6Addr;
use std::sync::Arc;

use landscape_common::database::LandscapeDBTrait;
use landscape_common::database::LandscapeServiceDBTrait;
use landscape_common::ipv6_pd::IAPrefixMap;
use landscape_common::lan_services::ipv6_ra::IPv6NAInfo;
use landscape_common::observer::IfaceObserverAction;
use landscape_common::route::LanRouteInfo;
use landscape_common::route::LanRouteMode;
use landscape_common::service::controller_service_v2::ControllerService;
use landscape_common::service::service_manager_v2::ServiceManager;
use landscape_common::service::service_manager_v2::ServiceStarterTrait;
use landscape_common::store::storev2::LandscapeStore;
use landscape_common::{
    config::ra::IPV6RAServiceConfig,
    service::{DefaultServiceStatus, DefaultWatchServiceStatus},
};
use landscape_database::provider::LandscapeDBServiceProvider;
use landscape_database::ra::repository::IPV6RAServiceRepository;
use tokio::sync::broadcast;
use tokio::sync::RwLock;

use crate::iface::get_iface_by_name;
use crate::route::IpRouteService;

/// 控制进行路由通告
#[derive(Clone)]
pub struct IPV6RAService {
    route_service: IpRouteService,
    prefix_map: IAPrefixMap,
    iface_lease_map: Arc<RwLock<HashMap<String, Arc<RwLock<IPv6NAInfo>>>>>,
}

impl IPV6RAService {
    pub fn new(route_service: IpRouteService, prefix_map: IAPrefixMap) -> Self {
        Self {
            route_service,
            prefix_map,
            iface_lease_map: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl ServiceStarterTrait for IPV6RAService {
    type Status = DefaultServiceStatus;
    type Config = IPV6RAServiceConfig;

    async fn start(&self, config: IPV6RAServiceConfig) -> DefaultWatchServiceStatus {
        let service_status = DefaultWatchServiceStatus::new();
        if config.enable {
            let route_service = self.route_service.clone();
            let prefix_map = self.prefix_map.clone();
            let status_clone = service_status.clone();
            if let Some(iface) = get_iface_by_name(&config.iface_name).await {
                let store_key = config.get_store_key();
                let assigned_ips = {
                    let mut write = self.iface_lease_map.write().await;
                    write
                        .entry(store_key.clone())
                        .or_insert_with(|| Arc::new(RwLock::new(IPv6NAInfo::init())))
                        .clone()
                };

                if let Some(mac) = iface.mac {
                    let lan_info = LanRouteInfo {
                        ifindex: iface.index,
                        iface_name: config.iface_name.clone(),
                        iface_ip: IpAddr::V6(Ipv6Addr::UNSPECIFIED),
                        mac: Some(mac.clone()),
                        prefix: 128,
                        mode: LanRouteMode::Reachable,
                    };
                    tokio::spawn(async move {
                        let _ = crate::icmp::v6::icmp_ra_server(
                            config.config,
                            mac,
                            config.iface_name,
                            status_clone,
                            lan_info,
                            route_service,
                            prefix_map,
                            assigned_ips,
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
    server_starter: IPV6RAService,
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
        route_service: IpRouteService,
        prefix_map: IAPrefixMap,
    ) -> Self {
        let store = store_service.ra_service_store();
        let server_starter = IPV6RAService::new(route_service, prefix_map);
        let service =
            ServiceManager::init(store.list().await.unwrap(), server_starter.clone()).await;

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
        Self { service, store, server_starter }
    }

    pub async fn get_assigned_ips_by_iface_name(&self, iface_name: String) -> Option<IPv6NAInfo> {
        let info = {
            let read_lock = self.server_starter.iface_lease_map.read().await;
            read_lock.get(&iface_name).map(Clone::clone)
        };

        let Some(offer_info) = info else { return None };

        let data = offer_info.read().await.clone();
        return Some(data);
    }

    pub async fn get_assigned_ips(&self) -> HashMap<String, IPv6NAInfo> {
        let mut result = HashMap::new();

        let map = {
            let read_lock = self.server_starter.iface_lease_map.read().await;
            read_lock.clone()
        };

        for (iface_name, assigned_ips) in map {
            if let Ok(read) = assigned_ips.try_read() {
                result.insert(iface_name, read.clone());
            }
        }

        result
    }
}
