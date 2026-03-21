use duckdb::DuckdbConnectionManager;
use landscape_common::concurrency::{spawn_named_thread, task_label, thread_name};
use landscape_common::config::MetricRuntimeConfig;
use landscape_common::event::{ConnectMessage, DnsMetricMessage};
use landscape_common::metric::connect::{
    ConnectHistoryQueryParams, ConnectHistoryStatus, ConnectKey, ConnectMetricPoint,
    ConnectRealtimeStatus, IpHistoryStat, IpRealtimeStat, MetricResolution,
};
use landscape_common::metric::dns::{
    DnsHistoryQueryParams, DnsHistoryResponse, DnsLightweightSummaryResponse,
    DnsSummaryQueryParams, DnsSummaryResponse,
};
use r2d2::{self, PooledConnection};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tokio::sync::{mpsc, Mutex};

use super::connect::{query as connect_query, schema as connect_schema};
use super::dns::{history as dns_history, schema as dns_schema, summary as dns_summary};
use super::global_stats::spawn_global_stats_refresh_task;
use super::ingest::{
    collect_connect_infos, collect_realtime_ip_stats, second_points_by_key, second_window_ms,
    start_db_thread, FlowCache, CHANNEL_CAPACITY,
};

#[derive(Clone)]
pub struct DuckMetricStore {
    connect_tx: mpsc::Sender<ConnectMessage>,
    dns_tx: mpsc::Sender<DnsMetricMessage>,
    pub config: MetricRuntimeConfig,
    pub disk_pool: r2d2::Pool<DuckdbConnectionManager>,
    pub(crate) flow_cache: FlowCache,
    pub(crate) global_stats_cache:
        Arc<RwLock<landscape_common::metric::connect::ConnectGlobalStats>>,
    pub(crate) global_stats_refresh_lock: Arc<Mutex<()>>,
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

        connect_schema::create_summaries_table(&conn_disk);
        connect_schema::create_metrics_table(&conn_disk)
            .expect("Failed to create connect metrics tables on disk");
        dns_schema::create_dns_table(&conn_disk)
            .expect("Failed to create DNS metrics tables on disk");
        let initial_global_stats = match connect_query::query_global_stats(&conn_disk) {
            Ok(stats) => stats,
            Err(error) => {
                tracing::error!("failed to prewarm connect global stats cache: {}", error);
                landscape_common::metric::connect::ConnectGlobalStats::default()
            }
        };

        let thread_disk_pool = disk_pool.clone();
        let conn_dns: PooledConnection<DuckdbConnectionManager> =
            disk_pool.get().expect("Failed to get DNS writer connection from disk pool");
        let conn_connect_writer = disk_pool.get().expect("Failed to get connect writer connection");
        let flow_cache: FlowCache = Arc::new(RwLock::new(HashMap::new()));
        let thread_flow_cache = flow_cache.clone();
        let global_stats_cache = Arc::new(RwLock::new(initial_global_stats));
        let global_stats_refresh_lock = Arc::new(Mutex::new(()));

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

        let store = DuckMetricStore {
            connect_tx,
            dns_tx,
            config,
            disk_pool,
            flow_cache,
            global_stats_cache,
            global_stats_refresh_lock,
        };
        spawn_global_stats_refresh_task(store.clone());
        store
    }

    pub fn get_connect_msg_channel(&self) -> mpsc::Sender<ConnectMessage> {
        self.connect_tx.clone()
    }

    pub fn get_dns_msg_channel(&self) -> mpsc::Sender<DnsMetricMessage> {
        self.dns_tx.clone()
    }

    pub async fn connect_infos(&self) -> Vec<ConnectRealtimeStatus> {
        let now_ms = landscape_common::utils::time::get_current_time_ms().unwrap_or_default();
        collect_connect_infos(&self.flow_cache, now_ms)
    }

    pub async fn get_realtime_ip_stats(&self, is_src: bool) -> Vec<IpRealtimeStat> {
        let now_ms = landscape_common::utils::time::get_current_time_ms().unwrap_or_default();
        collect_realtime_ip_stats(&self.flow_cache, now_ms, is_src)
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
            return second_points_by_key(&self.flow_cache, &key, cutoff);
        }

        self.run_query_default(task_label::op::METRIC_QUERY_BY_KEY, move |store| {
            let conn = store.get_disk_conn();
            connect_query::query_metric_by_key(&conn, &key, resolution)
        })
        .await
    }

    pub async fn history_summaries_complex(
        &self,
        params: ConnectHistoryQueryParams,
    ) -> Vec<ConnectHistoryStatus> {
        self.run_query_default(task_label::op::METRIC_HISTORY_SUMMARIES, move |store| {
            let conn = store.get_disk_conn();
            connect_query::query_historical_summaries_complex(&conn, params)
        })
        .await
    }

    pub async fn history_src_ip_stats(
        &self,
        params: ConnectHistoryQueryParams,
    ) -> Vec<IpHistoryStat> {
        self.run_query_default(task_label::op::METRIC_HISTORY_SRC_IP, move |store| {
            let conn = store.get_disk_conn();
            connect_query::query_connection_ip_history(&conn, params, true)
        })
        .await
    }

    pub async fn history_dst_ip_stats(
        &self,
        params: ConnectHistoryQueryParams,
    ) -> Vec<IpHistoryStat> {
        self.run_query_default(task_label::op::METRIC_HISTORY_DST_IP, move |store| {
            let conn = store.get_disk_conn();
            connect_query::query_connection_ip_history(&conn, params, false)
        })
        .await
    }

    pub async fn query_dns_history(&self, params: DnsHistoryQueryParams) -> DnsHistoryResponse {
        self.run_query_default(task_label::op::METRIC_DNS_HISTORY, move |store| {
            let conn = store.get_disk_conn();
            dns_history::query_dns_history(&conn, params)
        })
        .await
    }

    pub async fn get_dns_summary(&self, params: DnsSummaryQueryParams) -> DnsSummaryResponse {
        self.run_query_default(task_label::op::METRIC_DNS_SUMMARY, move |store| {
            let conn = store.get_disk_conn();
            dns_summary::query_dns_summary(&conn, params)
        })
        .await
    }

    pub async fn get_dns_lightweight_summary(
        &self,
        params: DnsSummaryQueryParams,
    ) -> DnsLightweightSummaryResponse {
        self.run_query_default(task_label::op::METRIC_DNS_LIGHTWEIGHT_SUMMARY, move |store| {
            let conn = store.get_disk_conn();
            dns_summary::query_dns_lightweight_summary(&conn, params)
        })
        .await
    }
}
