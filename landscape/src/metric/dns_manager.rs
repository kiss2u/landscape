pub use crate::metric::duckdb::DuckMetricStore;
use landscape_common::event::DnsMetricMessage;
use landscape_common::metric::dns::{DnsHistoryQueryParams, DnsHistoryResponse, DnsMetric, DnsSummaryResponse};
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct DnsMetricManager {
    metric_store: DuckMetricStore,
    msg_tx: mpsc::Sender<DnsMetricMessage>,
}

impl DnsMetricManager {
    pub fn with_store(metric_store: DuckMetricStore) -> Self {
        let (msg_tx, mut msg_rx) = mpsc::channel::<DnsMetricMessage>(1024);
        let store_clone = metric_store.clone();

        tokio::spawn(async move {
            while let Some(msg) = msg_rx.recv().await {
                match msg {
                    DnsMetricMessage::Metric(metric) => {
                        store_clone.insert_dns_metric(metric).await;
                    }
                }
            }
        });

        DnsMetricManager { metric_store, msg_tx }
    }

    pub fn get_msg_channel(&self) -> mpsc::Sender<DnsMetricMessage> {
        self.msg_tx.clone()
    }

    pub async fn insert_dns_metric(&self, metric: DnsMetric) {
        self.metric_store.insert_dns_metric(metric).await;
    }

    pub async fn query_dns_history(&self, params: DnsHistoryQueryParams) -> DnsHistoryResponse {
        self.metric_store.query_dns_history(params).await
    }

    pub async fn get_dns_summary(&self, params: DnsHistoryQueryParams) -> DnsSummaryResponse {
        self.metric_store.get_dns_summary(params).await
    }
}
