use landscape_common::{flow::FlowConfig, service::controller_service::ConfigController};
use landscape_database::{
    flow_rule::repository::FlowConfigRepository, provider::LandscapeDBServiceProvider,
};
use uuid::Uuid;

use crate::flow::update_flow_matchs;

#[derive(Clone)]
pub struct FlowRuleService {
    store: FlowConfigRepository,
}

impl FlowRuleService {
    pub async fn new(store: LandscapeDBServiceProvider) -> Self {
        let store = store.flow_rule_store();
        let result = Self { store };
        result.after_update_config(result.list().await, vec![]).await;
        result
    }
}

#[async_trait::async_trait]
impl ConfigController for FlowRuleService {
    type Id = Uuid;
    type Config = FlowConfig;
    type DatabseAction = FlowConfigRepository;

    fn get_repository(&self) -> &Self::DatabseAction {
        &self.store
    }

    async fn after_update_config(
        &self,
        new_configs: Vec<Self::Config>,
        old_configs: Vec<Self::Config>,
    ) {
        update_flow_matchs(new_configs, old_configs).await;
    }
}
