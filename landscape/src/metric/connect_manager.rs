use tokio::sync::mpsc;

use landscape_common::event::ConnectMessage;
use landscape_common::metric::connect::{
    ConnectGlobalStats, ConnectHistoryQueryParams, ConnectHistoryStatus, ConnectKey,
    ConnectMetricPoint, ConnectRealtimeStatus, IpRealtimeStat, MetricResolution,
};

use crate::metric::MetricStore;

#[derive(Clone)]
pub struct ConnectMetricManager {
    metric_store: MetricStore,
}

impl ConnectMetricManager {
    pub fn with_store(metric_store: MetricStore) -> Self {
        ConnectMetricManager { metric_store }
    }

    pub async fn get_global_stats(&self) -> ConnectGlobalStats {
        self.metric_store.get_global_stats().await
    }

    pub async fn get_src_ip_stats(&self) -> Vec<IpRealtimeStat> {
        self.metric_store.get_realtime_ip_stats(true).await
    }

    pub async fn get_dst_ip_stats(&self) -> Vec<IpRealtimeStat> {
        self.metric_store.get_realtime_ip_stats(false).await
    }

    pub fn get_msg_channel(&self) -> mpsc::Sender<ConnectMessage> {
        self.metric_store.get_connect_msg_channel()
    }

    pub async fn connect_infos(&self) -> Vec<ConnectRealtimeStatus> {
        self.metric_store.connect_infos().await
    }

    pub async fn query_metric_by_key(
        &self,
        key: ConnectKey,
        resolution: Option<MetricResolution>,
    ) -> Vec<ConnectMetricPoint> {
        let resolution = resolution.unwrap_or(MetricResolution::Second);

        self.metric_store.query_metric_by_key(key, resolution).await
    }

    pub async fn history_summaries_complex(
        &self,
        params: ConnectHistoryQueryParams,
    ) -> Vec<ConnectHistoryStatus> {
        self.metric_store.history_summaries_complex(params).await
    }

    pub async fn history_src_ip_stats(
        &self,
        params: ConnectHistoryQueryParams,
    ) -> Vec<landscape_common::metric::connect::IpHistoryStat> {
        self.metric_store.history_src_ip_stats(params).await
    }

    pub async fn history_dst_ip_stats(
        &self,
        params: ConnectHistoryQueryParams,
    ) -> Vec<landscape_common::metric::connect::IpHistoryStat> {
        self.metric_store.history_dst_ip_stats(params).await
    }
}
