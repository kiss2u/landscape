use landscape_common::{
    config::dns::DNSRuleConfig,
    service::controller_service::{ConfigController, FlowConfigController},
};
use landscape_database::{
    dns_rule::repository::DNSConfigRepository, provider::LandscapeDBServiceProvider,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct DNSConfigService {
    store: DNSConfigRepository,
}

impl DNSConfigService {
    pub fn new(store: LandscapeDBServiceProvider) -> Self {
        let store = store.dns_store();
        Self { store }
    }
}

impl FlowConfigController for DNSConfigService {}

impl ConfigController for DNSConfigService {
    type Id = Uuid;

    type Config = DNSRuleConfig;

    type DatabseAction = DNSConfigRepository;

    fn get_repository(&self) -> &Self::DatabseAction {
        &self.store
    }
}
