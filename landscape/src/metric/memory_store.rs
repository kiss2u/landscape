use std::collections::{HashMap, VecDeque};
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use landscape_common::config::MetricRuntimeConfig;
use landscape_common::error::LdResult;
use landscape_common::event::{ConnectMessage, DnsMetricMessage};
use landscape_common::metric::connect::{
    ConnectGlobalStats, ConnectHistoryQueryParams, ConnectHistoryStatus, ConnectKey, ConnectMetric,
    ConnectMetricPoint, ConnectRealtimeStatus, ConnectStatusType, IpAggregatedStats, IpHistoryStat,
    IpRealtimeStat, MetricResolution,
};
use landscape_common::metric::dns::{
    DnsHistoryQueryParams, DnsHistoryResponse, DnsLightweightSummaryResponse,
    DnsSummaryQueryParams, DnsSummaryResponse,
};
use tokio::sync::mpsc;

const CHANNEL_CAPACITY: usize = 1024;
const MS_PER_MINUTE: u64 = 60 * 1000;
const STALE_TIMEOUT_MS: u64 = 5 * MS_PER_MINUTE;
const DEFAULT_CONNECT_SAMPLE_INTERVAL_MS: u64 = 5 * 1000;

fn second_window_ms(config: &MetricRuntimeConfig) -> u64 {
    config.connect_second_window_minutes.max(1).saturating_mul(MS_PER_MINUTE)
}

fn second_ring_capacity(config: &MetricRuntimeConfig) -> usize {
    let target_points = second_window_ms(config) / DEFAULT_CONNECT_SAMPLE_INTERVAL_MS;
    target_points.saturating_add(8).clamp(32, 4096) as usize
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
    finalized: bool,
}

impl FlowState {
    fn new(metric: ConnectMetric, window_ms: u64, ring_cap: usize) -> Self {
        let mut second_ring = VecDeque::with_capacity(ring_cap.max(1));
        second_ring.push_back(metric_to_point(&metric));

        let mut state = Self {
            realtime: metric_to_realtime(&metric),
            last_metric: metric,
            second_ring,
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

    fn mark_finalized(&mut self) {
        self.finalized = true;
        self.last_metric.status = ConnectStatusType::Disabled;
        self.realtime.status = ConnectStatusType::Disabled;
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

fn process_connect_metric(
    flow_cache: &FlowCache,
    metric: ConnectMetric,
    second_window_ms: u64,
    second_ring_cap: usize,
) {
    let mut cache = flow_cache.write().expect("memory metric flow cache poisoned");
    match cache.entry(metric.key.clone()) {
        std::collections::hash_map::Entry::Occupied(mut entry) => {
            let state = entry.get_mut();
            if metric.report_time < state.last_metric.report_time {
                return;
            }

            let should_finalize = metric.status == ConnectStatusType::Disabled;
            state.update_from_metric(metric, second_window_ms, second_ring_cap);
            if should_finalize {
                state.mark_finalized();
            }
        }
        std::collections::hash_map::Entry::Vacant(entry) => {
            let should_finalize = metric.status == ConnectStatusType::Disabled;
            let mut state = FlowState::new(metric, second_window_ms, second_ring_cap);
            if should_finalize {
                state.mark_finalized();
            }
            entry.insert(state);
        }
    }
}

fn cleanup_flow_cache(flow_cache: &FlowCache, now_ms: u64, second_window_ms: u64) {
    let stale_cutoff = now_ms.saturating_sub(STALE_TIMEOUT_MS);
    let window_cutoff = now_ms.saturating_sub(second_window_ms);

    let mut cache = flow_cache.write().expect("memory metric flow cache poisoned");
    let mut expired_keys = Vec::new();

    for (key, state) in cache.iter_mut() {
        if !state.finalized && state.realtime.last_report_time < stale_cutoff {
            state.mark_finalized();
        }

        state.trim_second_ring_before(window_cutoff);

        if state.finalized && state.realtime.last_report_time < window_cutoff {
            expired_keys.push(key.clone());
        }
    }

    for key in expired_keys {
        cache.remove(&key);
    }
}

#[derive(Clone)]
pub struct MemoryMetricStore {
    connect_tx: mpsc::Sender<ConnectMessage>,
    dns_tx: mpsc::Sender<DnsMetricMessage>,
    flow_cache: FlowCache,
    second_window_ms: u64,
}

impl MemoryMetricStore {
    pub async fn new(_base_path: PathBuf, config: MetricRuntimeConfig) -> Self {
        tracing::info!("metric-duckdb feature disabled, using realtime-only memory metric store");

        let (connect_tx, mut connect_rx) = mpsc::channel::<ConnectMessage>(CHANNEL_CAPACITY);
        let (dns_tx, mut dns_rx) = mpsc::channel::<DnsMetricMessage>(CHANNEL_CAPACITY);
        let flow_cache: FlowCache = Arc::new(RwLock::new(HashMap::new()));
        let second_window_ms = second_window_ms(&config);
        let second_ring_cap = second_ring_capacity(&config);
        let cleanup_interval = std::time::Duration::from_secs(config.cleanup_interval_secs.max(1));
        let cleanup_flow_cache_ref = flow_cache.clone();

        tokio::spawn(async move {
            let mut cleanup_tick = tokio::time::interval(cleanup_interval);
            let mut connect_closed = false;
            let mut dns_closed = false;

            loop {
                tokio::select! {
                    _ = cleanup_tick.tick() => {
                        let now_ms = landscape_common::utils::time::get_current_time_ms().unwrap_or_default();
                        cleanup_flow_cache(&cleanup_flow_cache_ref, now_ms, second_window_ms);
                    }
                    msg_opt = connect_rx.recv(), if !connect_closed => {
                        match msg_opt {
                            Some(ConnectMessage::Metric(metric)) => {
                                process_connect_metric(
                                    &cleanup_flow_cache_ref,
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
                            Some(DnsMetricMessage::Metric(_)) => {}
                            None => dns_closed = true,
                        }
                    }
                }

                if connect_closed && dns_closed {
                    break;
                }
            }
        });

        Self { connect_tx, dns_tx, flow_cache, second_window_ms }
    }

    pub fn get_connect_msg_channel(&self) -> mpsc::Sender<ConnectMessage> {
        self.connect_tx.clone()
    }

    pub fn get_dns_msg_channel(&self) -> mpsc::Sender<DnsMetricMessage> {
        self.dns_tx.clone()
    }

    pub async fn connect_infos(&self) -> Vec<ConnectRealtimeStatus> {
        let now_ms = landscape_common::utils::time::get_current_time_ms().unwrap_or_default();
        let cache = self.flow_cache.read().expect("memory metric flow cache poisoned");
        let mut infos: Vec<_> = cache
            .values()
            .filter(|state| state.is_active(now_ms))
            .map(|state| state.realtime.clone())
            .collect();
        infos.sort_by(|a, b| b.last_report_time.cmp(&a.last_report_time));
        infos
    }

    pub async fn get_realtime_ip_stats(&self, is_src: bool) -> Vec<IpRealtimeStat> {
        let now_ms = landscape_common::utils::time::get_current_time_ms().unwrap_or_default();
        let cache = self.flow_cache.read().expect("memory metric flow cache poisoned");
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

        stats_map.into_iter().map(|(ip, stats)| IpRealtimeStat { ip, stats }).collect()
    }

    pub async fn query_metric_by_key(
        &self,
        key: ConnectKey,
        resolution: MetricResolution,
    ) -> Vec<ConnectMetricPoint> {
        if resolution != MetricResolution::Second {
            return Vec::new();
        }

        let cutoff = landscape_common::utils::time::get_current_time_ms()
            .unwrap_or_default()
            .saturating_sub(self.second_window_ms);
        let cache = self.flow_cache.read().expect("memory metric flow cache poisoned");
        cache.get(&key).map(|state| state.second_points_since(cutoff)).unwrap_or_default()
    }

    pub async fn history_summaries_complex(
        &self,
        _params: ConnectHistoryQueryParams,
    ) -> Vec<ConnectHistoryStatus> {
        Vec::new()
    }

    pub async fn history_src_ip_stats(
        &self,
        _params: ConnectHistoryQueryParams,
    ) -> Vec<IpHistoryStat> {
        Vec::new()
    }

    pub async fn history_dst_ip_stats(
        &self,
        _params: ConnectHistoryQueryParams,
    ) -> Vec<IpHistoryStat> {
        Vec::new()
    }

    pub async fn get_global_stats(&self, _force_refresh: bool) -> LdResult<ConnectGlobalStats> {
        Ok(ConnectGlobalStats::default())
    }

    pub async fn query_dns_history(&self, _params: DnsHistoryQueryParams) -> DnsHistoryResponse {
        DnsHistoryResponse::default()
    }

    pub async fn get_dns_summary(&self, _params: DnsSummaryQueryParams) -> DnsSummaryResponse {
        DnsSummaryResponse::default()
    }

    pub async fn get_dns_lightweight_summary(
        &self,
        _params: DnsSummaryQueryParams,
    ) -> DnsLightweightSummaryResponse {
        DnsLightweightSummaryResponse::default()
    }
}
