use duckdb::{params, Connection};
use landscape_common::metric::connect::{
    ConnectGlobalStats, ConnectHistoryQueryParams, ConnectHistoryStatus, ConnectKey,
    ConnectMetric,
};
use landscape_common::metric::dns::{DnsHistoryQueryParams, DnsHistoryResponse, DnsMetric, DnsSummaryResponse};
use std::net::IpAddr;
use std::path::PathBuf;
use std::thread;
use tokio::sync::{mpsc, oneshot};

fn clean_ip_string(ip: &IpAddr) -> String {
    match ip {
        IpAddr::V6(v6) => {
            if let Some(v4) = v6.to_ipv4_mapped() {
                v4.to_string()
            } else {
                v6.to_string()
            }
        }
        IpAddr::V4(v4) => v4.to_string(),
    }
}

pub mod connect;
pub mod dns;

use landscape_common::config::MetricRuntimeConfig;

/// Database operation messages
pub enum DBMessage {
    // Write Operations
    InsertMetric(ConnectMetric),
    InsertDnsMetric(DnsMetric),
    
    // Command Operations (Maintenance/Cleanup)
    CollectAndCleanupOldMetrics {
        cutoff: u64,
        resp: oneshot::Sender<Box<Vec<ConnectMetric>>>,
    },

    // Sub-Query Enums
    DnsQuery {
        query: dns::DnsQuery,
        resp: oneshot::Sender<dns::DnsQueryResult>,
    },
    ConnectQuery {
        query: connect::ConnectQuery,
        resp: oneshot::Sender<connect::ConnectQueryResult>,
    },
}

#[derive(Clone)]
pub struct DuckMetricStore {
    tx: mpsc::Sender<DBMessage>,
    pub db_path: PathBuf,
    pub config: MetricRuntimeConfig,
}

pub fn start_db_thread(
    mut rx: mpsc::Receiver<DBMessage>,
    db_path: PathBuf,
    metric_config: MetricRuntimeConfig,
) {
    let config = duckdb::Config::default()
        .threads(metric_config.max_threads as i64)
        .unwrap()
        .max_memory(&format!("{}MB", metric_config.max_memory))
        .unwrap();

    let conn = Connection::open_with_flags(&db_path, config).unwrap();

    // 初始化表
    connect::create_summaries_table(&conn);
    connect::create_metrics_table(&conn).unwrap();
    dns::create_dns_table(&conn).unwrap();

    // 状态管理
    let mut metrics_appender = conn.appender("metrics").unwrap();
    let mut dns_appender = conn.appender("dns_metrics").unwrap();
    let mut summary_stmt = conn.prepare(connect::SUMMARY_INSERT_SQL).unwrap();
    
    let mut batch_count = 0;
    let flush_interval = std::time::Duration::from_secs(landscape_common::DEFAULT_METRIC_FLUSH_INTERVAL_SECS);
    let mut last_flush = std::time::Instant::now();
    let mut last_cleanup = std::time::Instant::now();
    let cleanup_interval = std::time::Duration::from_secs(landscape_common::DEFAULT_METRIC_CLEANUP_INTERVAL_SECS);

    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();

    rt.block_on(async {
        loop {
            let now = std::time::Instant::now();
            let remaining = flush_interval.saturating_sub(now.duration_since(last_flush));

            let timeout_res = tokio::time::timeout(remaining, rx.recv()).await;

            match timeout_res {
                Ok(Some(msg)) => {
                    match msg {
                        // --- 写入类操作 ---
                        DBMessage::InsertMetric(metric) => {
                            let key = &metric.key;
                            let _ = metrics_appender.append_row(params![
                                key.create_time as i64,
                                key.cpu_id as i64,
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

                            if connect::update_summary_by_metric(&mut summary_stmt, &metric).is_ok() {
                                batch_count += 1;
                            }
                        }
                        DBMessage::InsertDnsMetric(metric) => {
                            let _ = dns_appender.append_row(params![
                                metric.flow_id as i64,
                                metric.domain,
                                metric.query_type,
                                metric.response_code,
                                metric.report_time as i64,
                                metric.duration_ms as i64,
                                clean_ip_string(&metric.src_ip),
                                serde_json::to_string(&metric.answers).unwrap_or_default(),
                                serde_json::to_string(&metric.status).unwrap_or_default(),
                            ]);
                            batch_count += 1;
                        }

                        // --- 抽象后的查询操作 ---
                        DBMessage::DnsQuery { query, resp } => {
                            let result = dns::handle_query(&conn, query);
                            let _ = resp.send(result);
                        }
                        DBMessage::ConnectQuery { query, resp } => {
                            let result = connect::handle_query(&conn, query);
                            let _ = resp.send(result);
                        }

                        // --- 管理类操作 ---
                        DBMessage::CollectAndCleanupOldMetrics { cutoff, resp } => {
                            let _ = metrics_appender.flush();
                            let _ = dns_appender.flush();
                            batch_count = 0;
                            last_flush = std::time::Instant::now();
                            let result = connect::collect_and_cleanup_old_metrics(&conn, cutoff);
                            dns::cleanup_old_dns_metrics(&conn, cutoff);
                            let _ = resp.send(result);
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

            if last_cleanup.elapsed() > cleanup_interval {
                let retention_days = metric_config.retention_days.max(1);
                let cutoff = landscape_common::utils::time::get_current_time_ms()
                    .unwrap_or_default()
                    .saturating_sub(retention_days as u64 * 24 * 3600 * 1000);

                let _ = metrics_appender.flush();
                let _ = dns_appender.flush();
                batch_count = 0;
                last_flush = std::time::Instant::now();
                last_cleanup = std::time::Instant::now();

                let _ = connect::collect_and_cleanup_old_metrics(&conn, cutoff);
                dns::cleanup_old_dns_metrics(&conn, cutoff);
                tracing::info!("Auto cleanup old metrics, cutoff: {}", cutoff);
            }
        }
    });
}

impl DuckMetricStore {
    pub async fn new(base_path: PathBuf, config: MetricRuntimeConfig) -> Self {
        let db_path = base_path
            .join(format!("metrics_v{}.duckdb", landscape_common::LANDSCAPE_METRIC_DB_VERSION));

        if let Some(parent) = db_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).expect("Failed to create base directory");
            }
        }

        let (tx, rx) = mpsc::channel::<DBMessage>(1024);
        let db_path_clone = db_path.clone();
        let config_clone = config.clone();
        thread::spawn(move || {
            start_db_thread(rx, db_path_clone, config_clone);
        });

        DuckMetricStore { tx, db_path, config }
    }

    pub async fn insert_metric(&self, metric: ConnectMetric) {
        let _ = self.tx.send(DBMessage::InsertMetric(metric)).await;
    }

    pub async fn query_metric_by_key(&self, key: ConnectKey) -> Vec<ConnectMetric> {
        let (resp, rx) = oneshot::channel();
        let q = connect::ConnectQuery::ByKey(key);
        if self.tx.send(DBMessage::ConnectQuery { query: q, resp }).await.is_err() {
            return Vec::new();
        }
        match rx.await {
            Ok(connect::ConnectQueryResult::Metrics(m)) => m,
            _ => Vec::new(),
        }
    }

    pub async fn current_active_connect_keys(&self) -> Vec<ConnectKey> {
        let (resp, rx) = oneshot::channel();
        let q = connect::ConnectQuery::ActiveKeys;
        if self.tx.send(DBMessage::ConnectQuery { query: q, resp }).await.is_err() {
            return Vec::new();
        }
        match rx.await {
            Ok(connect::ConnectQueryResult::Keys(k)) => k,
            _ => Vec::new(),
        }
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
        let (resp, rx) = oneshot::channel();
        let q = connect::ConnectQuery::HistorySummaries(params);
        if self.tx.send(DBMessage::ConnectQuery { query: q, resp }).await.is_err() {
            return Vec::new();
        }
        match rx.await {
            Ok(connect::ConnectQueryResult::HistoryStatuses(h)) => h,
            _ => Vec::new(),
        }
    }

    pub async fn get_global_stats(&self) -> ConnectGlobalStats {
        let (resp, rx) = oneshot::channel();
        let q = connect::ConnectQuery::GlobalStats;
        if self.tx.send(DBMessage::ConnectQuery { query: q, resp }).await.is_err() {
            return ConnectGlobalStats::default();
        }
        match rx.await {
            Ok(connect::ConnectQueryResult::Stats(s)) => s,
            _ => ConnectGlobalStats::default(),
        }
    }

    pub async fn insert_dns_metric(&self, mut metric: DnsMetric) {
        if metric.domain.ends_with('.') && metric.domain.len() > 1 {
            metric.domain.pop();
        }
        let _ = self.tx.send(DBMessage::InsertDnsMetric(metric)).await;
    }

    pub async fn query_dns_history(&self, params: DnsHistoryQueryParams) -> DnsHistoryResponse {
        let (resp, rx) = oneshot::channel();
        let q = dns::DnsQuery::History(params);
        if self.tx.send(DBMessage::DnsQuery { query: q, resp }).await.is_err() {
            return DnsHistoryResponse { items: Vec::new(), total: 0 };
        }
        match rx.await {
            Ok(dns::DnsQueryResult::History(result)) => result,
            _ => DnsHistoryResponse { items: Vec::new(), total: 0 },
        }
    }

    pub async fn get_dns_summary(&self, params: DnsHistoryQueryParams) -> DnsSummaryResponse {
        let (resp, rx) = oneshot::channel();
        let q = dns::DnsQuery::Summary(params);
        if self.tx.send(DBMessage::DnsQuery { query: q, resp }).await.is_err() {
             return DnsSummaryResponse {
                total_queries: 0,
                total_effective_queries: 0,
                cache_hit_count: 0,
                hit_count_v4: 0,
                hit_count_v6: 0,
                hit_count_other: 0,
                total_v4: 0,
                total_v6: 0,
                total_other: 0,
                block_count: 0,
                nxdomain_count: 0,
                error_count: 0,
                avg_duration_ms: 0.0,
                p50_duration_ms: 0.0,
                p95_duration_ms: 0.0,
                p99_duration_ms: 0.0,
                max_duration_ms: 0.0,
                top_clients: vec![],
                top_domains: vec![],
                top_blocked: vec![],
                slowest_domains: vec![],
            };
        }
        match rx.await {
            Ok(dns::DnsQueryResult::Summary(result)) => result,
            _ => DnsSummaryResponse {
                total_queries: 0,
                total_effective_queries: 0,
                cache_hit_count: 0,
                hit_count_v4: 0,
                hit_count_v6: 0,
                hit_count_other: 0,
                total_v4: 0,
                total_v6: 0,
                total_other: 0,
                block_count: 0,
                nxdomain_count: 0,
                error_count: 0,
                avg_duration_ms: 0.0,
                p50_duration_ms: 0.0,
                p95_duration_ms: 0.0,
                p99_duration_ms: 0.0,
                max_duration_ms: 0.0,
                top_clients: vec![],
                top_domains: vec![],
                top_blocked: vec![],
                slowest_domains: vec![],
            },
        }
    }
}
