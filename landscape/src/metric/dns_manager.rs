use crate::metric::MetricStore;
use landscape_common::event::DnsMetricMessage;
use landscape_common::metric::dns::{
    DnsHistoryQueryParams, DnsHistoryResponse, DnsLightweightSummaryResponse, DnsMetric,
    DnsSummaryQueryParams, DnsSummaryResponse,
};
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct DnsMetricManager {
    metric_store: MetricStore,
}

impl DnsMetricManager {
    pub fn with_store(metric_store: MetricStore) -> Self {
        DnsMetricManager { metric_store }
    }

    pub fn get_msg_channel(&self) -> mpsc::Sender<DnsMetricMessage> {
        self.metric_store.get_dns_msg_channel()
    }

    pub async fn insert_dns_metric(&self, metric: DnsMetric) {
        self.metric_store.insert_dns_metric(metric).await;
    }

    pub async fn query_dns_history(&self, params: DnsHistoryQueryParams) -> DnsHistoryResponse {
        self.metric_store.query_dns_history(params).await
    }

    pub async fn get_dns_summary(&self, params: DnsSummaryQueryParams) -> DnsSummaryResponse {
        self.metric_store.get_dns_summary(params).await
    }

    pub async fn get_dns_lightweight_summary(
        &self,
        params: DnsSummaryQueryParams,
    ) -> DnsLightweightSummaryResponse {
        self.metric_store.get_dns_lightweight_summary(params).await
    }
}
