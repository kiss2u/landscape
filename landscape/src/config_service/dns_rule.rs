use landscape_common::{
    config::dns::DNSRuleConfig,
    event::dns::DnsEvent,
    service::controller_service::{ConfigController, FlowConfigController},
};
use landscape_database::{
    dns_rule::repository::DNSRuleRepository, provider::LandscapeDBServiceProvider,
};
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Clone)]
pub struct DNSRuleService {
    store: DNSRuleRepository,
    dns_events_tx: mpsc::Sender<DnsEvent>,
}

impl DNSRuleService {
    pub async fn new(
        store: LandscapeDBServiceProvider,
        dns_events_tx: mpsc::Sender<DnsEvent>,
    ) -> Self {
        let store = store.dns_rule_store();
        Self { store, dns_events_tx }
    }
}

impl FlowConfigController for DNSRuleService {}

#[async_trait::async_trait]
impl ConfigController for DNSRuleService {
    type Id = Uuid;

    type Config = DNSRuleConfig;

    type DatabseAction = DNSRuleRepository;

    fn get_repository(&self) -> &Self::DatabseAction {
        &self.store
    }

    async fn after_update_config(
        &self,
        _new_configs: Vec<Self::Config>,
        _old_configs: Vec<Self::Config>,
    ) {
        let _ = self.dns_events_tx.send(DnsEvent::RuleUpdated).await;
    }
}
