use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::{net::IpAddr, sync::Arc};

use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, RwLock};
use ts_rs::TS;

use crate::event::ConnectMessage;
#[cfg(feature = "duckdb")]
use crate::metric::duckdb::DuckMetricStore;

///
#[derive(Debug, Serialize, Deserialize, Eq, Hash, PartialEq, Clone, TS)]
#[ts(export, export_to = "common/metric/connect.d.ts")]
pub struct ConnectKey {
    pub src_ip: IpAddr,
    pub dst_ip: IpAddr,
    pub src_port: u16,
    pub dst_port: u16,

    pub l4_proto: u8,
    pub l3_proto: u8,

    pub flow_id: u8,
    pub trace_id: u8,

    #[ts(type = "number")]
    pub create_time: u64,
}

#[derive(Debug, Serialize, Deserialize, TS, Clone)]
#[ts(export, export_to = "common/metric/connect.d.ts")]
pub struct ConnectInfo {
    pub key: ConnectKey,

    pub event_type: ConnectEventType,

    #[ts(type = "number")]
    pub report_time: u64,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, TS)]
#[ts(export, export_to = "common/metric/connect.d.ts")]
#[serde(rename_all = "snake_case")]
pub enum ConnectEventType {
    #[default]
    Unknow,
    CreateConnect,
    DisConnct,
}

impl From<u8> for ConnectEventType {
    fn from(value: u8) -> Self {
        match value {
            1 => ConnectEventType::CreateConnect,
            2 => ConnectEventType::DisConnct,
            _ => ConnectEventType::Unknow,
        }
    }
}

impl Into<u8> for ConnectEventType {
    fn into(self) -> u8 {
        match self {
            ConnectEventType::Unknow => 0,
            ConnectEventType::CreateConnect => 1,
            ConnectEventType::DisConnct => 2,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, Eq, Hash, PartialEq, TS)]
#[ts(export, export_to = "common/metric/connect.d.ts")]
#[serde(rename_all = "snake_case")]
pub enum ConnectStatusType {
    #[default]
    Unknow,
    Active,
    Disabled,
}

impl From<u8> for ConnectStatusType {
    fn from(value: u8) -> Self {
        match value {
            1 => ConnectStatusType::Active,
            2 => ConnectStatusType::Disabled,
            _ => ConnectStatusType::Unknow,
        }
    }
}

impl Into<u8> for ConnectStatusType {
    fn into(self) -> u8 {
        match self {
            ConnectStatusType::Unknow => 0,
            ConnectStatusType::Active => 1,
            ConnectStatusType::Disabled => 2,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, Hash, PartialEq, Clone, TS)]
#[ts(export, export_to = "common/metric/connect.d.ts")]
pub struct ConnectMetric {
    pub key: ConnectKey,

    #[ts(type = "number")]
    pub report_time: u64,

    #[ts(type = "number")]
    pub ingress_bytes: u64,
    #[ts(type = "number")]
    pub ingress_packets: u64,
    #[ts(type = "number")]
    pub egress_bytes: u64,
    #[ts(type = "number")]
    pub egress_packets: u64,

    pub status: ConnectStatusType,
}

#[derive(Debug, Serialize, Deserialize, Eq, Hash, PartialEq, Clone, TS)]
#[ts(export, export_to = "common/metric/connect.d.ts")]
pub struct ConnectAgg {
    #[ts(type = "number")]
    pub ingress_bytes: u64,
    #[ts(type = "number")]
    pub ingress_packets: u64,
    #[ts(type = "number")]
    pub egress_bytes: u64,
    #[ts(type = "number")]
    pub egress_packets: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(export, export_to = "common/metric/connect.d.ts")]
pub struct ConnectRealtimeStatus {
    pub key: ConnectKey,

    #[ts(type = "number")]
    pub ingress_bps: u64,
    #[ts(type = "number")]
    pub egress_bps: u64,
    #[ts(type = "number")]
    pub ingress_pps: u64,
    #[ts(type = "number")]
    pub egress_pps: u64,

    pub last_metric: Option<ConnectMetric>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, TS)]
#[ts(export, export_to = "common/metric/connect.d.ts")]
#[serde(rename_all = "lowercase")]
pub enum ConnectSortKey {
    #[default]
    Time,
    Port,
    Ingress,
    Egress,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, TS)]
#[ts(export, export_to = "common/metric/connect.d.ts")]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    #[default]
    Desc,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, TS)]
#[ts(export, export_to = "common/metric/connect.d.ts")]
pub struct ConnectHistoryQueryParams {
    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
    pub limit: Option<usize>,
    pub src_ip: Option<String>,
    pub dst_ip: Option<String>,
    pub port_start: Option<u16>,
    pub port_end: Option<u16>,
    pub l3_proto: Option<u8>,
    pub l4_proto: Option<u8>,
    pub flow_id: Option<u8>,
    pub sort_key: Option<ConnectSortKey>,
    pub sort_order: Option<SortOrder>,
    pub status: Option<u8>, // 0: Active, 1: Closed
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(export, export_to = "common/metric/connect.d.ts")]
pub struct ConnectHistoryStatus {
    pub key: ConnectKey,
    #[ts(type = "number")]
    pub total_ingress_bytes: u64,
    #[ts(type = "number")]
    pub total_egress_bytes: u64,
    #[ts(type = "number")]
    pub total_ingress_pkts: u64,
    #[ts(type = "number")]
    pub total_egress_pkts: u64,
    #[ts(type = "number")]
    pub last_report_time: u64,

    pub status: u8,
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct ConnectMetricManager {
    /// 存储所有连接事件：创建、销毁
    active_connects: Arc<RwLock<HashSet<ConnectKey>>>,
    /// 实时速率缓存: Key -> (上一次的指标, 当前算出的状态)
    realtime_metrics: Arc<RwLock<HashMap<ConnectKey, ConnectRealtimeStatus>>>,
    msg_channel: mpsc::Sender<ConnectMessage>,
    #[cfg(feature = "duckdb")]
    metric_store: DuckMetricStore,
}

#[allow(unused_variables)]
impl ConnectMetricManager {
    pub async fn new(base_path: PathBuf) -> Self {
        let active_connects = Arc::new(RwLock::new(HashSet::new()));
        let active_connects_clone = active_connects.clone();

        let realtime_metrics = Arc::new(RwLock::new(HashMap::new()));
        let realtime_metrics_clone = realtime_metrics.clone();

        #[cfg(feature = "duckdb")]
        let metric_store = DuckMetricStore::new(base_path).await;

        #[cfg(feature = "duckdb")]
        let metric_store_clone = metric_store.clone();

        #[cfg(feature = "duckdb")]
        tokio::spawn(async move {
            // 定时清理
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(
                crate::DEFAULT_METRIC_CLEANUP_INTERVAL_SECS,
            ));

            loop {
                interval.tick().await;
                let now = chrono::Utc::now().timestamp_millis() as u64;
                let cutoff = now - (crate::DEFAULT_METRIC_RETENTION_DAYS * 24 * 60 * 60 * 1000);
                let cold_metrics = metric_store_clone.collect_and_cleanup_old_metrics(cutoff).await;
                if !cold_metrics.is_empty() {
                    tracing::info!("Collected {} cold metrics", cold_metrics.len());
                }
            }
        });

        let (conn_tx, mut conn_rx) = mpsc::channel::<ConnectInfo>(1024);

        tokio::spawn(async move {
            while let Some(info) = conn_rx.recv().await {
                match info.event_type {
                    ConnectEventType::Unknow => {}
                    ConnectEventType::CreateConnect => {
                        let mut connect_set = active_connects_clone.write().await;
                        connect_set.insert(info.key);
                    }
                    ConnectEventType::DisConnct => {
                        let mut connect_set = active_connects_clone.write().await;
                        connect_set.remove(&info.key);

                        let mut realtime_set = realtime_metrics_clone.write().await;
                        realtime_set.remove(&info.key);
                    }
                }
            }
        });

        #[cfg(feature = "duckdb")]
        let metric_store_clone = metric_store.clone();
        let (msg_channel, mut message_rx) = mpsc::channel(1024);
        let realtime_metrics_clone_for_msg = realtime_metrics.clone();

        tokio::spawn(async move {
            while let Some(msg) = message_rx.recv().await {
                match msg {
                    ConnectMessage::Event(info) => {
                        let _ = conn_tx.send(info.clone()).await;
                        #[cfg(feature = "duckdb")]
                        metric_store_clone.insert_connect_info(info).await;
                    }
                    ConnectMessage::Metric(metric) => {
                        // 计算实时速率
                        {
                            let mut realtime_set = realtime_metrics_clone_for_msg.write().await;
                            let key = metric.key.clone();

                            let status =
                                realtime_set.entry(key.clone()).or_insert(ConnectRealtimeStatus {
                                    key: key.clone(),
                                    ingress_bps: 0,
                                    egress_bps: 0,
                                    ingress_pps: 0,
                                    egress_pps: 0,
                                    last_metric: None,
                                });

                            if let Some(old_metric) = &status.last_metric {
                                let delta_time_ms =
                                    metric.report_time.saturating_sub(old_metric.report_time);
                                if delta_time_ms > 0 {
                                    let delta_ingress_bytes = metric
                                        .ingress_bytes
                                        .saturating_sub(old_metric.ingress_bytes);
                                    let delta_egress_bytes =
                                        metric.egress_bytes.saturating_sub(old_metric.egress_bytes);
                                    let delta_ingress_pkts = metric
                                        .ingress_packets
                                        .saturating_sub(old_metric.ingress_packets);
                                    let delta_egress_pkts = metric
                                        .egress_packets
                                        .saturating_sub(old_metric.egress_packets);

                                    status.ingress_bps =
                                        (delta_ingress_bytes * 8000) / delta_time_ms;
                                    status.egress_bps = (delta_egress_bytes * 8000) / delta_time_ms;
                                    status.ingress_pps =
                                        (delta_ingress_pkts * 1000) / delta_time_ms;
                                    status.egress_pps = (delta_egress_pkts * 1000) / delta_time_ms;
                                }
                            }
                            status.last_metric = Some(metric.clone());
                        }

                        #[cfg(feature = "duckdb")]
                        metric_store_clone.insert_metric(metric).await;
                    }
                }
            }

            tracing::info!("connect metric exit");
        });

        ConnectMetricManager {
            active_connects,
            realtime_metrics,
            msg_channel,
            #[cfg(feature = "duckdb")]
            metric_store,
        }
    }

    pub fn send_connect_msg(&self, msg: ConnectMessage) {
        if let Err(e) = self.msg_channel.try_send(msg) {
            tracing::error!("send firewall metric error: {e:?}");
        }
    }

    pub async fn connect_infos(&self) -> Vec<ConnectRealtimeStatus> {
        let connects = self.active_connects.read().await;
        let realtime_metrics = self.realtime_metrics.read().await;

        let mut result: Vec<ConnectRealtimeStatus> = connects
            .iter()
            .map(|key| {
                realtime_metrics.get(key).cloned().unwrap_or_else(|| ConnectRealtimeStatus {
                    key: key.clone(),
                    ingress_bps: 0,
                    egress_bps: 0,
                    ingress_pps: 0,
                    egress_pps: 0,
                    last_metric: None,
                })
            })
            .collect();

        result.sort_by(|a, b| a.key.create_time.cmp(&b.key.create_time));
        result
    }

    pub async fn query_metric_by_key(&self, key: ConnectKey) -> Vec<ConnectMetric> {
        #[cfg(any(feature = "duckdb", feature = "polars"))]
        {
            self.metric_store.query_metric_by_key(key).await
        }

        #[cfg(not(any(feature = "duckdb", feature = "polars")))]
        {
            let _ = key;
            Vec::new()
        }
    }

    pub async fn history_summaries_complex(
        &self,
        params: ConnectHistoryQueryParams,
    ) -> Vec<ConnectHistoryStatus> {
        #[cfg(feature = "duckdb")]
        {
            self.metric_store.history_summaries_complex(params).await
        }

        #[cfg(not(feature = "duckdb"))]
        {
            let _ = params;
            Vec::new()
        }
    }
}
