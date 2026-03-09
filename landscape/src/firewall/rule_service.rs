use landscape_common::{
    firewall::{insert_default_firewall_rule, FirewallRuleConfig},
    service::controller::ConfigController,
};
use landscape_database::{
    firewall_rule::repository::FirewallRuleRepository, provider::LandscapeDBServiceProvider,
};
use uuid::Uuid;

use crate::firewall::rules::update_firewall_rules;

#[derive(Clone)]
pub struct FirewallRuleService {
    store: FirewallRuleRepository,
}

impl FirewallRuleService {
    pub async fn new(store: LandscapeDBServiceProvider) -> Self {
        let store = store.firewall_rule_store();
        let firewall_rule_service = Self { store };
        let mut rules = firewall_rule_service.list().await;

        if rules.is_empty() {
            // 规则为空时插入默认规则
            if let Some(rule) = insert_default_firewall_rule() {
                firewall_rule_service.set(rule).await;
            }
            rules = firewall_rule_service.list().await;
        }

        update_firewall_rules(rules, vec![]);
        firewall_rule_service
    }
}

#[async_trait::async_trait]
impl ConfigController for FirewallRuleService {
    type Id = Uuid;

    type Config = FirewallRuleConfig;

    type DatabseAction = FirewallRuleRepository;

    fn get_repository(&self) -> &Self::DatabseAction {
        &self.store
    }

    async fn after_update_config(
        &self,
        mut firewall_rules: Vec<Self::Config>,
        mut old_configs: Vec<Self::Config>,
    ) {
        firewall_rules.sort_by(|a, b| a.index.cmp(&b.index));
        old_configs.sort_by(|a, b| a.index.cmp(&b.index));
        update_firewall_rules(firewall_rules, old_configs);
    }
}
