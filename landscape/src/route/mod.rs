use std::{collections::HashMap, sync::Arc};

use landscape_common::{
    config::FlowId,
    flow::FlowConfig,
    route::{LanRouteInfo, RouteTargetInfo},
};
use landscape_database::flow_rule::repository::FlowConfigRepository;
use landscape_ebpf::map_setting::route::{add_lan_route, add_wan_route, del_lan_route};
use tokio::sync::RwLock;

use landscape_common::database::LandscapeDBTrait;

#[derive(Clone)]
pub struct IpRouteService {
    flow_repo: FlowConfigRepository,
    wan_ifaces: Arc<RwLock<HashMap<String, RouteTargetInfo>>>,
    lan_ifaces: Arc<RwLock<HashMap<String, LanRouteInfo>>>,
}

impl IpRouteService {
    pub fn new(flow_repo: FlowConfigRepository) -> Self {
        IpRouteService {
            flow_repo,
            wan_ifaces: Arc::new(RwLock::new(HashMap::new())),
            lan_ifaces: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn insert_lan_route(&self, key: &str, info: LanRouteInfo) {
        let mut lock = self.lan_ifaces.write().await;
        add_lan_route(info.clone());
        lock.insert(key.to_string(), info);
    }

    pub async fn remove_lan_route(&self, key: &str) {
        let mut lock = self.lan_ifaces.write().await;
        let result = lock.remove(key);
        if let Some(info) = result {
            del_lan_route(info);
        }
    }

    pub async fn insert_wan_route(&self, key: &str, info: RouteTargetInfo) {
        let mut lock = self.wan_ifaces.write().await;
        lock.insert(key.to_string(), info);
        self.refreash_target_map().await;
    }

    pub async fn remove_wan_route(&self, key: &str) {
        let mut lock = self.wan_ifaces.write().await;
        let result = lock.remove(key);
        if let Some(_) = result {
            self.refreash_target_map().await;
        }
    }

    pub async fn refreash_target_map(&self) {
        let flow_configs = self.flow_repo.list().await.unwrap_or_default();
        let wan_infos = {
            let read_lock = self.wan_ifaces.read().await;
            read_lock.clone()
        };

        refresh_target_bpf_map(flow_configs, wan_infos);
    }
}

pub fn refresh_target_bpf_map(
    flow_configs: Vec<FlowConfig>,
    wan_infos: HashMap<String, RouteTargetInfo>,
) {
    let mut result: HashMap<FlowId, Vec<RouteTargetInfo>> = HashMap::new();
    for each_flow_config in flow_configs {
        let mut targets = vec![];
        if each_flow_config.enable {
            for target in each_flow_config.flow_targets {
                match target {
                    landscape_common::flow::FlowTarget::Interface { name } => {
                        if let Some(result) = wan_infos.get(&name) {
                            targets.push(result.clone());
                        }
                    }
                    landscape_common::flow::FlowTarget::Netns { container_name } => {
                        if let Some(result) = wan_infos.get(&container_name) {
                            targets.push(result.clone());
                        }
                    }
                }
            }
        }
        result.insert(each_flow_config.flow_id, targets);
    }

    for (flow_id, configes) in result {
        if let Some(info) = configes.get(0) {
            add_wan_route(flow_id, info.clone());
        }
    }
}
