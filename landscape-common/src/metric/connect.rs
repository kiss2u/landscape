use std::collections::HashSet;
use std::path::PathBuf;
use std::{net::IpAddr, sync::Arc};

use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, RwLock};
use ts_rs::TS;

use crate::event::ConnectMessage;
use crate::metric::duckdb::DuckMetricStore;

#[cfg(debug_assertions)]
const CLEAR_INTERVAL: u64 = 60;
#[cfg(not(debug_assertions))]
const CLEAR_INTERVAL: u64 = 60 * 10;

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

#[derive(Clone)]
#[allow(dead_code)]
pub struct ConnectMetricManager {
    /// 存储所有连接事件：创建、销毁
    active_connects: Arc<RwLock<HashSet<ConnectKey>>>,
    msg_channel: mpsc::Sender<ConnectMessage>,
    metric_store: DuckMetricStore,
}

impl ConnectMetricManager {
    pub async fn new(base_path: PathBuf) -> Self {
        let active_connects = Arc::new(RwLock::new(HashSet::new()));
        let active_connects_clone = active_connects.clone();

        let metric_store = DuckMetricStore::new(base_path).await;

        let metric_store_clone = metric_store.clone();
        tokio::spawn(async move {
            // 定时清理
            let mut interval =
                tokio::time::interval(tokio::time::Duration::from_secs(CLEAR_INTERVAL));
            loop {
                interval.tick().await;
                let now = chrono::Utc::now().timestamp_millis() as u64;
                let cutoff = now - CLEAR_INTERVAL * 1000;
                let cold_metrics = metric_store_clone.collect_and_cleanup_old_metrics(cutoff).await;
                let cold_infos = metric_store_clone.collect_and_cleanup_old_infos(cutoff).await;
                if !cold_metrics.is_empty() {
                    tracing::info!("Collected {} cold metrics", cold_metrics.len());
                }

                if !cold_infos.is_empty() {
                    tracing::info!("Collected {} cold infos", cold_infos.len());
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
                    }
                }
            }
        });

        let metric_store_clone = metric_store.clone();
        let (msg_channel, mut message_rx) = mpsc::channel(1024);
        tokio::spawn(async move {
            while let Some(msg) = message_rx.recv().await {
                match msg {
                    ConnectMessage::Event(info) => {
                        let _ = conn_tx.send(info.clone()).await;
                        metric_store_clone.insert_connect_info(info).await;
                    }
                    ConnectMessage::Metric(metric) => {
                        metric_store_clone.insert_metric(metric).await;
                    }
                }
            }

            tracing::info!("connect metric exit");
        });

        ConnectMetricManager { active_connects, msg_channel, metric_store }
    }

    pub fn send_connect_msg(&self, msg: ConnectMessage) {
        if let Err(e) = self.msg_channel.try_send(msg) {
            tracing::error!("send firewall metric error: {e:?}");
        }
    }

    pub async fn connect_infos(&self) -> Vec<ConnectKey> {
        let connects = self.active_connects.read().await;
        let mut result: Vec<ConnectKey> = connects.iter().map(Clone::clone).collect();
        result.sort_by(|a, b| a.create_time.cmp(&b.create_time));
        result
    }

    pub async fn query_metric_by_key(&self, key: ConnectKey) -> Vec<ConnectMetric> {
        self.metric_store.query_metric_by_key(key).await
    }
}
