use std::collections::HashMap;
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::Arc;

use landscape_common::config::MetricRuntimeConfig;
use tokio::sync::{mpsc, RwLock};

use landscape_common::event::ConnectMessage;
use landscape_common::metric::connect::{
    ConnectGlobalStats, ConnectHistoryQueryParams, ConnectHistoryStatus, ConnectKey, ConnectMetric,
    ConnectRealtimeStatus, ConnectStatusType, IpAggregatedStats, IpRealtimeStat, MetricResolution,
};

use crate::metric::duckdb::DuckMetricStore;

const MAX_REPORT_INTERVAL_MS: u64 = 5000;
const HOUR_RESOLUTION_THRESHOLD_MS: u64 = 30 * 24 * 3600 * 1000;
const DAY_RESOLUTION_THRESHOLD_MS: u64 = 180 * 24 * 3600 * 1000;

#[derive(Clone)]
pub struct ConnectMetricManager {
    realtime_metrics: Arc<RwLock<HashMap<ConnectKey, ConnectRealtimeStatus>>>,
    src_ip_stats: Arc<RwLock<HashMap<IpAddr, IpAggregatedStats>>>,
    dst_ip_stats: Arc<RwLock<HashMap<IpAddr, IpAggregatedStats>>>,
    msg_channel: mpsc::Sender<ConnectMessage>,
    metric_store: DuckMetricStore,
    global_stats: Arc<RwLock<ConnectGlobalStats>>,
}

impl ConnectMetricManager {
    pub fn with_store(metric_store: DuckMetricStore) -> Self {
        let realtime_metrics: Arc<RwLock<HashMap<ConnectKey, ConnectRealtimeStatus>>> =
            Arc::new(RwLock::new(HashMap::new()));
        let src_ip_stats: Arc<RwLock<HashMap<IpAddr, IpAggregatedStats>>> =
            Arc::new(RwLock::new(HashMap::new()));
        let dst_ip_stats: Arc<RwLock<HashMap<IpAddr, IpAggregatedStats>>> =
            Arc::new(RwLock::new(HashMap::new()));
        let (msg_channel, mut message_rx) = mpsc::channel(1024);

        let metric_store_clone = metric_store.clone();
        let realtime_metrics_clone = realtime_metrics.clone();
        let src_ip_stats_clone = src_ip_stats.clone();
        let dst_ip_stats_clone = dst_ip_stats.clone();

        tokio::spawn(async move {
            while let Some(msg) = message_rx.recv().await {
                match msg {
                    ConnectMessage::Metric(metric) => {
                        metric_store_clone.insert_metric(metric.clone()).await;

                        let mut realtime_set = realtime_metrics_clone.write().await;
                        let mut src_agg = src_ip_stats_clone.write().await;
                        let mut dst_agg = dst_ip_stats_clone.write().await;

                        match metric.status {
                            ConnectStatusType::Disabled => {
                                // 连接结束：从汇总表中扣除该连接的所有贡献
                                if let Some(conn) = realtime_set.remove(&metric.key) {
                                    Self::apply_ip_diff(&mut src_agg, conn.src_ip, &conn, false);
                                    Self::apply_ip_diff(&mut dst_agg, conn.dst_ip, &conn, false);
                                }
                            }
                            ConnectStatusType::Active => {
                                let key = metric.key.clone();

                                // 1. 获取（或初始化）连接状态
                                let conn_status =
                                    realtime_set.entry(key.clone()).or_insert_with(|| {
                                        // 如果是新连接，初始化汇总表的计数
                                        src_agg.entry(metric.src_ip).or_default().active_conns += 1;
                                        dst_agg.entry(metric.dst_ip).or_default().active_conns += 1;

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

                                // 2. 读取旧速率
                                let old_ingress_bps = conn_status.ingress_bps;
                                let old_egress_bps = conn_status.egress_bps;
                                let old_ingress_pps = conn_status.ingress_pps;
                                let old_egress_pps = conn_status.egress_pps;

                                // 3. 计算并应用新速率
                                if let Some(old_metric) = &conn_status.last_metric {
                                    let delta_ms =
                                        metric.report_time.saturating_sub(old_metric.report_time);
                                    if delta_ms > 0 {
                                        // 关键优化：限制最大时间间隔，防止长时间空闲后首条数据的速率失真
                                        let effective_delta_ms =
                                            delta_ms.min(MAX_REPORT_INTERVAL_MS);

                                        conn_status.ingress_bps = (metric
                                            .ingress_bytes
                                            .saturating_sub(old_metric.ingress_bytes)
                                            * 8000)
                                            / effective_delta_ms;
                                        conn_status.egress_bps = (metric
                                            .egress_bytes
                                            .saturating_sub(old_metric.egress_bytes)
                                            * 8000)
                                            / effective_delta_ms;
                                        conn_status.ingress_pps = (metric
                                            .ingress_packets
                                            .saturating_sub(old_metric.ingress_packets)
                                            * 1000)
                                            / effective_delta_ms;
                                        conn_status.egress_pps = (metric
                                            .egress_packets
                                            .saturating_sub(old_metric.egress_packets)
                                            * 1000)
                                            / effective_delta_ms;
                                    }
                                }
                                conn_status.last_metric = Some(metric);

                                // 4. 应用差值到源和目的 IP 汇总表
                                // 公式：总带宽 = 总带宽 - 旧速率 + 新速率
                                let s = src_agg.entry(conn_status.src_ip).or_default();
                                s.ingress_bps = s
                                    .ingress_bps
                                    .saturating_sub(old_ingress_bps)
                                    .saturating_add(conn_status.ingress_bps);
                                s.egress_bps = s
                                    .egress_bps
                                    .saturating_sub(old_egress_bps)
                                    .saturating_add(conn_status.egress_bps);
                                s.ingress_pps = s
                                    .ingress_pps
                                    .saturating_sub(old_ingress_pps)
                                    .saturating_add(conn_status.ingress_pps);
                                s.egress_pps = s
                                    .egress_pps
                                    .saturating_sub(old_egress_pps)
                                    .saturating_add(conn_status.egress_pps);

                                let d = dst_agg.entry(conn_status.dst_ip).or_default();
                                d.ingress_bps = d
                                    .ingress_bps
                                    .saturating_sub(old_ingress_bps)
                                    .saturating_add(conn_status.ingress_bps);
                                d.egress_bps = d
                                    .egress_bps
                                    .saturating_sub(old_egress_bps)
                                    .saturating_add(conn_status.egress_bps);
                                d.ingress_pps = d
                                    .ingress_pps
                                    .saturating_sub(old_ingress_pps)
                                    .saturating_add(conn_status.ingress_pps);
                                d.egress_pps = d
                                    .egress_pps
                                    .saturating_sub(old_egress_pps)
                                    .saturating_add(conn_status.egress_pps);
                            }
                            _ => {}
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
            src_ip_stats,
            dst_ip_stats,
            msg_channel,
            metric_store,
            global_stats,
        }
    }

    /// 辅助方法：完整应用或扣除一个连接的速率
    fn apply_ip_diff(
        agg_map: &mut HashMap<IpAddr, IpAggregatedStats>,
        ip: IpAddr,
        conn: &ConnectRealtimeStatus,
        add: bool,
    ) {
        let entry = agg_map.entry(ip).or_default();
        if add {
            entry.ingress_bps = entry.ingress_bps.saturating_add(conn.ingress_bps);
            entry.egress_bps = entry.egress_bps.saturating_add(conn.egress_bps);
            entry.ingress_pps = entry.ingress_pps.saturating_add(conn.ingress_pps);
            entry.egress_pps = entry.egress_pps.saturating_add(conn.egress_pps);
            entry.active_conns += 1;
        } else {
            entry.ingress_bps = entry.ingress_bps.saturating_sub(conn.ingress_bps);
            entry.egress_bps = entry.egress_bps.saturating_sub(conn.egress_bps);
            entry.ingress_pps = entry.ingress_pps.saturating_sub(conn.ingress_pps);
            entry.egress_pps = entry.egress_pps.saturating_sub(conn.egress_pps);
            entry.active_conns = entry.active_conns.saturating_sub(1);
            if entry.active_conns == 0 {
                agg_map.remove(&ip);
            }
        }
    }

    pub async fn new(base_path: PathBuf, config: MetricRuntimeConfig) -> Self {
        let metric_store = DuckMetricStore::new(base_path, config).await;
        Self::with_store(metric_store)
    }

    pub async fn get_global_stats(&self) -> ConnectGlobalStats {
        self.global_stats.read().await.clone()
    }

    pub async fn get_src_ip_stats(&self) -> Vec<IpRealtimeStat> {
        let stats = self.src_ip_stats.read().await;
        stats.iter().map(|(ip, s)| IpRealtimeStat { ip: *ip, stats: s.clone() }).collect()
    }

    pub async fn get_dst_ip_stats(&self) -> Vec<IpRealtimeStat> {
        let stats = self.dst_ip_stats.read().await;
        stats.iter().map(|(ip, s)| IpRealtimeStat { ip: *ip, stats: s.clone() }).collect()
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
        result.sort_by_key(|r| r.key.create_time);
        result
    }

    pub async fn query_metric_by_key(&self, key: ConnectKey) -> Vec<ConnectMetric> {
        let now = landscape_common::utils::time::get_current_time_ms().unwrap_or_default();
        let age_ms = now.saturating_sub(key.create_time);

        let resolution = if age_ms > DAY_RESOLUTION_THRESHOLD_MS {
            MetricResolution::Day
        } else if age_ms > HOUR_RESOLUTION_THRESHOLD_MS {
            MetricResolution::Hour
        } else {
            MetricResolution::Second
        };

        self.metric_store.query_metric_by_key(key, resolution).await
    }

    pub async fn query_metric_by_key_with_resolution(
        &self,
        key: ConnectKey,
        resolution: MetricResolution,
    ) -> Vec<ConnectMetric> {
        self.metric_store.query_metric_by_key(key, resolution).await
    }

    pub async fn history_summaries_complex(
        &self,
        params: ConnectHistoryQueryParams,
    ) -> Vec<ConnectHistoryStatus> {
        self.metric_store.history_summaries_complex(params).await
    }
}
