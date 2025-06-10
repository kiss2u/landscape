use std::time::Instant;

use landscape_common::{
    event::dns::DnsEvent,
    service::{
        controller_service::{ConfigController, FlowConfigController},
        DefaultWatchServiceStatus,
    },
};
use landscape_dns::diff_server::{CheckDnsReq, CheckDnsResult, LandscapeFiffFlowDnsService};
use tokio::sync::mpsc;

use crate::config_service::{
    dns_rule::DNSRuleService, flow_rule::FlowRuleService, geo_site_service::GeoSiteService,
};

#[derive(Clone)]
pub struct LandscapeDnsService {
    dns_service: LandscapeFiffFlowDnsService,
    dns_rule_service: DNSRuleService,
    flow_rule_service: FlowRuleService,
    geo_site_service: GeoSiteService,
}

impl LandscapeDnsService {
    pub async fn new(
        mut receiver: mpsc::Receiver<DnsEvent>,
        dns_rule_service: DNSRuleService,
        flow_rule_service: FlowRuleService,
        geo_site_service: GeoSiteService,
    ) -> Self {
        let dns_service = LandscapeFiffFlowDnsService::new().await;
        let dns_rules = dns_rule_service.list().await;
        let dns_rules = geo_site_service.convert_config_to_runtime_rule(dns_rules).await;

        dns_service.restart(53).await;
        dns_service.init_handle(dns_rules).await;
        dns_service.update_flow_map(&flow_rule_service.list().await).await;

        let dns_rule_service_clone = dns_rule_service.clone();
        let flow_rule_service_clone = flow_rule_service.clone();
        let dns_service_clone = dns_service.clone();
        let geo_site_service_clone = geo_site_service.clone();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                match event {
                    DnsEvent::RuleUpdated { flow_id: None } | DnsEvent::GeositeUpdated => {
                        tracing::info!("refresh dns rule");
                        let time = Instant::now();
                        let dns_rules = dns_rule_service_clone.list().await;
                        let flow_rules = flow_rule_service_clone.list().await;

                        tracing::info!("load rule: {:?}", time.elapsed().as_secs());
                        let dns_rules =
                            geo_site_service_clone.convert_config_to_runtime_rule(dns_rules).await;

                        tracing::info!("convert rule: {:?}", time.elapsed().as_secs());
                        dns_service_clone.init_handle(dns_rules).await;
                        tracing::info!("init rule: {:?}", time.elapsed().as_secs());

                        dns_service_clone.update_flow_map(&flow_rules).await;
                        tracing::info!("update flow: {:?}", time.elapsed().as_secs());
                        // dns_service_clone.restart(53).await;
                    }
                    DnsEvent::RuleUpdated { flow_id: Some(flow_id) } => {
                        tracing::info!("refresh dns rule");
                        let time = Instant::now();
                        let flow_dns_rules =
                            dns_rule_service_clone.list_flow_configs(flow_id).await;
                        let flow_rules = flow_rule_service_clone.list_flow_configs(flow_id).await;

                        tracing::info!("load rule: {:?}", time.elapsed().as_secs());
                        let dns_rules = geo_site_service_clone
                            .convert_config_to_runtime_rule(flow_dns_rules)
                            .await;

                        tracing::info!("convert rule: {:?}", time.elapsed().as_secs());
                        dns_service_clone.init_handle(dns_rules).await;
                        tracing::info!("init rule: {:?}", time.elapsed().as_secs());

                        dns_service_clone.update_flow_map(&flow_rules).await;
                        tracing::info!("update flow: {:?}", time.elapsed().as_secs());
                    }
                }
            }
        });
        Self {
            dns_service,
            dns_rule_service,
            flow_rule_service,
            geo_site_service,
        }
    }

    pub async fn get_status(&self) -> DefaultWatchServiceStatus {
        self.dns_service.status.clone()
    }

    pub async fn start_dns_service(&self) {
        let dns_rules = self.dns_rule_service.list().await;
        let flow_rules = self.flow_rule_service.list().await;
        let dns_rules = self.geo_site_service.convert_config_to_runtime_rule(dns_rules).await;
        // TODO 重置 Flow 相关 map 信息
        self.dns_service.init_handle(dns_rules).await;
        self.dns_service.update_flow_map(&flow_rules).await;
        self.dns_service.restart(53).await;
    }

    pub async fn stop(&self) {
        self.dns_service.stop();
    }

    pub async fn check_domain(&self, req: CheckDnsReq) -> CheckDnsResult {
        self.dns_service.check_domain(req).await
    }
}
