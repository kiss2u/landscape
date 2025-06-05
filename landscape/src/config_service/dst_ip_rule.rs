use std::collections::{HashMap, HashSet};

use landscape_common::{
    event::dns::DstIpEvent,
    ip_mark::WanIpRuleConfig,
    service::controller_service::{ConfigController, FlowConfigController},
};
use landscape_database::{
    dst_ip_rule::repository::DstIpRuleRepository, provider::LandscapeDBServiceProvider,
};
use tokio::sync::mpsc;
use uuid::Uuid;

use super::geo_ip_service::GeoIpService;

#[derive(Clone)]
pub struct DstIpRuleService {
    store: DstIpRuleRepository,
    geo_ip_service: GeoIpService,
}

impl DstIpRuleService {
    pub async fn new(
        store: LandscapeDBServiceProvider,
        geo_ip_service: GeoIpService,
        mut receiver: mpsc::Receiver<DstIpEvent>,
    ) -> Self {
        let store = store.dst_ip_rule_store();
        let dst_ip_rule_service = Self { store, geo_ip_service };
        dst_ip_rule_service.after_update_config(dst_ip_rule_service.list().await, vec![]).await;
        let dst_ip_rule_service_clone = dst_ip_rule_service.clone();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                match event {
                    DstIpEvent::GeoIpUpdated => {
                        tracing::info!("refresh dst ip rule");
                        dst_ip_rule_service_clone
                            .after_update_config(dst_ip_rule_service_clone.list().await, vec![])
                            .await;
                    }
                }
            }
        });

        dst_ip_rule_service
    }
}

impl FlowConfigController for DstIpRuleService {}

#[async_trait::async_trait]
impl ConfigController for DstIpRuleService {
    type Id = Uuid;

    type Config = WanIpRuleConfig;

    type DatabseAction = DstIpRuleRepository;

    fn get_repository(&self) -> &Self::DatabseAction {
        &self.store
    }

    async fn after_update_config(
        &self,
        mut new_configs: Vec<Self::Config>,
        _old_configs: Vec<Self::Config>,
    ) {
        new_configs.sort_by(|a, b| a.index.cmp(&b.index));
        let mut flow_ids = HashSet::new();
        let mut rule_map: HashMap<u32, Vec<WanIpRuleConfig>> = HashMap::new();

        for r in new_configs.into_iter() {
            if !flow_ids.contains(&r.flow_id) {
                flow_ids.insert(r.flow_id.clone());
            }
            match rule_map.entry(r.flow_id.clone()) {
                std::collections::hash_map::Entry::Occupied(mut entry) => entry.get_mut().push(r),
                std::collections::hash_map::Entry::Vacant(entry) => {
                    entry.insert(vec![r]);
                }
            }
        }

        for flow_id in flow_ids {
            let rules = rule_map.remove(&flow_id).unwrap_or_default();
            let result = self.geo_ip_service.convert_config_to_runtime_rule(rules).await;
            landscape_ebpf::map_setting::flow_wanip::add_wan_ip_mark(flow_id, result);
        }
    }
}
