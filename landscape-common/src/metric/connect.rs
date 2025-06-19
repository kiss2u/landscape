use std::collections::HashSet;
use std::ops::BitAnd;
use std::ops::Not;
use std::path::PathBuf;
use std::{fs::create_dir_all, time::Duration};
use std::{net::IpAddr, sync::Arc};

use polars::prelude::*;
use polars::{df, error::PolarsResult, frame::DataFrame, prelude::DataType, series::Series};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, RwLock};
use tokio::time::interval;
use ts_rs::TS;

use crate::event::ConnectMessage;

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

#[derive(Debug, Serialize, Deserialize, TS)]
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
    connect_df: Arc<RwLock<DataFrame>>,
    metrics_df: Arc<RwLock<DataFrame>>,
}

impl ConnectMetricManager {
    pub fn new(base_path: PathBuf) -> Self {
        let active_connects = Arc::new(RwLock::new(HashSet::new()));
        let active_connects_clone = active_connects.clone();

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

        let connect_df = init_connect_info_df().expect("init_connect_info_df failed");
        let metrics_df = init_connect_metric_df().expect("init_connect_metric_df failed");
        let connect_df = Arc::new(RwLock::new(connect_df));
        let metrics_df = Arc::new(RwLock::new(metrics_df));

        tokio::spawn(start_metrics_flush_task(connect_df.clone(), base_path.clone(), "connect"));
        tokio::spawn(start_metrics_flush_task(metrics_df.clone(), base_path.clone(), "metrics"));

        let connect_df_clone = connect_df.clone();
        let metrics_df_clone = metrics_df.clone();
        let (msg_channel, mut message_rx) = mpsc::channel(1024);
        tokio::spawn(async move {
            while let Some(msg) = message_rx.recv().await {
                match msg {
                    ConnectMessage::Event(info) => {
                        let key = &info.key;

                        let event_type_val: u8 = info.event_type.clone().into();

                        let new = df![
                            "src_ip" => &[key.src_ip.to_string()],
                            "dst_ip" => &[key.dst_ip.to_string()],
                            "src_port" => &[key.src_port],
                            "dst_port" => &[key.dst_port],
                            "l4_proto" => &[key.l4_proto],
                            "l3_proto" => &[key.l3_proto],
                            "flow_id" => &[key.flow_id],
                            "trace_id" => &[key.trace_id],
                            "create_time" => &[key.create_time],
                            "event_type" => &[event_type_val],
                            "report_time" => &[info.report_time],
                        ]
                        .unwrap();
                        {
                            connect_df_clone.write().await.vstack_mut(&new).unwrap();
                        }
                        // println!("after apend size: {}", connect_df_clone.read().await.size());
                        let _ = conn_tx.send(info).await;
                    }
                    ConnectMessage::Metric(metric) => {
                        let key = &metric.key;

                        let new = df![
                            "src_ip" => &[key.src_ip.to_string()],
                            "dst_ip" => &[key.dst_ip.to_string()],
                            "src_port" => &[key.src_port],
                            "dst_port" => &[key.dst_port],
                            "l4_proto" => &[key.l4_proto],
                            "l3_proto" => &[key.l3_proto],
                            "flow_id" => &[key.flow_id],
                            "trace_id" => &[key.trace_id],
                            "create_time" => &[key.create_time],
                            "report_time" => &[metric.report_time],
                            "ingress_bytes" => &[metric.ingress_bytes],
                            "ingress_packets" => &[metric.ingress_packets],
                            "egress_bytes" => &[metric.egress_bytes],
                            "egress_packets" => &[metric.egress_packets],
                        ]
                        .unwrap();

                        {
                            metrics_df_clone.write().await.vstack_mut(&new).unwrap();
                        }
                        // println!("after apend size: {}", metrics_df_clone.read().await.size());
                    }
                }
            }

            tracing::info!("connect metric exit");
        });

        ConnectMetricManager {
            active_connects,
            msg_channel,
            connect_df,
            metrics_df,
            // active_connects: HashSet::new(),
        }
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

    pub async fn connect_metric_in_a_min(&self, key: ConnectKey) -> Vec<ConnectMetric> {
        let df_snapshot = {
            let df = self.metrics_df.read().await;
            df.clone()
        };

        let mask = df_snapshot
            .column("src_ip")
            .unwrap()
            .str()
            .unwrap()
            .equal(key.src_ip.to_string().as_str())
            .bitand(
                df_snapshot
                    .column("dst_ip")
                    .unwrap()
                    .str()
                    .unwrap()
                    .equal(key.dst_ip.to_string().as_str()),
            )
            .bitand(df_snapshot.column("src_port").unwrap().u16().unwrap().equal(key.src_port))
            .bitand(df_snapshot.column("dst_port").unwrap().u16().unwrap().equal(key.dst_port))
            .bitand(df_snapshot.column("l4_proto").unwrap().u8().unwrap().equal(key.l4_proto))
            .bitand(df_snapshot.column("l3_proto").unwrap().u8().unwrap().equal(key.l3_proto))
            .bitand(df_snapshot.column("flow_id").unwrap().u8().unwrap().equal(key.flow_id))
            .bitand(df_snapshot.column("trace_id").unwrap().u8().unwrap().equal(key.trace_id))
            .bitand(
                df_snapshot.column("create_time").unwrap().u64().unwrap().equal(key.create_time),
            );

        let filtered = df_snapshot.filter(&mask).unwrap();

        let mut result = Vec::new();
        for row in 0..filtered.height() {
            result.push(ConnectMetric {
                key: key.clone(),
                report_time: filtered
                    .column("report_time")
                    .unwrap()
                    .u64()
                    .unwrap()
                    .get(row)
                    .unwrap(),
                ingress_bytes: filtered
                    .column("ingress_bytes")
                    .unwrap()
                    .u64()
                    .unwrap()
                    .get(row)
                    .unwrap(),
                ingress_packets: filtered
                    .column("ingress_packets")
                    .unwrap()
                    .u64()
                    .unwrap()
                    .get(row)
                    .unwrap(),
                egress_bytes: filtered
                    .column("egress_bytes")
                    .unwrap()
                    .u64()
                    .unwrap()
                    .get(row)
                    .unwrap(),
                egress_packets: filtered
                    .column("egress_packets")
                    .unwrap()
                    .u64()
                    .unwrap()
                    .get(row)
                    .unwrap(),
            });
        }

        result
    }
}

pub async fn start_metrics_flush_task(
    df: Arc<RwLock<DataFrame>>,
    base_path: PathBuf,
    folder_name: &str,
) {
    let base_path = base_path.join(folder_name);
    create_dir_all(&base_path).unwrap();

    let mut ticker = interval(Duration::from_secs(60));

    loop {
        ticker.tick().await;

        let now = crate::utils::time::get_current_time_ms().unwrap();
        let cutoff = now.saturating_sub(20 * 60 * 1_000); //one hour

        let mut metrics_lock = df.write().await;
        let df = &mut *metrics_lock;

        if df.height() == 0 {
            continue;
        }

        // 找到冷数据（report_time < cutoff）
        let report_time_col = df.column("report_time").unwrap();
        let mask = report_time_col.u64().unwrap().lt(cutoff);

        let cold_df = df.filter(&mask).unwrap();
        let hot_df = df.filter(&mask.not()).unwrap();
        if cold_df.height() == 0 {
            continue;
        }

        // 暂时先不进行存储
        drop(cold_df);

        // 写入磁盘
        // let file_name = format!("{}/{}.parquet", base_path.display(), now);
        // let mut file = std::fs::File::create(&file_name).unwrap();
        // ParquetWriter::new(&mut file).finish(&mut cold_df).unwrap();

        // 替换为热数据
        *df = hot_df;
        // println!("Wrote cold metrics to: {file_name}");
    }
}

pub fn add_connect_metric(metrics_df: &mut DataFrame, metric: &ConnectMetric) {
    let key = &metric.key;

    let new = df![
        "src_ip" => &[key.src_ip.to_string()],
        "dst_ip" => &[key.dst_ip.to_string()],
        "src_port" => &[key.src_port],
        "dst_port" => &[key.dst_port],
        "l4_proto" => &[key.l4_proto ],
        "l3_proto" => &[key.l3_proto],
        "flow_id" => &[key.flow_id],
        "trace_id" => &[key.trace_id ],
        "create_time" => &[key.create_time],
        "report_time" => &[metric.report_time],
        "ingress_bytes" => &[metric.ingress_bytes],
        "ingress_packets" => &[metric.ingress_packets],
        "egress_bytes" => &[metric.egress_bytes],
        "egress_packets" => &[metric.egress_packets],
    ]
    .unwrap();

    metrics_df.vstack_mut(&new).unwrap();
    // println!("after apend size: {}", metrics_df.size());
}

fn init_connect_info_df() -> PolarsResult<DataFrame> {
    df![
        "src_ip" => Series::new_empty("src_ip".into(), &DataType::String),
        "dst_ip" => Series::new_empty("dst_ip".into(), &DataType::String),
        "src_port" => Series::new_empty("src_port".into(), &DataType::UInt16),
        "dst_port" => Series::new_empty("dst_port".into(), &DataType::UInt16),
        "l4_proto" => Series::new_empty("l4_proto".into(), &DataType::UInt8),
        "l3_proto" => Series::new_empty("l3_proto".into(), &DataType::UInt8),
        "flow_id" => Series::new_empty("flow_id".into(), &DataType::UInt8),
        "trace_id" => Series::new_empty("trace_id".into(), &DataType::UInt8),
        "create_time" => Series::new_empty("create_time".into(), &DataType::UInt64),
        "event_type" => Series::new_empty("event_type".into(), &DataType::UInt8),
        "report_time" => Series::new_empty("report_time".into(), &DataType::UInt64),
    ]
}

fn init_connect_metric_df() -> PolarsResult<DataFrame> {
    df![
        "src_ip" => Series::new_empty("src_ip".into(), &DataType::String),
        "dst_ip" => Series::new_empty("dst_ip".into(), &DataType::String),
        "src_port" => Series::new_empty("src_port".into(), &DataType::UInt16),
        "dst_port" => Series::new_empty("dst_port".into(), &DataType::UInt16),
        "l4_proto" => Series::new_empty("l4_proto".into(), &DataType::UInt8),
        "l3_proto" => Series::new_empty("l3_proto".into(), &DataType::UInt8),
        "flow_id" => Series::new_empty("flow_id".into(), &DataType::UInt8),
        "trace_id" => Series::new_empty("trace_id".into(), &DataType::UInt8),
        "create_time" => Series::new_empty("create_time".into(), &DataType::UInt64),
        "report_time" => Series::new_empty("report_time".into(), &DataType::UInt64),
        "ingress_bytes" => Series::new_empty("ingress_bytes".into(), &DataType::UInt64),
        "ingress_packets" => Series::new_empty("ingress_packets".into(), &DataType::UInt64),
        "egress_bytes" => Series::new_empty("egress_bytes".into(), &DataType::UInt64),
        "egress_packets" => Series::new_empty("egress_packets".into(), &DataType::UInt64),
    ]
}
