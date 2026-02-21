use dashmap::DashMap;
use duckdb::{params, Appender, DuckdbConnectionManager};

use landscape_common::metric::connect::{
    ConnectGlobalStats, ConnectHistoryQueryParams, ConnectHistoryStatus, ConnectKey, ConnectMetric,
    ConnectMetricPoint, ConnectRealtimeStatus, ConnectStatusType, MetricResolution,
};
use landscape_common::metric::dns::{
    DnsHistoryQueryParams, DnsHistoryResponse, DnsLightweightSummaryResponse, DnsMetric,
    DnsSummaryQueryParams, DnsSummaryResponse,
};
use r2d2::{self, PooledConnection};
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::Arc;
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

const A_MIN: u64 = 60 * 1000;
const MS_PER_MINUTE: u64 = A_MIN;
const MS_PER_DAY: u64 = 24 * 60 * A_MIN;
const STALE_TIMEOUT_MS: u64 = 5 * A_MIN;

/// Database operation messages
pub enum DBMessage {
    // Write Operations
    InsertMetric(ConnectMetric),
    InsertDnsMetric(DnsMetric),

    // Command Operations (Maintenance/Cleanup)
    CollectAndCleanupOldMetrics {
        cutoff_raw: u64,
        cutoff_1m: u64,
        cutoff_1h: u64,
        cutoff_1d: u64,
        cutoff_dns: u64,
        resp: oneshot::Sender<Box<Vec<ConnectMetric>>>,
    },
}

#[derive(Clone)]
pub struct RealtimeState {
    pub status: ConnectRealtimeStatus,
    pub last_ingress_bytes: u64,
    pub last_egress_bytes: u64,
    pub last_ingress_pkts: u64,
    pub last_egress_pkts: u64,
}

#[derive(Clone)]
pub struct DuckMetricStore {
    tx: mpsc::Sender<DBMessage>,
    pub db_path: PathBuf,
    pub config: MetricRuntimeConfig,
    pub disk_pool: r2d2::Pool<DuckdbConnectionManager>,
    pub realtime_cache: Arc<DashMap<ConnectKey, RealtimeState>>,
}

pub fn start_db_thread(
    mut rx: mpsc::Receiver<DBMessage>,
    metric_config: MetricRuntimeConfig,
    disk_pool: r2d2::Pool<DuckdbConnectionManager>,
    conn_dns: PooledConnection<DuckdbConnectionManager>,
    conn_disk_writer: PooledConnection<DuckdbConnectionManager>,
    realtime_cache: Arc<DashMap<ConnectKey, RealtimeState>>,
) {
    let flush_interval_duration =
        std::time::Duration::from_secs(landscape_common::DEFAULT_METRIC_FLUSH_INTERVAL_SECS);
    let cleanup_interval_duration =
        std::time::Duration::from_secs(landscape_common::DEFAULT_METRIC_CLEANUP_INTERVAL_SECS);
    let summary_sync_duration = std::time::Duration::from_secs(60);

    let rt = tokio::runtime::Builder::new_current_thread().enable_time().build().unwrap();
    rt.block_on(async move {
        let mut metrics_appender: Option<Appender> = Some(conn_disk_writer.appender("conn_metrics").unwrap());
        let mut dns_appender: Option<Appender> = Some(conn_dns.appender("dns_metrics").unwrap());
        let mut batch_count = 0;

        let mut flush_interval = tokio::time::interval(flush_interval_duration);
        let mut cleanup_interval = tokio::time::interval(cleanup_interval_duration);
        let mut summary_sync_interval = tokio::time::interval(summary_sync_duration);

        loop {
            tokio::select! {
                _ = summary_sync_interval.tick() => {
                    let now_ms = landscape_common::utils::time::get_current_time_ms().unwrap_or_default();
                    let cutoff_live = now_ms.saturating_sub(STALE_TIMEOUT_MS);

                    // 1. Flush appenders
                    if let Some(ref mut appender) = metrics_appender {
                        let _ = appender.flush();
                    }

                    // 2. Sync summaries using a SEPARATE connection to avoid borrow issues with appender
                    if let Ok(mut conn_sync) = disk_pool.get() {
                        match conn_sync.transaction() {
                            Ok(tx) => {
                                let mut sync_count = 0;
                                {
                                    let mut stmt = tx.prepare_cached(connect::SUMMARY_INSERT_SQL).unwrap();
                                    for entry in realtime_cache.iter() {
                                        let state = entry.value();
                                        let s = &state.status;
                                        let status_val: u8 = s.status.clone().into();

                                        if let Err(e) = stmt.execute(params![
                                            s.key.create_time as i64,
                                            s.key.cpu_id as i64,
                                            clean_ip_string(&s.src_ip),
                                            clean_ip_string(&s.dst_ip),
                                            s.src_port as i64,
                                            s.dst_port as i64,
                                            s.l4_proto as i64,
                                            s.l3_proto as i64,
                                            s.flow_id as i64,
                                            s.trace_id as i64,
                                            s.last_report_time as i64,
                                            state.last_ingress_bytes as i64,
                                            state.last_egress_bytes as i64,
                                            state.last_ingress_pkts as i64,
                                            state.last_egress_pkts as i64,
                                            status_val as i64,
                                            s.create_time_ms as i64,
                                        ]) {
                                            tracing::error!("Sync summary failed for {}:{}: {}", s.key.create_time, s.key.cpu_id, e);
                                        } else {
                                            sync_count += 1;
                                        }
                                    }
                                }
                                if let Err(e) = tx.commit() {
                                    tracing::error!("Sync summary transaction commit failed: {}", e);
                                } else {
                                    tracing::debug!("Synced {} summaries to disk", sync_count);
                                }
                            }
                            Err(e) => {
                                tracing::error!("Failed to start sync summary transaction on disk writer: {}", e);
                            }
                        }
                    }

                    // 3. Cleanup stale live sessions
                    realtime_cache.retain(|_, v| {
                        let is_disabled = v.status.status == ConnectStatusType::Disabled;
                        let is_stale = v.status.last_report_time < cutoff_live;
                        !is_disabled && !is_stale
                    });
                }

                _ = flush_interval.tick() => {
                    if let Some(ref mut appender) = dns_appender {
                        let _ = appender.flush();
                    }
                    if let Some(ref mut appender) = metrics_appender {
                        let _ = appender.flush();
                    }
                }

                _ = cleanup_interval.tick() => {
                    let now_ms = landscape_common::utils::time::get_current_time_ms().unwrap_or_default();

                    let cutoff_raw = now_ms.saturating_sub(metric_config.conn_retention_mins * MS_PER_MINUTE);
                    let cutoff_1m = now_ms.saturating_sub(metric_config.conn_retention_minute_days * MS_PER_DAY);
                    let cutoff_1h = now_ms.saturating_sub(metric_config.conn_retention_hour_days * MS_PER_DAY);
                    let cutoff_1d = now_ms.saturating_sub(metric_config.conn_retention_day_days * MS_PER_DAY);
                    let cutoff_dns = now_ms.saturating_sub(metric_config.dns_retention_days * MS_PER_DAY);

                    // Flush appenders
                    if let Some(ref mut appender) = dns_appender {
                        let _ = appender.flush();
                    }
                    if let Some(ref mut appender) = metrics_appender {
                        let _ = appender.flush();
                    }

                    dns::cleanup_old_dns_metrics(&conn_dns, cutoff_dns);
                    if let Ok(conn_disk) = disk_pool.get() {
                        // Rollup raw metrics into 1m/1h/1d buckets
                        let _ = connect::perform_inner_db_rollup(&conn_disk);
                        // Use a fresh connection for maintenance instead of conn_disk_writer
                        if let Ok(conn_maint) = disk_pool.get() {
                            let _ = connect::collect_and_cleanup_old_metrics(
                                &conn_maint, &conn_disk, cutoff_raw, cutoff_1m, cutoff_1h, cutoff_1d,
                            );
                        }
                        let _ = connect::aggregate_global_stats(&conn_disk);
                    }

                    tracing::info!(
                        "Auto cleanup metrics, raw: {}, 1m: {}, 1h: {}, 1d: {}, dns: {}",
                        cutoff_raw, cutoff_1m, cutoff_1h, cutoff_1d, cutoff_dns
                    );
                }
                msg_opt = rx.recv() => {
                    match msg_opt {
                        Some(msg) => {
                            let mut current_msg = Some(msg);
                            // Process in batches to reduce select! overhead
                            for _ in 0..metric_config.batch_size.max(100) {
                                if let Some(m) = current_msg.take() {
                                    match m {
                                        DBMessage::InsertMetric(metric) => {
                                            let key = &metric.key;
                                            if let Some(ref mut appender) = metrics_appender {
                                                let _ = appender.append_row(params![
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
                                                    metric.create_time_ms as i64,
                                                ]);
                                            }

                                            // Update realtime cache (DashMap)
                                            let key_clone = metric.key.clone();
                                            let now = metric.report_time;

                                            realtime_cache.entry(key_clone.clone()).and_modify(|e| {
                                                if now > e.status.last_report_time {
                                                    let delta_t = now.saturating_sub(e.status.last_report_time);
                                                    if delta_t > 0 {
                                                        e.status.ingress_bps = (metric.ingress_bytes.saturating_sub(e.last_ingress_bytes)) * 8000 / delta_t;
                                                        e.status.egress_bps = (metric.egress_bytes.saturating_sub(e.last_egress_bytes)) * 8000 / delta_t;
                                                        e.status.ingress_pps = (metric.ingress_packets.saturating_sub(e.last_ingress_pkts)) * 1000 / delta_t;
                                                        e.status.egress_pps = (metric.egress_packets.saturating_sub(e.last_egress_pkts)) * 1000 / delta_t;
                                                    }
                                                    e.status.last_report_time = now;
                                                    // Only update status if the new status is NOT Unknow.
                                                    // Once it's Active, it should stay Active until Disabled.
                                                    if metric.status != ConnectStatusType::Unknow {
                                                        e.status.status = metric.status.clone();
                                                    }
                                                    // Ensure counters only increase
                                                    e.last_ingress_bytes = e.last_ingress_bytes.max(metric.ingress_bytes);
                                                    e.last_egress_bytes = e.last_egress_bytes.max(metric.egress_bytes);
                                                    e.last_ingress_pkts = e.last_ingress_pkts.max(metric.ingress_packets);
                                                    e.last_egress_pkts = e.last_egress_pkts.max(metric.egress_packets);
                                                }
                                            }).or_insert(RealtimeState {
                                                status: landscape_common::metric::connect::ConnectRealtimeStatus {
                                                    key: key_clone,
                                                    src_ip: metric.src_ip,
                                                    dst_ip: metric.dst_ip,
                                                    src_port: metric.src_port,
                                                    dst_port: metric.dst_port,
                                                    l4_proto: metric.l4_proto,
                                                    l3_proto: metric.l3_proto,
                                                    flow_id: metric.flow_id,
                                                    trace_id: metric.trace_id,
                                                    create_time_ms: metric.create_time_ms,
                                                    ingress_bps: 0,
                                                    egress_bps: 0,
                                                    ingress_pps: 0,
                                                    egress_pps: 0,
                                                    last_report_time: now,
                                                    status: metric.status.clone(),
                                                },
                                                last_ingress_bytes: metric.ingress_bytes,
                                                last_egress_bytes: metric.egress_bytes,
                                                last_ingress_pkts: metric.ingress_packets,
                                                last_egress_pkts: metric.egress_packets,
                                            });

                                            batch_count += 1;
                                        }
                                        DBMessage::InsertDnsMetric(metric) => {
                                            if let Some(ref mut appender) = dns_appender {
                                                let _ = appender.append_row(params![
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
                                            }
                                        }

                                        DBMessage::CollectAndCleanupOldMetrics {
                                            cutoff_raw,
                                            cutoff_1m,
                                            cutoff_1h,
                                            cutoff_1d,
                                            cutoff_dns,
                                            resp,
                                        } => {
                                            if let Some(ref mut appender) = dns_appender {
                                                let _ = appender.flush();
                                            }
                                            if let Some(ref mut appender) = metrics_appender {
                                                let _ = appender.flush();
                                            }

                                            dns::cleanup_old_dns_metrics(&conn_dns, cutoff_dns);

                                            let result = if let Ok(conn_disk_pool) = disk_pool.get() {
                                                if let Ok(conn_maint) = disk_pool.get() {
                                                    connect::collect_and_cleanup_old_metrics(
                                                        &conn_maint, &conn_disk_pool, cutoff_raw, cutoff_1m, cutoff_1h, cutoff_1d,
                                                    )
                                                } else {
                                                    Box::new(Vec::new())
                                                }
                                            } else {
                                                Box::new(Vec::new())
                                            };

                                            let _ = resp.send(result);
                                        }
                                    }

                                    if batch_count >= metric_config.batch_size {
                                        if let Some(ref mut appender) = dns_appender {
                                            let _ = appender.flush();
                                        }
                                        if let Some(ref mut appender) = metrics_appender {
                                            let _ = appender.flush();
                                        }
                                        batch_count = 0;
                                    }
                                }

                                // Try to get next message without blocking
                                match rx.try_recv() {
                                    Ok(m) => current_msg = Some(m),
                                    Err(_) => break,
                                }
                            }
                        }
                        None => break,
                    }
                }
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
        let config_clone = config.clone();

        // Create independent disk pool (disk database)

        let disk_manager = DuckdbConnectionManager::file_with_flags(
            &db_path,
            duckdb::Config::default()
                .threads(config.max_threads as i64)
                .unwrap()
                .max_memory(&format!("{}MB", config.max_memory))
                .unwrap(),
        )
        .unwrap();

        let disk_pool = r2d2::Pool::builder()
            .max_size(8) // Disk pool for queries and sync
            .max_lifetime(Some(std::time::Duration::from_secs(120)))
            .build(disk_manager)
            .expect("Failed to create disk connection pool");

        // Initialize tables in disk database
        let conn_disk = disk_pool.get().expect("Failed to get disk connection");

        // Performance optimizations for disk database
        let _ = conn_disk.execute("PRAGMA wal_autocheckpoint='256MB'", []);

        connect::create_summaries_table(&conn_disk, "");
        connect::create_metrics_table(&conn_disk, "")
            .expect("Failed to create connect metrics tables on disk");
        connect::create_live_tables(&conn_disk)
            .expect("Failed to create raw metric tables on disk");
        dns::create_dns_table(&conn_disk, "").expect("Failed to create DNS metrics tables on disk");

        let thread_disk_pool = disk_pool.clone();
        let conn_dns = disk_pool.get().expect("Failed to get DNS writer connection from disk pool");
        let conn_disk_writer = disk_pool.get().expect("Failed to get disk writer connection");

        let realtime_cache = Arc::new(DashMap::new());
        let thread_realtime_cache = realtime_cache.clone();

        thread::spawn(move || {
            start_db_thread(
                rx,
                config_clone,
                thread_disk_pool,
                conn_dns,
                conn_disk_writer,
                thread_realtime_cache,
            );
        });

        DuckMetricStore { tx, db_path, config, disk_pool, realtime_cache }
    }

    /// Get a connection from the disk pool
    fn get_disk_conn(&self) -> r2d2::PooledConnection<DuckdbConnectionManager> {
        self.disk_pool.get().expect("Failed to get disk connection from pool")
    }

    pub async fn insert_metric(&self, metric: ConnectMetric) {
        let _ = self.tx.send(DBMessage::InsertMetric(metric)).await;
    }

    pub async fn connect_infos(
        &self,
    ) -> Vec<landscape_common::metric::connect::ConnectRealtimeStatus> {
        let mut infos: Vec<_> =
            self.realtime_cache.iter().map(|v| v.value().status.clone()).collect();
        // Sort by last_report_time DESC to match original behavior
        infos.sort_by(|a, b| b.last_report_time.cmp(&a.last_report_time));
        infos
    }

    pub async fn get_realtime_ip_stats(
        &self,
        is_src: bool,
    ) -> Vec<landscape_common::metric::connect::IpRealtimeStat> {
        use std::collections::HashMap;
        let mut stats_map: HashMap<IpAddr, landscape_common::metric::connect::IpAggregatedStats> =
            HashMap::new();

        for entry in self.realtime_cache.iter() {
            let status = &entry.value().status;
            let ip = if is_src { status.src_ip } else { status.dst_ip };

            let stats = stats_map.entry(ip).or_default();
            stats.ingress_bps += status.ingress_bps;
            stats.egress_bps += status.egress_bps;
            stats.ingress_pps += status.ingress_pps;
            stats.egress_pps += status.egress_pps;
            stats.active_conns += 1;
        }

        stats_map
            .into_iter()
            .map(|(ip, stats)| landscape_common::metric::connect::IpRealtimeStat { ip, stats })
            .collect()
    }

    pub async fn query_metric_by_key(
        &self,
        key: ConnectKey,
        resolution: MetricResolution,
    ) -> Vec<ConnectMetricPoint> {
        let store = self.clone();
        tokio::task::spawn_blocking(move || -> Vec<ConnectMetricPoint> {
            let conn = store.get_disk_conn();
            connect::query_metric_by_key(&conn, &key, resolution, Some(&store.db_path))
        })
        .await
        .unwrap_or_default()
    }

    pub async fn collect_and_cleanup_old_metrics(
        &self,
        cutoff_raw: u64,
        cutoff_1m: u64,
        cutoff_1h: u64,
        cutoff_1d: u64,
    ) -> Box<Vec<ConnectMetric>> {
        let (resp, rx) = oneshot::channel();
        let now_ms = landscape_common::utils::time::get_current_time_ms().unwrap_or_default();
        let cutoff_dns = now_ms.saturating_sub(self.config.dns_retention_days * MS_PER_DAY);

        let _ = self
            .tx
            .send(DBMessage::CollectAndCleanupOldMetrics {
                cutoff_raw,
                cutoff_1m,
                cutoff_1h,
                cutoff_1d,
                cutoff_dns,
                resp,
            })
            .await;
        rx.await.unwrap()
    }

    pub async fn history_summaries_complex(
        &self,
        params: ConnectHistoryQueryParams,
    ) -> Vec<ConnectHistoryStatus> {
        let store = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = store.get_disk_conn();
            connect::query_historical_summaries_complex(&conn, params, Some(&store.db_path))
        })
        .await
        .unwrap_or_default()
    }

    pub async fn history_src_ip_stats(
        &self,
        params: ConnectHistoryQueryParams,
    ) -> Vec<landscape_common::metric::connect::IpHistoryStat> {
        let store = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = store.get_disk_conn();
            connect::query_connection_ip_history(&conn, params, true, Some(&store.db_path))
        })
        .await
        .unwrap_or_default()
    }

    pub async fn history_dst_ip_stats(
        &self,
        params: ConnectHistoryQueryParams,
    ) -> Vec<landscape_common::metric::connect::IpHistoryStat> {
        let store = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = store.get_disk_conn();
            connect::query_connection_ip_history(&conn, params, false, Some(&store.db_path))
        })
        .await
        .unwrap_or_default()
    }

    pub async fn get_global_stats(&self) -> ConnectGlobalStats {
        let store = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = store.get_disk_conn();
            connect::query_global_stats(&conn)
        })
        .await
        .unwrap_or_default()
    }

    pub async fn insert_dns_metric(&self, mut metric: DnsMetric) {
        if metric.domain.ends_with('.') && metric.domain.len() > 1 {
            metric.domain.pop();
        }
        let _ = self.tx.send(DBMessage::InsertDnsMetric(metric)).await;
    }

    pub async fn query_dns_history(&self, params: DnsHistoryQueryParams) -> DnsHistoryResponse {
        let store = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = store.get_disk_conn();
            dns::query_dns_history(&conn, params)
        })
        .await
        .unwrap_or(DnsHistoryResponse { items: Vec::new(), total: 0 })
    }

    pub async fn get_dns_summary(&self, params: DnsSummaryQueryParams) -> DnsSummaryResponse {
        let store = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = store.get_disk_conn();
            dns::query_dns_summary(&conn, params)
        })
        .await
        .unwrap_or_else(|_| DnsSummaryResponse::default())
    }

    pub async fn get_dns_lightweight_summary(
        &self,
        params: DnsSummaryQueryParams,
    ) -> DnsLightweightSummaryResponse {
        let store = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = store.get_disk_conn();
            dns::query_dns_lightweight_summary(&conn, params)
        })
        .await
        .unwrap_or_else(|_| DnsLightweightSummaryResponse::default())
    }
}
