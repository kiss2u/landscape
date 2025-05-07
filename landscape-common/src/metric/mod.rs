use firewall::FirewallMetricService;

pub mod firewall;

#[derive(Clone)]
pub struct MetricData {
    pub firewall: FirewallMetricService,
}

impl MetricData {
    pub async fn new() -> Self {
        MetricData { firewall: FirewallMetricService::new().await }
    }
}
