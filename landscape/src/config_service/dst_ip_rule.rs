use landscape_common::{
    ip_mark::WanIpRuleConfig,
    service::controller_service::{ConfigController, FlowConfigController},
};
use landscape_database::{
    dst_ip_rule::repository::DstIpRuleRepository, provider::LandscapeDBServiceProvider,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct DstIpRuleService {
    store: DstIpRuleRepository,
}

impl DstIpRuleService {
    pub fn new(store: LandscapeDBServiceProvider) -> Self {
        let store = store.dst_ip_rule_store();
        Self { store }
    }
}

impl FlowConfigController for DstIpRuleService {}

impl ConfigController for DstIpRuleService {
    type Id = Uuid;

    type Config = WanIpRuleConfig;

    type DatabseAction = DstIpRuleRepository;

    fn get_repository(&self) -> &Self::DatabseAction {
        &self.store
    }
}
