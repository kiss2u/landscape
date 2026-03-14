use core::mem::drop;
use std::{
    collections::{HashMap, HashSet},
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    sync::Arc,
};

pub mod lan_service;
pub mod wan_service;

use hickory_proto::rr::RecordType;
use landscape_common::{
    config::FlowId,
    event::route::RouteEvent,
    flow::{config::FlowConfig, FlowTarget},
    route::{LanIPv6RouteKey, LanRouteInfo, LanRouteMode, RouteTargetInfo},
};
use landscape_database::flow_rule::repository::FlowConfigRepository;
use landscape_dns::server::LocalDnsAnswerProvider;
use landscape_ebpf::map_setting::route::{add_lan_route, del_lan_route};
use tokio::sync::{mpsc, RwLock};

use landscape_common::database::LandscapeStore;

type ShareRwLock<T> = Arc<RwLock<T>>;
#[derive(Clone)]
pub struct IpRouteService {
    flow_repo: FlowConfigRepository,
    ipv4_wan_ifaces: ShareRwLock<HashMap<String, RouteTargetInfo>>,
    ipv6_wan_ifaces: ShareRwLock<HashMap<String, RouteTargetInfo>>,

    ipv4_lan_ifaces: ShareRwLock<HashMap<String, LanRouteInfo>>,
    ipv6_lan_ifaces: ShareRwLock<HashMap<LanIPv6RouteKey, LanRouteInfo>>,
}

impl IpRouteService {
    pub fn new(
        mut route_event_sender: mpsc::Receiver<RouteEvent>,
        flow_repo: FlowConfigRepository,
    ) -> Self {
        let service = IpRouteService {
            flow_repo,
            ipv4_wan_ifaces: Arc::new(RwLock::new(HashMap::new())),
            ipv6_wan_ifaces: Arc::new(RwLock::new(HashMap::new())),
            ipv4_lan_ifaces: Arc::new(RwLock::new(HashMap::new())),
            ipv6_lan_ifaces: Arc::new(RwLock::new(HashMap::new())),
        };
        let route_service = service.clone();
        tokio::spawn(async move {
            while let Some(event) = route_event_sender.recv().await {
                use std::option::Option::None;
                //
                match event {
                    RouteEvent::FlowRuleUpdate { flow_id: Some(flow_id) } => {
                        if let Ok(Some(flow_config)) =
                            route_service.flow_repo.find_by_flow_id(flow_id).await
                        {
                            let ipv4_wan_infos = {
                                let read_lock = route_service.ipv4_wan_ifaces.read().await;
                                read_lock.clone()
                            };

                            let ipv6_wan_infos = {
                                let read_lock = route_service.ipv6_wan_ifaces.read().await;
                                read_lock.clone()
                            };

                            let flow_configs = vec![flow_config];
                            refresh_ipv4_target_bpf_map(&flow_configs, ipv4_wan_infos);
                            refresh_ipv6_target_bpf_map(&flow_configs, ipv6_wan_infos);
                        }
                    }
                    RouteEvent::FlowRuleUpdate { flow_id: None } => {
                        let flow_configs = route_service.flow_repo.list().await.unwrap_or_default();
                        let ipv4_wan_infos = {
                            let read_lock = route_service.ipv4_wan_ifaces.read().await;
                            read_lock.clone()
                        };

                        let ipv6_wan_infos = {
                            let read_lock = route_service.ipv6_wan_ifaces.read().await;
                            read_lock.clone()
                        };

                        refresh_ipv4_target_bpf_map(&flow_configs, ipv4_wan_infos);
                        refresh_ipv6_target_bpf_map(&flow_configs, ipv6_wan_infos);
                    }
                }
            }
        });
        service
    }

    pub async fn remove_all_wan_docker(&self) {
        {
            let mut lock = self.ipv4_wan_ifaces.write().await;
            lock.retain(|_, value| !value.is_docker);
        }

        {
            let mut lock = self.ipv6_wan_ifaces.write().await;
            lock.retain(|_, value| !value.is_docker);
        }
    }

    pub async fn print_wan_ifaces(&self) {
        {
            let lock = self.ipv4_wan_ifaces.read().await;
            tracing::info!("ipv4 wan ifaces: {:?}", lock)
        }

        {
            let lock = self.ipv6_wan_ifaces.read().await;
            tracing::info!("ipv6 wan ifaces: {:?}", lock)
        }
    }

    pub async fn print_lan_ifaces(&self) {
        {
            let lock = self.ipv4_lan_ifaces.read().await;
            tracing::info!("ipv4 wan ifaces: {:?}", lock)
        }

        {
            let lock = self.ipv6_lan_ifaces.read().await;
            tracing::info!("ipv6 wan ifaces: {:?}", lock)
        }
    }

    pub async fn insert_ipv6_lan_route(&self, key: LanIPv6RouteKey, new_info: LanRouteInfo) {
        let mut lock = self.ipv6_lan_ifaces.write().await;
        let info = lock.insert(key, new_info.clone());
        drop(lock);
        if let Some(info) = info {
            if info.is_same_subnet(&new_info) {
                del_lan_route(info);
                add_lan_route(new_info);
            }
        } else {
            add_lan_route(new_info);
        }
    }

    pub async fn insert_ipv4_lan_route(&self, key: &str, info: LanRouteInfo) {
        let mut lock = self.ipv4_lan_ifaces.write().await;
        let old_info = lock.insert(key.to_string(), info.clone());
        add_lan_route(info.clone());
        if let Some(old) = old_info {
            if !old.is_same_subnet(&info) {
                del_lan_route(old);
            }
        }
        drop(lock);
    }

    pub async fn remove_ipv6_lan_route(&self, key: &str) {
        let mut lock = self.ipv6_lan_ifaces.write().await;
        let remove_keys: Vec<_> = lock.keys().filter(|k| k.iface_name == key).cloned().collect();

        let mut remove_values = Vec::with_capacity(remove_keys.len());
        for key in remove_keys {
            if let Some(result) = lock.remove(&key) {
                remove_values.push(result);
            }
        }
        drop(lock);

        for info in remove_values {
            del_lan_route(info);
        }
    }

    pub async fn remove_ipv6_lan_route_by_key(&self, key: &LanIPv6RouteKey) {
        let mut lock = self.ipv6_lan_ifaces.write().await;
        if let Some(info) = lock.remove(key) {
            drop(lock);
            del_lan_route(info);
        }
    }

    pub async fn remove_ipv4_lan_route(&self, key: &str) {
        let mut lock = self.ipv4_lan_ifaces.write().await;
        let result = lock.remove(key);
        drop(lock);
        if let Some(info) = result {
            del_lan_route(info);
        }
    }

    pub async fn insert_ipv6_wan_route(&self, key: &str, info: RouteTargetInfo) {
        let mut refresh_default_router = info.default_route;
        let target = info.get_flow_target();
        let mut lock = self.ipv6_wan_ifaces.write().await;
        if let Some(old_info) = lock.insert(key.to_string(), info) {
            refresh_default_router = refresh_default_router || old_info.default_route;
        }
        drop(lock);
        self.refresh_ipv6_target_map(target).await;
        if refresh_default_router {
            self.refresh_default_router().await;
        }
    }

    pub async fn insert_ipv4_wan_route(&self, key: &str, info: RouteTargetInfo) {
        let mut refresh_default_router = info.default_route;
        let target = info.get_flow_target();
        let mut lock = self.ipv4_wan_ifaces.write().await;
        if let Some(old_info) = lock.insert(key.to_string(), info) {
            refresh_default_router = refresh_default_router || old_info.default_route;
        }
        drop(lock);
        self.refresh_ipv4_target_map(target).await;
        if refresh_default_router {
            self.refresh_default_router().await;
        }
    }

    pub async fn remove_ipv4_wan_route(&self, key: &str) {
        let mut lock = self.ipv4_wan_ifaces.write().await;
        let result = lock.remove(key);
        drop(lock);
        if let Some(info) = result {
            self.refresh_ipv4_target_map(info.get_flow_target()).await;
            if info.default_route {
                self.refresh_default_router().await;
            }
        }
    }

    pub async fn remove_ipv6_wan_route(&self, key: &str) {
        let mut lock = self.ipv6_wan_ifaces.write().await;
        let result = lock.remove(key);
        drop(lock);
        if let Some(info) = result {
            self.refresh_ipv6_target_map(info.get_flow_target()).await;
            if info.default_route {
                self.refresh_default_router().await;
            }
        }
    }

    pub async fn refresh_default_router(&self) {
        let wan_ifaces = self.ipv4_wan_ifaces.read().await;
        let route = wan_ifaces.values().find(|e| e.default_route);
        if let Some(route) = route {
            landscape_ebpf::map_setting::route::add_wan_route(0, route.clone());
        }
        drop(wan_ifaces);
        let wan_ifaces = self.ipv6_wan_ifaces.read().await;
        let route = wan_ifaces.values().find(|e| e.default_route);
        if let Some(route) = route {
            landscape_ebpf::map_setting::route::add_wan_route(0, route.clone());
        }
    }

    pub async fn refresh_ipv4_target_map(&self, t: FlowTarget) {
        let flow_configs = self.flow_repo.find_by_target(t).await.unwrap_or_default();

        let ipv4_wan_infos = {
            let read_lock = self.ipv4_wan_ifaces.read().await;
            read_lock.clone()
        };

        refresh_ipv4_target_bpf_map(&flow_configs, ipv4_wan_infos);
    }
    pub async fn refresh_ipv6_target_map(&self, t: FlowTarget) {
        let flow_configs = self.flow_repo.find_by_target(t).await.unwrap_or_default();

        let ipv6_wan_infos = {
            let read_lock = self.ipv6_wan_ifaces.read().await;
            read_lock.clone()
        };

        refresh_ipv6_target_bpf_map(&flow_configs, ipv6_wan_infos);
    }

    pub async fn list_dns_answer_ipv4_addrs(&self) -> Vec<Ipv4Addr> {
        let routes = self.ipv4_lan_ifaces.read().await;
        let mut candidates: Vec<_> = routes
            .values()
            .filter_map(|info| match (&info.mode, info.iface_ip) {
                (LanRouteMode::Reachable, IpAddr::V4(ip)) if is_valid_dns_answer_ipv4(ip) => {
                    Some((info.iface_name.clone(), ip))
                }
                _ => None,
            })
            .collect();
        drop(routes);

        candidates.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.octets().cmp(&b.1.octets())));

        let mut seen = HashSet::new();
        let mut result = Vec::with_capacity(candidates.len());
        for (_, ip) in candidates {
            if seen.insert(ip) {
                result.push(ip);
            }
        }
        result
    }

    pub async fn list_dns_answer_ipv6_addrs(&self) -> Vec<Ipv6Addr> {
        let routes = self.ipv6_lan_ifaces.read().await;
        let mut candidates: Vec<_> = routes
            .values()
            .filter_map(|info| match (&info.mode, info.iface_ip) {
                (LanRouteMode::Reachable, IpAddr::V6(ip)) if is_valid_dns_answer_ipv6(ip) => {
                    Some((info.iface_name.clone(), ip))
                }
                _ => None,
            })
            .collect();
        drop(routes);

        candidates.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.octets().cmp(&b.1.octets())));

        let mut seen = HashSet::new();
        let mut result = Vec::with_capacity(candidates.len());
        for (_, ip) in candidates {
            if seen.insert(ip) {
                result.push(ip);
            }
        }
        result
    }
}

#[async_trait::async_trait]
impl LocalDnsAnswerProvider for IpRouteService {
    async fn list_local_answer_addrs(&self, query_type: RecordType) -> Vec<IpAddr> {
        match query_type {
            RecordType::A => {
                self.list_dns_answer_ipv4_addrs().await.into_iter().map(IpAddr::V4).collect()
            }
            RecordType::AAAA => {
                self.list_dns_answer_ipv6_addrs().await.into_iter().map(IpAddr::V6).collect()
            }
            _ => Vec::new(),
        }
    }
}

fn is_valid_dns_answer_ipv4(ip: Ipv4Addr) -> bool {
    !(ip.is_unspecified() || ip.is_broadcast() || ip.is_multicast() || ip.is_loopback())
}

fn is_valid_dns_answer_ipv6(ip: Ipv6Addr) -> bool {
    !(ip.is_unspecified() || ip.is_multicast() || ip.is_loopback())
}

pub fn refresh_ipv4_target_bpf_map(
    flow_configs: &Vec<FlowConfig>,
    ipv4_wan_infos: HashMap<String, RouteTargetInfo>,
) {
    let mut result: HashMap<FlowId, Vec<RouteTargetInfo>> = HashMap::new();
    for each_flow_config in flow_configs.iter() {
        let mut targets = vec![];
        if each_flow_config.enable {
            for target in each_flow_config.flow_targets.iter() {
                match target {
                    landscape_common::flow::FlowTarget::Interface { name } => {
                        if let Some(result) = ipv4_wan_infos.get(name) {
                            targets.push(result.clone());
                        }
                    }
                    landscape_common::flow::FlowTarget::Netns { container_name } => {
                        if let Some(result) = ipv4_wan_infos.get(container_name) {
                            targets.push(result.clone());
                        }
                    }
                }
            }
        }
        result.insert(each_flow_config.flow_id, targets);
    }

    tracing::info!("ipv4 flow target refresh result: {:#?}", result);
    for (flow_id, configes) in result {
        if let Some(info) = configes.get(0) {
            landscape_ebpf::map_setting::route::add_wan_route(flow_id, info.clone());
        } else {
            landscape_ebpf::map_setting::route::del_ipv4_wan_route(flow_id);
        }
    }
}

pub fn refresh_ipv6_target_bpf_map(
    flow_configs: &Vec<FlowConfig>,
    ipv6_wan_infos: HashMap<String, RouteTargetInfo>,
) {
    // IPV6
    let mut result: HashMap<FlowId, Vec<RouteTargetInfo>> = HashMap::new();
    for each_flow_config in flow_configs.iter() {
        let mut targets = vec![];
        if each_flow_config.enable {
            for target in each_flow_config.flow_targets.iter() {
                match target {
                    landscape_common::flow::FlowTarget::Interface { name } => {
                        if let Some(result) = ipv6_wan_infos.get(name) {
                            targets.push(result.clone());
                        }
                    }
                    landscape_common::flow::FlowTarget::Netns { container_name } => {
                        if let Some(result) = ipv6_wan_infos.get(container_name) {
                            targets.push(result.clone());
                        }
                    }
                }
            }
        }
        result.insert(each_flow_config.flow_id, targets);
    }

    tracing::info!("ipv6 flow target refresh result: {:#?}", result);
    for (flow_id, configes) in result {
        if let Some(info) = configes.get(0) {
            landscape_ebpf::map_setting::route::add_wan_route(flow_id, info.clone());
        } else {
            landscape_ebpf::map_setting::route::del_ipv6_wan_route(flow_id);
        }
    }
}

pub async fn test_used_ip_route() -> (mpsc::Sender<RouteEvent>, IpRouteService) {
    let db_store_provider =
        landscape_database::provider::LandscapeDBServiceProvider::mem_test_db().await;
    let flow_repo = db_store_provider.flow_rule_store();
    let (route_tx, route_rx) = mpsc::channel(1);
    let ip_route = IpRouteService::new(route_rx, flow_repo);
    (route_tx, ip_route)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_async_test(test: impl std::future::Future<Output = ()>) {
        tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(test);
    }

    #[test]
    fn list_dns_answer_ipv4_addrs_filters_invalid_entries_and_next_hop() {
        run_async_test(async {
            let (_tx, service) = test_used_ip_route().await;
            let mut routes = service.ipv4_lan_ifaces.write().await;
            routes.insert(
                "wan0".to_string(),
                LanRouteInfo {
                    ifindex: 1,
                    iface_name: "wan0".to_string(),
                    iface_ip: IpAddr::V4(Ipv4Addr::new(192, 168, 2, 1)),
                    mac: None,
                    prefix: 24,
                    mode: LanRouteMode::Reachable,
                },
            );
            routes.insert(
                "lan0".to_string(),
                LanRouteInfo {
                    ifindex: 2,
                    iface_name: "lan0".to_string(),
                    iface_ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
                    mac: None,
                    prefix: 24,
                    mode: LanRouteMode::Reachable,
                },
            );
            routes.insert(
                "lan0-nexthop".to_string(),
                LanRouteInfo {
                    ifindex: 2,
                    iface_name: "lan0".to_string(),
                    iface_ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 254)),
                    mac: None,
                    prefix: 24,
                    mode: LanRouteMode::NextHop {
                        next_hop_ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2)),
                    },
                },
            );
            routes.insert(
                "loopback".to_string(),
                LanRouteInfo {
                    ifindex: 3,
                    iface_name: "lo".to_string(),
                    iface_ip: IpAddr::V4(Ipv4Addr::LOCALHOST),
                    mac: None,
                    prefix: 8,
                    mode: LanRouteMode::Reachable,
                },
            );
            routes.insert(
                "lan1".to_string(),
                LanRouteInfo {
                    ifindex: 4,
                    iface_name: "lan1".to_string(),
                    iface_ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
                    mac: None,
                    prefix: 24,
                    mode: LanRouteMode::Reachable,
                },
            );
            drop(routes);

            assert_eq!(
                service.list_dns_answer_ipv4_addrs().await,
                vec![Ipv4Addr::new(192, 168, 1, 1), Ipv4Addr::new(192, 168, 2, 1)]
            );
        });
    }

    #[test]
    fn list_dns_answer_ipv6_addrs_keeps_link_local_and_deduplicates() {
        run_async_test(async {
            let (_tx, service) = test_used_ip_route().await;
            let mut routes = service.ipv6_lan_ifaces.write().await;
            routes.insert(
                LanIPv6RouteKey { iface_name: "lan0".to_string(), subnet_index: 0 },
                LanRouteInfo {
                    ifindex: 1,
                    iface_name: "lan0".to_string(),
                    iface_ip: IpAddr::V6(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1)),
                    mac: None,
                    prefix: 64,
                    mode: LanRouteMode::Reachable,
                },
            );
            routes.insert(
                LanIPv6RouteKey { iface_name: "lan1".to_string(), subnet_index: 0 },
                LanRouteInfo {
                    ifindex: 2,
                    iface_name: "lan1".to_string(),
                    iface_ip: IpAddr::V6(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1)),
                    mac: None,
                    prefix: 64,
                    mode: LanRouteMode::Reachable,
                },
            );
            routes.insert(
                LanIPv6RouteKey { iface_name: "lan2".to_string(), subnet_index: 0 },
                LanRouteInfo {
                    ifindex: 3,
                    iface_name: "lan2".to_string(),
                    iface_ip: IpAddr::V6(Ipv6Addr::UNSPECIFIED),
                    mac: None,
                    prefix: 64,
                    mode: LanRouteMode::Reachable,
                },
            );
            routes.insert(
                LanIPv6RouteKey { iface_name: "lan3".to_string(), subnet_index: 0 },
                LanRouteInfo {
                    ifindex: 4,
                    iface_name: "lan3".to_string(),
                    iface_ip: IpAddr::V6(Ipv6Addr::LOCALHOST),
                    mac: None,
                    prefix: 128,
                    mode: LanRouteMode::Reachable,
                },
            );
            routes.insert(
                LanIPv6RouteKey { iface_name: "lan4".to_string(), subnet_index: 0 },
                LanRouteInfo {
                    ifindex: 5,
                    iface_name: "lan4".to_string(),
                    iface_ip: IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1)),
                    mac: None,
                    prefix: 64,
                    mode: LanRouteMode::NextHop {
                        next_hop_ip: IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 2)),
                    },
                },
            );
            drop(routes);

            assert_eq!(
                service.list_dns_answer_ipv6_addrs().await,
                vec![Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1)]
            );
        });
    }
}
