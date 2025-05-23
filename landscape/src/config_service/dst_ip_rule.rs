use landscape_common::{
    args::LAND_HOME_PATH,
    ip_mark::WanIpRuleConfig,
    service::controller_service::{ConfigController, FlowConfigController},
    GEO_IP_FILE_NAME,
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
    pub async fn new(store: LandscapeDBServiceProvider) -> Self {
        let store = store.dst_ip_rule_store();
        let result = Self { store };
        result.after_update_config(result.list().await, vec![]).await;
        result
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
        new_configs: Vec<Self::Config>,
        old_configs: Vec<Self::Config>,
    ) {
        landscape_dns::ip_rule::update_wan_rules(
            new_configs,
            old_configs,
            // TODO 将 GEO 变为服务进行组合到 self
            LAND_HOME_PATH.join(GEO_IP_FILE_NAME),
            None,
        )
        .await;
    }
}
