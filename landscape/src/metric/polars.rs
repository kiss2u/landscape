use std::ops::BitAnd;
use std::ops::Not;
use std::path::PathBuf;
use std::sync::Arc;
use std::{fs::create_dir_all, time::Duration};

use polars::prelude::*;
use polars::{df, error::PolarsResult, frame::DataFrame, prelude::DataType, series::Series};
use tokio::sync::RwLock;
use tokio::time::interval;

use landscape_common::metric::connect::ConnectInfo;
use landscape_common::metric::connect::ConnectKey;
use landscape_common::metric::connect::ConnectMetric;

#[derive(Clone)]
pub struct PolarsMetricStore {
    connect_df: Arc<RwLock<DataFrame>>,
    metrics_df: Arc<RwLock<DataFrame>>,
}

impl PolarsMetricStore {
    pub async fn new(base_path: PathBuf) -> Self {
        let connect_df = init_connect_info_df().expect("init_connect_info_df failed");
        let metrics_df = init_connect_metric_df().expect("init_connect_metric_df failed");
        let connect_df = Arc::new(RwLock::new(connect_df));
        let metrics_df = Arc::new(RwLock::new(metrics_df));

        tokio::spawn(start_metrics_flush_task(connect_df.clone(), base_path.clone(), "connect"));
        tokio::spawn(start_metrics_flush_task(metrics_df.clone(), base_path.clone(), "metrics"));

        Self { connect_df, metrics_df }
    }

    pub async fn insert_connect_info(&self, info: &ConnectInfo) {
        let mut connect_df_clone = self.connect_df.write().await;
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
            connect_df_clone.vstack_mut(&new).unwrap();
        }
        // println!("after apend size: {}", connect_df_clone.read().await.size());
    }

    pub async fn insert_connect_metric(&self, metric: &ConnectMetric) {
        let mut metrics_df_clone = self.metrics_df.write().await;

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
            metrics_df_clone.vstack_mut(&new).unwrap();
        }
        // println!("after apend size: {}", metrics_df_clone.read().await.size());
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
                status: landscape_common::metric::connect::ConnectStatusType::Active, // Fallback status
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

        let now = chrono::Utc::now().timestamp_millis() as u64;
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

        // 写入磁盘
        // let file_name = format!("{}/{}.parquet", base_path.display(), now);
        // let mut file = std::fs::File::create(&file_name).unwrap();
        // ParquetWriter::new(&mut file).finish(&mut cold_df).unwrap();

        // 替换为热数据
        *df = hot_df;

        drop(metrics_lock);
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
