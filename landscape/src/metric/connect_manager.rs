use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use landscape_common::config::MetricRuntimeConfig;
use tokio::sync::{mpsc, RwLock};

use landscape_common::event::ConnectMessage;
use landscape_common::metric::connect::{
    ConnectGlobalStats, ConnectHistoryQueryParams, ConnectHistoryStatus, ConnectKey, ConnectMetric,
    ConnectRealtimeStatus, ConnectStatusType,
};

use crate::metric::duckdb::DuckMetricStore;

#[derive(Clone)]
pub struct ConnectMetricManager {
    /// 实时速率缓存: Key -> 当前算出的状态
    /// 我们只保留活跃的连接在内存中
    realtime_metrics: Arc<RwLock<HashMap<ConnectKey, ConnectRealtimeStatus>>>,
    msg_channel: mpsc::Sender<ConnectMessage>,
    metric_store: DuckMetricStore,
    global_stats: Arc<RwLock<ConnectGlobalStats>>,
}

impl ConnectMetricManager {
    pub fn with_store(metric_store: DuckMetricStore) -> Self {
        let realtime_metrics = Arc::new(RwLock::new(HashMap::new()));
        let (msg_channel, mut message_rx) = mpsc::channel(1024);

        let metric_store_clone = metric_store.clone();
        let realtime_metrics_clone = realtime_metrics.clone();

        tokio::spawn(async move {
            while let Some(msg) = message_rx.recv().await {
                match msg {
                    ConnectMessage::Metric(metric) => {
                        // 1. 更新数据库 (异步发送到 DB 线程)
                        metric_store_clone.insert_metric(metric.clone()).await;

                        // 2. 更新内存中的活跃状态和实时速率
                        let mut realtime_set = realtime_metrics_clone.write().await;

                        if metric.status == ConnectStatusType::Disabled {
                            realtime_set.remove(&metric.key);
                        } else if metric.status == ConnectStatusType::Active {
                            let key = metric.key.clone();
                            let status = realtime_set.entry(key.clone()).or_insert_with(|| {
                                ConnectRealtimeStatus {
                                    key,
                                    src_ip: metric.src_ip,
                                    dst_ip: metric.dst_ip,
                                    src_port: metric.src_port,
                                    dst_port: metric.dst_port,
                                    l4_proto: metric.l4_proto,
                                    l3_proto: metric.l3_proto,
                                    flow_id: metric.flow_id,
                                    trace_id: metric.trace_id,
                                    ingress_bps: 0,
                                    egress_bps: 0,
                                    ingress_pps: 0,
                                    egress_pps: 0,
                                    last_metric: None,
                                }
                            });

                            if let Some(old_metric) = &status.last_metric {
                                let delta_time_ms =
                                    metric.report_time.saturating_sub(old_metric.report_time);
                                if delta_time_ms > 0 {
                                    // 速率计算公式: (delta_bytes * 8位 / (ms/1000)) = delta_bytes * 8000 / ms
                                    status.ingress_bps = (metric
                                        .ingress_bytes
                                        .saturating_sub(old_metric.ingress_bytes)
                                        * 8000)
                                        / delta_time_ms;
                                    status.egress_bps = (metric
                                        .egress_bytes
                                        .saturating_sub(old_metric.egress_bytes)
                                        * 8000)
                                        / delta_time_ms;
                                    status.ingress_pps = (metric
                                        .ingress_packets
                                        .saturating_sub(old_metric.ingress_packets)
                                        * 1000)
                                        / delta_time_ms;
                                    status.egress_pps = (metric
                                        .egress_packets
                                        .saturating_sub(old_metric.egress_packets)
                                        * 1000)
                                        / delta_time_ms;
                                }
                            }
                            status.last_metric = Some(metric);
                        }
                    }
                }
            }
        });

        let global_stats = Arc::new(RwLock::new(ConnectGlobalStats::default()));

        // 定时汇总全量统计 (每 24 小时)
        {
            let metric_store_clone = metric_store.clone();
            let global_stats_clone = global_stats.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(86400));
                loop {
                    interval.tick().await;
                    let stats = metric_store_clone.get_global_stats().await;
                    let mut lock = global_stats_clone.write().await;
                    *lock = stats;
                    tracing::info!("Global stats updated: {} connects", lock.total_connect_count);
                }
            });
        }

        ConnectMetricManager {
            realtime_metrics,
            msg_channel,
            metric_store,
            global_stats,
        }
    }

    pub async fn new(base_path: PathBuf, config: MetricRuntimeConfig) -> Self {
        let metric_store = DuckMetricStore::new(base_path, config).await;
        Self::with_store(metric_store)
    }

    pub async fn get_global_stats(&self) -> ConnectGlobalStats {
        self.global_stats.read().await.clone()
    }

    pub fn get_msg_channel(&self) -> mpsc::Sender<ConnectMessage> {
        self.msg_channel.clone()
    }

    pub fn send_connect_msg(&self, msg: ConnectMessage) {
        if let Err(e) = self.msg_channel.try_send(msg) {
            tracing::error!("send firewall metric error: {e:?}");
        }
    }

    pub async fn connect_infos(&self) -> Vec<ConnectRealtimeStatus> {
        let realtime_metrics = self.realtime_metrics.read().await;
        let mut result: Vec<ConnectRealtimeStatus> = realtime_metrics.values().cloned().collect();
        // 按创建时间排序
        result.sort_by_key(|r| r.key.create_time);
        result
    }

    pub async fn query_metric_by_key(&self, key: ConnectKey) -> Vec<ConnectMetric> {
        self.metric_store.query_metric_by_key(key).await
    }

    pub async fn history_summaries_complex(
        &self,
        params: ConnectHistoryQueryParams,
    ) -> Vec<ConnectHistoryStatus> {
        self.metric_store.history_summaries_complex(params).await
    }
}
