use landscape_common::{
    firewall::FirewallRuleConfig, service::controller_service::ConfigController,
};
use landscape_database::{
    firewall_rule::repository::FirewallRuleRepository, provider::LandscapeDBServiceProvider,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct FirewallRuleService {
    store: FirewallRuleRepository,
}

impl FirewallRuleService {
    pub fn new(store: LandscapeDBServiceProvider) -> Self {
        let store = store.firewall_rule_store();
        Self { store }
    }
}

impl ConfigController for FirewallRuleService {
    type Id = Uuid;

    type Config = FirewallRuleConfig;

    type DatabseAction = FirewallRuleRepository;

    fn get_repository(&self) -> &Self::DatabseAction {
        &self.store
    }
}
