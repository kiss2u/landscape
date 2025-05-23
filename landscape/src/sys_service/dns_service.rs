use landscape_common::{
    event::dns::DnsEvent,
    service::{controller_service::ConfigController, DefaultWatchServiceStatus},
};
use landscape_dns::diff_server::LandscapeFiffFlowDnsService;
use tokio::sync::mpsc;

use crate::config_service::{dns_rule::DNSRuleService, flow_rule::FlowRuleService};

#[derive(Clone)]
pub struct LandscapeDnsService {
    dns_service: LandscapeFiffFlowDnsService,
    dns_rule_service: DNSRuleService,
    flow_rule_service: FlowRuleService,
    // geo_service: GeoService,
}

impl LandscapeDnsService {
    pub async fn new(
        mut receiver: mpsc::Receiver<DnsEvent>,
        dns_rule_service: DNSRuleService,
        flow_rule_service: FlowRuleService,
    ) -> Self {
        let dns_service = LandscapeFiffFlowDnsService::new().await;

        dns_service.restart(53).await;
        dns_service.init_handle(dns_rule_service.list().await).await;
        dns_service.update_flow_map(&flow_rule_service.list().await).await;

        let dns_rule_service_clone = dns_rule_service.clone();
        let flow_rule_service_clone = flow_rule_service.clone();
        let dns_service_clone = dns_service.clone();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                match event {
                    DnsEvent::RuleUpdated | DnsEvent::GeositeUpdated => {
                        tracing::info!("refresh dns rule");
                        let dns_rules = dns_rule_service_clone.list().await;
                        let flow_rules = flow_rule_service_clone.list().await;
                        dns_service_clone.init_handle(dns_rules).await;
                        dns_service_clone.update_flow_map(&flow_rules).await;
                        // dns_service_clone.restart(53).await;
                    }
                }
            }
        });
        Self { dns_service, dns_rule_service, flow_rule_service }
    }

    // pub async fn get_event_sender(&self) -> mpsc::Sender<DnsEvent> {
    //     self.dns_events_tx.clone()
    // }

    pub async fn get_status(&self) -> DefaultWatchServiceStatus {
        self.dns_service.status.clone()
    }

    pub async fn start_dns_service(&self) {
        let dns_rules = self.dns_rule_service.list().await;
        let flow_rules = self.flow_rule_service.list().await;
        // TODO 重置 Flow 相关 map 信息

        self.dns_service.init_handle(dns_rules).await;
        self.dns_service.update_flow_map(&flow_rules).await;
        self.dns_service.restart(53).await;
    }

    pub async fn stop(&self) {
        self.dns_service.stop();
    }
}
