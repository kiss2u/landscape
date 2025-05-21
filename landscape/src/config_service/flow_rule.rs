use landscape_common::{flow::FlowConfig, service::controller_service::ConfigController};
use landscape_database::{
    flow_rule::repository::FlowConfigRepository, provider::LandscapeDBServiceProvider,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct FlowRuleService {
    store: FlowConfigRepository,
}

impl FlowRuleService {
    pub fn new(store: LandscapeDBServiceProvider) -> Self {
        let store = store.flow_rule_store();
        Self { store }
    }
}

impl ConfigController for FlowRuleService {
    type Id = Uuid;
    type Config = FlowConfig;
    type DatabseAction = FlowConfigRepository;

    fn get_repository(&self) -> &Self::DatabseAction {
        &self.store
    }
}
