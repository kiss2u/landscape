use duckdb::{params, Appender, Connection, DuckdbConnectionManager};

use landscape_common::concurrency::{
    runtime_thread_name_fn, spawn_named_thread, task_label, thread_name,
};
use landscape_common::config::MetricRuntimeConfig;
use landscape_common::event::{ConnectMessage, DnsMetricMessage};
use landscape_common::metric::connect::{
    ConnectGlobalStats, ConnectHistoryQueryParams, ConnectHistoryStatus, ConnectKey, ConnectMetric,
    ConnectMetricPoint, ConnectRealtimeStatus, ConnectStatusType, IpAggregatedStats,
    MetricResolution,
};
use landscape_common::metric::dns::{
    DnsHistoryQueryParams, DnsHistoryResponse, DnsLightweightSummaryResponse, DnsMetric,
    DnsSummaryQueryParams, DnsSummaryResponse,
};
use r2d2::{self, PooledConnection};
use std::collections::{HashMap, VecDeque};
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tokio::runtime::{Builder as RuntimeBuilder, Runtime};
use tokio::sync::mpsc;

pub mod connect;
pub mod dns;

const CHANNEL_CAPACITY: usize = 1024;
const MS_PER_MINUTE: u64 = 60 * 1000;
const MS_PER_TEN_MINUTES: u64 = 10 * MS_PER_MINUTE;
const MS_PER_HOUR: u64 = 60 * MS_PER_MINUTE;
const MS_PER_DAY: u64 = 24 * MS_PER_HOUR;
const STALE_TIMEOUT_MS: u64 = 5 * MS_PER_MINUTE;
const DEFAULT_CONNECT_SAMPLE_INTERVAL_MS: u64 = 5 * 1000;

fn bucket_start(report_time: u64, bucket_ms: u64) -> u64 {
    report_time / bucket_ms * bucket_ms
}

fn minute_slot(report_time: u64) -> u64 {
    bucket_start(report_time, MS_PER_MINUTE)
}

fn hour_refresh_slot(report_time: u64) -> u64 {
    bucket_start(report_time, MS_PER_TEN_MINUTES)
}

fn day_refresh_slot(report_time: u64) -> u64 {
    bucket_start(report_time, MS_PER_HOUR)
}

fn second_window_ms(config: &MetricRuntimeConfig) -> u64 {
    config.connect_second_window_minutes.max(1).saturating_mul(MS_PER_MINUTE)
}

fn second_ring_capacity(config: &MetricRuntimeConfig) -> usize {
    let target_points = second_window_ms(config) / DEFAULT_CONNECT_SAMPLE_INTERVAL_MS;
    target_points.saturating_add(8).clamp(32, 4096) as usize
}

pub(super) fn clean_ip_string(ip: &IpAddr) -> String {
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

fn metric_to_point(metric: &ConnectMetric) -> ConnectMetricPoint {
    ConnectMetricPoint {
        report_time: metric.report_time,
        ingress_bytes: metric.ingress_bytes,
        ingress_packets: metric.ingress_packets,
        egress_bytes: metric.egress_bytes,
        egress_packets: metric.egress_packets,
        status: metric.status.clone(),
    }
}

fn metric_to_realtime(metric: &ConnectMetric) -> ConnectRealtimeStatus {
    ConnectRealtimeStatus {
        key: metric.key.clone(),
        src_ip: metric.src_ip,
        dst_ip: metric.dst_ip,
        src_port: metric.src_port,
        dst_port: metric.dst_port,
        l4_proto: metric.l4_proto,
        l3_proto: metric.l3_proto,
        flow_id: metric.flow_id,
        trace_id: metric.trace_id,
        gress: metric.gress,
        create_time_ms: metric.create_time_ms,
        ingress_bps: 0,
        egress_bps: 0,
        ingress_pps: 0,
        egress_pps: 0,
        last_report_time: metric.report_time,
        status: metric.status.clone(),
    }
}

#[derive(Clone)]
struct FlowState {
    last_metric: ConnectMetric,
    realtime: ConnectRealtimeStatus,
    second_ring: VecDeque<ConnectMetricPoint>,
    last_minute_slot: u64,
    last_hour_refresh_slot: u64,
    last_day_refresh_slot: u64,
    finalized: bool,
}

impl FlowState {
    fn new(metric: ConnectMetric, window_ms: u64, ring_cap: usize) -> Self {
        let report_time = metric.report_time;
        let mut second_ring = VecDeque::with_capacity(ring_cap.max(1));
        second_ring.push_back(metric_to_point(&metric));

        let mut state = Self {
            realtime: metric_to_realtime(&metric),
            last_metric: metric,
            second_ring,
            last_minute_slot: minute_slot(report_time),
            last_hour_refresh_slot: hour_refresh_slot(report_time),
            last_day_refresh_slot: day_refresh_slot(report_time),
            finalized: false,
        };
        state.trim_second_ring(window_ms, ring_cap);
        state
    }

    fn update_from_metric(&mut self, metric: ConnectMetric, window_ms: u64, ring_cap: usize) {
        if metric.report_time > self.last_metric.report_time {
            let delta_t = metric.report_time.saturating_sub(self.last_metric.report_time);
            if delta_t > 0 {
                self.realtime.ingress_bps =
                    metric.ingress_bytes.saturating_sub(self.last_metric.ingress_bytes) * 8000
                        / delta_t;
                self.realtime.egress_bps =
                    metric.egress_bytes.saturating_sub(self.last_metric.egress_bytes) * 8000
                        / delta_t;
                self.realtime.ingress_pps =
                    metric.ingress_packets.saturating_sub(self.last_metric.ingress_packets) * 1000
                        / delta_t;
                self.realtime.egress_pps =
                    metric.egress_packets.saturating_sub(self.last_metric.egress_packets) * 1000
                        / delta_t;
            }
        }

        self.realtime.last_report_time = metric.report_time;
        self.realtime.src_ip = metric.src_ip;
        self.realtime.dst_ip = metric.dst_ip;
        self.realtime.src_port = metric.src_port;
        self.realtime.dst_port = metric.dst_port;
        self.realtime.l4_proto = metric.l4_proto;
        self.realtime.l3_proto = metric.l3_proto;
        self.realtime.flow_id = metric.flow_id;
        self.realtime.trace_id = metric.trace_id;
        self.realtime.gress = metric.gress;
        self.realtime.create_time_ms = metric.create_time_ms;
        if metric.status != ConnectStatusType::Unknow {
            self.realtime.status = metric.status.clone();
        }

        self.last_metric = metric.clone();
        self.second_ring.push_back(metric_to_point(&metric));
        self.finalized = false;
        self.trim_second_ring(window_ms, ring_cap);
    }

    fn trim_second_ring(&mut self, window_ms: u64, ring_cap: usize) {
        let cutoff = self.realtime.last_report_time.saturating_sub(window_ms);
        self.trim_second_ring_before(cutoff);
        while self.second_ring.len() > ring_cap.max(1) {
            self.second_ring.pop_front();
        }
    }

    fn trim_second_ring_before(&mut self, cutoff: u64) {
        while let Some(point) = self.second_ring.front() {
            if point.report_time >= cutoff {
                break;
            }
            self.second_ring.pop_front();
        }
    }

    fn second_points_since(&self, cutoff: u64) -> Vec<ConnectMetricPoint> {
        self.second_ring.iter().filter(|point| point.report_time >= cutoff).cloned().collect()
    }

    fn is_active(&self, now_ms: u64) -> bool {
        !self.finalized
            && self.realtime.status != ConnectStatusType::Disabled
            && self.realtime.last_report_time >= now_ms.saturating_sub(STALE_TIMEOUT_MS)
    }
}

type FlowCache = Arc<RwLock<HashMap<ConnectKey, FlowState>>>;

#[derive(Clone)]
pub struct DuckMetricStore {
    connect_tx: mpsc::Sender<ConnectMessage>,
    dns_tx: mpsc::Sender<DnsMetricMessage>,
    pub config: MetricRuntimeConfig,
    pub disk_pool: r2d2::Pool<DuckdbConnectionManager>,
    flow_cache: FlowCache,
    query_runtime: Arc<Runtime>,
}

fn upsert_bucket_row(
    conn: &Connection,
    table: &str,
    metric: &ConnectMetric,
    bucket_report_time: u64,
) {
    if let Err(error) = connect::upsert_metric_bucket(conn, table, metric, bucket_report_time) {
        tracing::error!(
            "failed to upsert {} bucket for {}:{} at {}: {}",
            table,
            metric.key.create_time,
            metric.key.cpu_id,
            bucket_report_time,
            error,
        );
    }
}

fn upsert_summary_row(conn: &Connection, metric: &ConnectMetric) {
    if let Err(error) = connect::upsert_summary(conn, metric) {
        tracing::error!(
            "failed to upsert summary for {}:{}: {}",
            metric.key.create_time,
            metric.key.cpu_id,
            error,
        );
    }
}

fn finalize_state(conn: &Connection, state: &mut FlowState, mark_disabled: bool) {
    if state.finalized {
        return;
    }

    let mut metric = state.last_metric.clone();
    if mark_disabled {
        metric.status = ConnectStatusType::Disabled;
        state.last_metric.status = ConnectStatusType::Disabled;
        state.realtime.status = ConnectStatusType::Disabled;
    }

    upsert_bucket_row(conn, "conn_metrics_1m", &metric, minute_slot(metric.report_time));
    upsert_bucket_row(
        conn,
        "conn_metrics_1h",
        &metric,
        bucket_start(metric.report_time, MS_PER_HOUR),
    );
    upsert_bucket_row(
        conn,
        "conn_metrics_1d",
        &metric,
        bucket_start(metric.report_time, MS_PER_DAY),
    );
    upsert_summary_row(conn, &metric);
    state.finalized = true;
}

fn process_connect_metric(
    conn: &Connection,
    flow_cache: &FlowCache,
    metric: ConnectMetric,
    second_window_ms: u64,
    second_ring_cap: usize,
) {
    let curr_minute_slot = minute_slot(metric.report_time);
    let curr_hour_refresh_slot = hour_refresh_slot(metric.report_time);
    let curr_day_refresh_slot = day_refresh_slot(metric.report_time);

    let mut cache = flow_cache.write().expect("metric flow cache poisoned");
    match cache.entry(metric.key.clone()) {
        std::collections::hash_map::Entry::Occupied(mut entry) => {
            let state = entry.get_mut();
            if metric.report_time < state.last_metric.report_time {
                return;
            }

            let previous_minute_bucket = minute_slot(state.last_metric.report_time);
            let previous_hour_bucket = bucket_start(state.last_metric.report_time, MS_PER_HOUR);
            let previous_day_bucket = bucket_start(state.last_metric.report_time, MS_PER_DAY);

            if curr_minute_slot > state.last_minute_slot {
                upsert_bucket_row(conn, "conn_metrics_1m", &metric, previous_minute_bucket);
                upsert_summary_row(conn, &metric);
                state.last_minute_slot = curr_minute_slot;
            }
            if curr_hour_refresh_slot > state.last_hour_refresh_slot {
                upsert_bucket_row(conn, "conn_metrics_1h", &metric, previous_hour_bucket);
                state.last_hour_refresh_slot = curr_hour_refresh_slot;
            }
            if curr_day_refresh_slot > state.last_day_refresh_slot {
                upsert_bucket_row(conn, "conn_metrics_1d", &metric, previous_day_bucket);
                state.last_day_refresh_slot = curr_day_refresh_slot;
            }

            let should_finalize = metric.status == ConnectStatusType::Disabled;
            state.update_from_metric(metric, second_window_ms, second_ring_cap);
            if should_finalize {
                finalize_state(conn, state, true);
            }
        }
        std::collections::hash_map::Entry::Vacant(entry) => {
            let should_finalize = metric.status == ConnectStatusType::Disabled;
            let mut state = FlowState::new(metric, second_window_ms, second_ring_cap);
            if should_finalize {
                finalize_state(conn, &mut state, true);
            }
            entry.insert(state);
        }
    }
}

#[derive(Default)]
struct FlowCacheStats {
    active_flows: usize,
    finalized_flows: usize,
    finalized_in_run: usize,
    second_ring_points: usize,
}

fn cleanup_flow_cache(
    conn: &Connection,
    flow_cache: &FlowCache,
    now_ms: u64,
    second_window_ms: u64,
) -> FlowCacheStats {
    let stale_cutoff = now_ms.saturating_sub(STALE_TIMEOUT_MS);
    let window_cutoff = now_ms.saturating_sub(second_window_ms);

    let mut cache = flow_cache.write().expect("metric flow cache poisoned");
    let mut expired_keys = Vec::new();
    let mut stats = FlowCacheStats::default();

    for (key, state) in cache.iter_mut() {
        if !state.finalized && state.realtime.last_report_time < stale_cutoff {
            finalize_state(conn, state, true);
            stats.finalized_in_run += 1;
        }

        state.trim_second_ring_before(window_cutoff);
        stats.second_ring_points += state.second_ring.len();

        if state.finalized {
            stats.finalized_flows += 1;
        } else if state.is_active(now_ms) {
            stats.active_flows += 1;
        }

        if state.finalized && state.realtime.last_report_time < window_cutoff {
            expired_keys.push(key.clone());
        }
    }

    for key in expired_keys {
        cache.remove(&key);
    }

    stats
}

fn finalize_all_flows(conn: &Connection, flow_cache: &FlowCache) {
    let mut cache = flow_cache.write().expect("metric flow cache poisoned");
    for state in cache.values_mut() {
        finalize_state(conn, state, true);
    }
}

fn start_db_thread(
    mut connect_rx: mpsc::Receiver<ConnectMessage>,
    mut dns_rx: mpsc::Receiver<DnsMetricMessage>,
    metric_config: MetricRuntimeConfig,
    disk_pool: r2d2::Pool<DuckdbConnectionManager>,
    conn_dns: PooledConnection<DuckdbConnectionManager>,
    conn_connect_writer: PooledConnection<DuckdbConnectionManager>,
    flow_cache: FlowCache,
) {
    let flush_interval_duration =
        std::time::Duration::from_secs(metric_config.write_flush_interval_secs.max(1));
    let cleanup_interval_duration =
        std::time::Duration::from_secs(metric_config.cleanup_interval_secs.max(1));
    let second_window_ms = second_window_ms(&metric_config);
    let second_ring_cap = second_ring_capacity(&metric_config);

    let rt = tokio::runtime::Builder::new_current_thread().enable_time().build().unwrap();
    rt.block_on(async move {
        let mut dns_appender: Option<Appender> = Some(conn_dns.appender("dns_metrics").unwrap());
        let mut dns_batch_count = 0usize;
        let mut flush_interval = tokio::time::interval(flush_interval_duration);
        let mut cleanup_interval = tokio::time::interval(cleanup_interval_duration);
        let mut connect_closed = false;
        let mut dns_closed = false;

        loop {
            tokio::select! {
                _ = flush_interval.tick() => {
                    if let Some(ref mut appender) = dns_appender {
                        let _ = appender.flush();
                    }
                }
                _ = cleanup_interval.tick() => {
                    if let Some(ref mut appender) = dns_appender {
                        let _ = appender.flush();
                    }

                    let now_ms = landscape_common::utils::time::get_current_time_ms().unwrap_or_default();
                    let flow_stats =
                        cleanup_flow_cache(&conn_connect_writer, &flow_cache, now_ms, second_window_ms);

                    let cutoff_1m = now_ms.saturating_sub(metric_config.connect_1m_retention_days * MS_PER_DAY);
                    let cutoff_1h = now_ms.saturating_sub(metric_config.connect_1h_retention_days * MS_PER_DAY);
                    let cutoff_1d = now_ms.saturating_sub(metric_config.connect_1d_retention_days * MS_PER_DAY);
                    let cutoff_dns = now_ms.saturating_sub(metric_config.dns_retention_days * MS_PER_DAY);

                    dns::cleanup_old_dns_metrics(&conn_dns, cutoff_dns);
                    if let Ok(conn_agg) = disk_pool.get() {
                        let stats = connect::cleanup_old_metrics_only(
                            &conn_agg,
                            cutoff_1m,
                            cutoff_1h,
                            cutoff_1d,
                            metric_config.cleanup_time_budget_ms,
                            metric_config.cleanup_slice_window_secs,
                        );
                        tracing::info!(
                            "phase=cleanup elapsed_ms={} budget_hit={} deleted_1m={} deleted_1h={} deleted_1d={} deleted_summaries={} active_flows={} finalized_flows={} finalized_in_run={} second_ring_points={}",
                            stats.elapsed_ms,
                            stats.budget_hit,
                            stats.deleted_1m,
                            stats.deleted_1h,
                            stats.deleted_1d,
                            stats.deleted_summaries,
                            flow_stats.active_flows,
                            flow_stats.finalized_flows,
                            flow_stats.finalized_in_run,
                            flow_stats.second_ring_points,
                        );
                    }
                }
                msg_opt = connect_rx.recv(), if !connect_closed => {
                    match msg_opt {
                        Some(ConnectMessage::Metric(metric)) => {
                            process_connect_metric(
                                &conn_connect_writer,
                                &flow_cache,
                                metric,
                                second_window_ms,
                                second_ring_cap,
                            );
                        }
                        None => connect_closed = true,
                    }
                }
                msg_opt = dns_rx.recv(), if !dns_closed => {
                    match msg_opt {
                        Some(DnsMetricMessage::Metric(metric)) => {
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
                            dns_batch_count += 1;
                            if dns_batch_count >= metric_config.write_batch_size.max(1) {
                                if let Some(ref mut appender) = dns_appender {
                                    let _ = appender.flush();
                                }
                                dns_batch_count = 0;
                            }
                        }
                        None => dns_closed = true,
                    }
                }
            }

            if connect_closed && dns_closed {
                break;
            }
        }

        finalize_all_flows(&conn_connect_writer, &flow_cache);
        if let Some(ref mut appender) = dns_appender {
            let _ = appender.flush();
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

        let (connect_tx, connect_rx) = mpsc::channel::<ConnectMessage>(CHANNEL_CAPACITY);
        let (dns_tx, dns_rx) = mpsc::channel::<DnsMetricMessage>(CHANNEL_CAPACITY);
        let config_clone = config.clone();

        let disk_manager = DuckdbConnectionManager::file_with_flags(
            &db_path,
            duckdb::Config::default()
                .threads(config.db_max_threads as i64)
                .unwrap()
                .max_memory(&format!("{}MB", config.db_max_memory_mb))
                .unwrap(),
        )
        .unwrap();

        let disk_pool = r2d2::Pool::builder()
            .max_size(8)
            .max_lifetime(Some(std::time::Duration::from_secs(120)))
            .build(disk_manager)
            .expect("Failed to create disk connection pool");

        let conn_disk = disk_pool.get().expect("Failed to get disk connection");
        let _ = conn_disk.execute("PRAGMA wal_autocheckpoint='256MB'", []);

        connect::create_summaries_table(&conn_disk);
        connect::create_metrics_table(&conn_disk)
            .expect("Failed to create connect metrics tables on disk");
        dns::create_dns_table(&conn_disk).expect("Failed to create DNS metrics tables on disk");

        let thread_disk_pool = disk_pool.clone();
        let conn_dns = disk_pool.get().expect("Failed to get DNS writer connection from disk pool");
        let conn_connect_writer = disk_pool.get().expect("Failed to get connect writer connection");
        let flow_cache: FlowCache = Arc::new(RwLock::new(HashMap::new()));
        let thread_flow_cache = flow_cache.clone();

        let query_runtime = Arc::new(
            RuntimeBuilder::new_multi_thread()
                .worker_threads(1)
                .max_blocking_threads(config.db_max_threads.max(2))
                .thread_name_fn(runtime_thread_name_fn(thread_name::prefix::METRIC_QUERY_RUNTIME))
                .build()
                .expect("failed to create metric query runtime"),
        );

        spawn_named_thread(thread_name::fixed::METRIC_DB_WRITER, move || {
            start_db_thread(
                connect_rx,
                dns_rx,
                config_clone,
                thread_disk_pool,
                conn_dns,
                conn_connect_writer,
                thread_flow_cache,
            );
        })
        .expect("failed to spawn metric db thread");

        DuckMetricStore {
            connect_tx,
            dns_tx,
            config,
            disk_pool,
            flow_cache,
            query_runtime,
        }
    }

    fn get_disk_conn(&self) -> r2d2::PooledConnection<DuckdbConnectionManager> {
        self.disk_pool.get().expect("Failed to get disk connection from pool")
    }

    pub fn get_connect_msg_channel(&self) -> mpsc::Sender<ConnectMessage> {
        self.connect_tx.clone()
    }

    pub fn get_dns_msg_channel(&self) -> mpsc::Sender<DnsMetricMessage> {
        self.dns_tx.clone()
    }

    pub async fn connect_infos(&self) -> Vec<ConnectRealtimeStatus> {
        let now_ms = landscape_common::utils::time::get_current_time_ms().unwrap_or_default();
        let cache = self.flow_cache.read().expect("metric flow cache poisoned");
        let mut infos: Vec<_> = cache
            .values()
            .filter(|state| state.is_active(now_ms))
            .map(|state| state.realtime.clone())
            .collect();
        infos.sort_by(|a, b| b.last_report_time.cmp(&a.last_report_time));
        infos
    }

    pub async fn get_realtime_ip_stats(
        &self,
        is_src: bool,
    ) -> Vec<landscape_common::metric::connect::IpRealtimeStat> {
        let now_ms = landscape_common::utils::time::get_current_time_ms().unwrap_or_default();
        let cache = self.flow_cache.read().expect("metric flow cache poisoned");
        let mut stats_map: HashMap<IpAddr, IpAggregatedStats> = HashMap::new();

        for state in cache.values().filter(|state| state.is_active(now_ms)) {
            let ip = if is_src { state.realtime.src_ip } else { state.realtime.dst_ip };
            let stats = stats_map.entry(ip).or_default();
            stats.ingress_bps += state.realtime.ingress_bps;
            stats.egress_bps += state.realtime.egress_bps;
            stats.ingress_pps += state.realtime.ingress_pps;
            stats.egress_pps += state.realtime.egress_pps;
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
        if resolution == MetricResolution::Second {
            let cutoff = landscape_common::utils::time::get_current_time_ms()
                .unwrap_or_default()
                .saturating_sub(second_window_ms(&self.config));
            let cache = self.flow_cache.read().expect("metric flow cache poisoned");
            return cache
                .get(&key)
                .map(|state| state.second_points_since(cutoff))
                .unwrap_or_default();
        }

        self.run_query(
            task_label::op::METRIC_QUERY_BY_KEY,
            move |store| -> Vec<ConnectMetricPoint> {
                let conn = store.get_disk_conn();
                connect::query_metric_by_key(&conn, &key, resolution)
            },
        )
        .await
    }

    pub async fn history_summaries_complex(
        &self,
        params: ConnectHistoryQueryParams,
    ) -> Vec<ConnectHistoryStatus> {
        self.run_query(task_label::op::METRIC_HISTORY_SUMMARIES, move |store| {
            let conn = store.get_disk_conn();
            connect::query_historical_summaries_complex(&conn, params)
        })
        .await
    }

    pub async fn history_src_ip_stats(
        &self,
        params: ConnectHistoryQueryParams,
    ) -> Vec<landscape_common::metric::connect::IpHistoryStat> {
        self.run_query(task_label::op::METRIC_HISTORY_SRC_IP, move |store| {
            let conn = store.get_disk_conn();
            connect::query_connection_ip_history(&conn, params, true)
        })
        .await
    }

    pub async fn history_dst_ip_stats(
        &self,
        params: ConnectHistoryQueryParams,
    ) -> Vec<landscape_common::metric::connect::IpHistoryStat> {
        self.run_query(task_label::op::METRIC_HISTORY_DST_IP, move |store| {
            let conn = store.get_disk_conn();
            connect::query_connection_ip_history(&conn, params, false)
        })
        .await
    }

    pub async fn get_global_stats(&self) -> ConnectGlobalStats {
        self.run_query(task_label::op::METRIC_GLOBAL_STATS, move |store| {
            let conn = store.get_disk_conn();
            connect::query_global_stats(&conn)
        })
        .await
    }

    pub async fn insert_dns_metric(&self, mut metric: DnsMetric) {
        if metric.domain.ends_with('.') && metric.domain.len() > 1 {
            metric.domain.pop();
        }
        let _ = self.dns_tx.send(DnsMetricMessage::Metric(metric)).await;
    }

    pub async fn query_dns_history(&self, params: DnsHistoryQueryParams) -> DnsHistoryResponse {
        let store = self.clone();
        let span = tracing::info_span!(
            "task",
            task = task_label::task::METRIC_QUERY,
            op = task_label::op::METRIC_DNS_HISTORY
        );
        self.query_runtime
            .handle()
            .spawn_blocking(move || {
                span.in_scope(|| {
                    let conn = store.get_disk_conn();
                    dns::query_dns_history(&conn, params)
                })
            })
            .await
            .unwrap_or(DnsHistoryResponse { items: Vec::new(), total: 0 })
    }

    pub async fn get_dns_summary(&self, params: DnsSummaryQueryParams) -> DnsSummaryResponse {
        self.run_query(task_label::op::METRIC_DNS_SUMMARY, move |store| {
            let conn = store.get_disk_conn();
            dns::query_dns_summary(&conn, params)
        })
        .await
    }

    pub async fn get_dns_lightweight_summary(
        &self,
        params: DnsSummaryQueryParams,
    ) -> DnsLightweightSummaryResponse {
        self.run_query(task_label::op::METRIC_DNS_LIGHTWEIGHT_SUMMARY, move |store| {
            let conn = store.get_disk_conn();
            dns::query_dns_lightweight_summary(&conn, params)
        })
        .await
    }

    async fn run_query<T, F>(&self, op: &'static str, f: F) -> T
    where
        T: Default + Send + 'static,
        F: FnOnce(Self) -> T + Send + 'static,
    {
        let store = self.clone();
        let span = tracing::info_span!("task", task = task_label::task::METRIC_QUERY, op = op);
        self.query_runtime
            .handle()
            .spawn_blocking(move || span.in_scope(|| f(store)))
            .await
            .unwrap_or_default()
    }
}
