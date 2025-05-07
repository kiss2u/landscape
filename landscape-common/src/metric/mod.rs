use firewall::FirewallMetricService;

pub mod firewall;

#[derive(Clone)]
pub struct MetricService {
    pub firewall: FirewallMetricService,
}

impl MetricService {
    pub async fn new() -> Self {
        MetricService { firewall: FirewallMetricService::new().await }
    }
}
