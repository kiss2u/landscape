use landscape_common::{
    config::dns::DNSRuleConfig,
    service::controller_service::{ConfigController, FlowConfigController},
};
use landscape_database::{
    dns_rule::repository::DNSRuleRepository, provider::LandscapeDBServiceProvider,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct DNSRuleService {
    store: DNSRuleRepository,
}

impl DNSRuleService {
    pub fn new(store: LandscapeDBServiceProvider) -> Self {
        let store = store.dns_store();
        Self { store }
    }
}

impl FlowConfigController for DNSRuleService {}

impl ConfigController for DNSRuleService {
    type Id = Uuid;

    type Config = DNSRuleConfig;

    type DatabseAction = DNSRuleRepository;

    fn get_repository(&self) -> &Self::DatabseAction {
        &self.store
    }
}
