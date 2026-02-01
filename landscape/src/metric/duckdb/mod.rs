use landscape_common::metric::connect::{
    ConnectGlobalStats, ConnectHistoryQueryParams, ConnectHistoryStatus, ConnectInfo, ConnectKey,
    ConnectMetric,
};
use landscape_common::metric::dns::{DnsHistoryQueryParams, DnsMetric};
use duckdb::{params, Connection};
use std::path::PathBuf;
use std::thread;
use tokio::sync::{mpsc, oneshot};

pub mod connect;
pub mod dns;

use landscape_common::config::MetricRuntimeConfig;

/// Database operation messages
pub enum DBMessage {
    InsertConnectInfo(ConnectInfo),
    InsertMetric(ConnectMetric),

    CollectAndCleanupOldMetrics {
        cutoff: u64,
        resp: oneshot::Sender<Box<Vec<ConnectMetric>>>,
    },
    // DNS Metrics
    InsertDnsMetric(DnsMetric),
}

#[derive(Clone)]
pub struct DuckMetricStore {
    tx: mpsc::Sender<DBMessage>,
    pub db_path: PathBuf,
    pub config: MetricRuntimeConfig,
}

pub fn start_db_thread(
    mut rx: mpsc::Receiver<DBMessage>,
    base_path: PathBuf,
    metric_config: MetricRuntimeConfig,
) {
    if !base_path.exists() {
        std::fs::create_dir_all(&base_path).expect("Failed to create base directory");
    }

    let db_path = base_path.join(format!(
        "metrics_v{}.duckdb",
        landscape_common::LANDSCAPE_METRIC_DB_VERSION
    ));

    let config = duckdb::Config::default()
        .threads(metric_config.max_threads as i64)
        .unwrap()
        .max_memory(&format!("{}MB", metric_config.max_memory))
        .unwrap();

    let conn = Connection::open_with_flags(&db_path, config).unwrap();

    connect::create_summaries_table(&conn);
    connect::create_metrics_table(&conn).unwrap();
    dns::create_dns_table(&conn).unwrap();

    // Schema migration: ensure columns exist
    let _ =
        conn.execute("ALTER TABLE connect_summaries ADD COLUMN IF NOT EXISTS status INTEGER", []);
    let _ = conn.execute("ALTER TABLE metrics ADD COLUMN IF NOT EXISTS status INTEGER", []);

    let mut metrics_appender = conn.appender("metrics").unwrap();
    let mut dns_appender = conn.appender("dns_metrics").unwrap();

    let mut summary_stmt = conn.prepare("
        INSERT OR REPLACE INTO connect_summaries (
            src_ip, dst_ip, src_port, dst_port, l4_proto, l3_proto, flow_id, trace_id, create_time,
            last_report_time, total_ingress_bytes, total_egress_bytes, total_ingress_pkts, total_egress_pkts, status
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
    ").unwrap();

    let mut batch_count = 0;
    let flush_interval = std::time::Duration::from_secs(landscape_common::DEFAULT_METRIC_FLUSH_INTERVAL_SECS);
    let mut last_flush = std::time::Instant::now();

    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();

    rt.block_on(async {
        loop {
            let now = std::time::Instant::now();
            let remaining = flush_interval.saturating_sub(now.duration_since(last_flush));

            let timeout_res = tokio::time::timeout(remaining, rx.recv()).await;

            match timeout_res {
                Ok(Some(msg)) => {
                    match msg {
                        DBMessage::InsertConnectInfo(info) => {
                            let key = &info.key;
                            let event_type_val: u8 = info.event_type.into();

                            let _ = summary_stmt.execute(params![
                                key.src_ip.to_string(),
                                key.dst_ip.to_string(),
                                key.src_port as i64,
                                key.dst_port as i64,
                                key.l4_proto as i64,
                                key.l3_proto as i64,
                                key.flow_id as i64,
                                key.trace_id as i64,
                                key.create_time as i64,
                                info.report_time as i64,
                                0_i64,
                                0_i64,
                                0_i64,
                                0_i64,
                                event_type_val as i64,
                            ]);
                            batch_count += 1;
                        }
                        DBMessage::InsertMetric(metric) => {
                            let key = &metric.key;
                            let _ = metrics_appender.append_row(params![
                                key.src_ip.to_string(),
                                key.dst_ip.to_string(),
                                key.src_port as i64,
                                key.dst_port as i64,
                                key.l4_proto as i64,
                                key.l3_proto as i64,
                                key.flow_id as i64,
                                key.trace_id as i64,
                                key.create_time as i64,
                                metric.report_time as i64,
                                metric.ingress_bytes as i64,
                                metric.ingress_packets as i64,
                                metric.egress_bytes as i64,
                                metric.egress_packets as i64,
                                {
                                    let v: u8 = metric.status.clone().into();
                                    v as i64
                                },
                            ]);

                            let _ = summary_stmt.execute(params![
                                key.src_ip.to_string(),
                                key.dst_ip.to_string(),
                                key.src_port as i64,
                                key.dst_port as i64,
                                key.l4_proto as i64,
                                key.l3_proto as i64,
                                key.flow_id as i64,
                                key.trace_id as i64,
                                key.create_time as i64,
                                metric.report_time as i64,
                                metric.ingress_bytes as i64,
                                metric.egress_bytes as i64,
                                metric.ingress_packets as i64,
                                metric.egress_packets as i64,
                                {
                                    let v: u8 = metric.status.clone().into();
                                    v as i64
                                },
                            ]);

                            batch_count += 1;
                        }
                        DBMessage::CollectAndCleanupOldMetrics { cutoff, resp } => {
                            let _ = metrics_appender.flush();
                            let _ = dns_appender.flush();
                            batch_count = 0;
                            last_flush = std::time::Instant::now();
                            let result = connect::collect_and_cleanup_old_metrics(&conn, cutoff);
                            dns::cleanup_old_dns_metrics(&conn, cutoff);
                            let _ = resp.send(result);
                        }
                        DBMessage::InsertDnsMetric(metric) => {
                            let _ = dns_appender.append_row(params![
                                metric.flow_id as i64,
                                metric.domain,
                                metric.query_type,
                                metric.response_code,
                                metric.report_time as i64,
                                metric.duration_ms as i64,
                                metric.src_ip.to_string(),
                                serde_json::to_string(&metric.answers).unwrap_or_default(),
                            ]);
                            batch_count += 1;
                        }
                    }

                    if batch_count >= landscape_common::DEFAULT_METRIC_BATCH_SIZE {
                        let _ = metrics_appender.flush();
                        let _ = dns_appender.flush();
                        batch_count = 0;
                        last_flush = std::time::Instant::now();
                    }
                }
                Ok(None) => break,
                Err(_) => {
                    if batch_count > 0 {
                        let _ = metrics_appender.flush();
                        let _ = dns_appender.flush();
                        batch_count = 0;
                    }
                    last_flush = std::time::Instant::now();
                }
            }
        }
    });
}

impl DuckMetricStore {
    pub async fn new(base_path: PathBuf, config: MetricRuntimeConfig) -> Self {
        let db_path = base_path.join(format!(
            "metrics_v{}.duckdb",
            landscape_common::LANDSCAPE_METRIC_DB_VERSION
        ));
        let (tx, rx) = mpsc::channel::<DBMessage>(1024);
        let base_path_clone = base_path.clone();
        let config_clone = config.clone();
        thread::spawn(move || {
            start_db_thread(rx, base_path_clone, config_clone);
        });

        DuckMetricStore { tx, db_path, config }
    }

    pub fn get_readonly_conn(&self) -> Connection {
        // 以只读模式打开连接 (DuckDB 同时支持多读)
        let config = duckdb::Config::default()
            .access_mode(duckdb::AccessMode::ReadOnly)
            .unwrap()
            .threads(self.config.max_threads as i64)
            .unwrap()
            .max_memory(&format!("{}MB", self.config.max_memory))
            .unwrap();
        Connection::open_with_flags(&self.db_path, config).unwrap()
    }

    pub async fn insert_connect_info(&self, info: ConnectInfo) {
        let _ = self.tx.send(DBMessage::InsertConnectInfo(info)).await;
    }

    pub async fn insert_metric(&self, metric: ConnectMetric) {
        let _ = self.tx.send(DBMessage::InsertMetric(metric)).await;
    }

    pub async fn query_metric_by_key(&self, key: ConnectKey) -> Vec<ConnectMetric> {
        let store = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = store.get_readonly_conn();
            connect::query_metric_by_key(&conn, &key)
        })
        .await
        .unwrap_or_default()
    }

    pub async fn current_active_connect_keys(&self) -> Vec<ConnectKey> {
        let store = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = store.get_readonly_conn();
            connect::current_active_connect_keys(&conn)
        })
        .await
        .unwrap_or_default()
    }

    pub async fn collect_and_cleanup_old_metrics(&self, cutoff: u64) -> Box<Vec<ConnectMetric>> {
        let (resp, rx) = oneshot::channel();
        let _ = self.tx.send(DBMessage::CollectAndCleanupOldMetrics { cutoff, resp }).await;
        rx.await.unwrap()
    }

    pub async fn history_summaries_complex(
        &self,
        params: ConnectHistoryQueryParams,
    ) -> Vec<ConnectHistoryStatus> {
        let store = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = store.get_readonly_conn();
            connect::query_historical_summaries_complex(&conn, params)
        })
        .await
        .unwrap_or_default()
    }

    pub async fn get_global_stats(&self) -> ConnectGlobalStats {
        let store = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = store.get_readonly_conn();
            connect::query_global_stats(&conn)
        })
        .await
        .unwrap_or_default()
    }

    pub async fn insert_dns_metric(&self, metric: DnsMetric) {
        let _ = self.tx.send(DBMessage::InsertDnsMetric(metric)).await;
    }

    pub async fn query_dns_history(&self, params: DnsHistoryQueryParams) -> Vec<DnsMetric> {
        let store = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = store.get_readonly_conn();
            dns::query_dns_history(&conn, params)
        })
        .await
        .unwrap_or_default()
    }
}
