use landscape_common::config::MetricRuntimeConfig;
use landscape_common::metric::connect::{
    ConnectKey, ConnectMetric, ConnectMetricPoint, ConnectRealtimeStatus, ConnectStatusType,
    IpAggregatedStats, IpRealtimeStat,
};
use std::collections::{HashMap, VecDeque};
use std::net::IpAddr;
use std::sync::{Arc, RwLock};

pub(crate) const CHANNEL_CAPACITY: usize = 1024;
pub(crate) const MS_PER_MINUTE: u64 = 60 * 1000;
pub(crate) const MS_PER_HOUR: u64 = 60 * MS_PER_MINUTE;
pub(crate) const MS_PER_DAY: u64 = 24 * MS_PER_HOUR;
const MS_PER_TEN_MINUTES: u64 = 10 * MS_PER_MINUTE;
const STALE_TIMEOUT_MS: u64 = 5 * MS_PER_MINUTE;
const DEFAULT_CONNECT_SAMPLE_INTERVAL_MS: u64 = 5 * 1000;

pub(crate) fn bucket_start(report_time: u64, bucket_ms: u64) -> u64 {
    report_time / bucket_ms * bucket_ms
}

pub(crate) fn minute_slot(report_time: u64) -> u64 {
    bucket_start(report_time, MS_PER_MINUTE)
}

pub(crate) fn hour_refresh_slot(report_time: u64) -> u64 {
    bucket_start(report_time, MS_PER_TEN_MINUTES)
}

pub(crate) fn day_refresh_slot(report_time: u64) -> u64 {
    bucket_start(report_time, MS_PER_HOUR)
}

pub(crate) fn second_window_ms(config: &MetricRuntimeConfig) -> u64 {
    config.connect_second_window_minutes.max(1).saturating_mul(MS_PER_MINUTE)
}

pub(crate) fn second_ring_capacity(config: &MetricRuntimeConfig) -> usize {
    let target_points = second_window_ms(config) / DEFAULT_CONNECT_SAMPLE_INTERVAL_MS;
    target_points.saturating_add(8).clamp(32, 4096) as usize
}

pub(crate) fn clean_ip_string(ip: &IpAddr) -> String {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BucketKind {
    Minute,
    Hour,
    Day,
}

impl BucketKind {
    pub(crate) fn table_name(self) -> &'static str {
        match self {
            Self::Minute => "conn_metrics_1m",
            Self::Hour => "conn_metrics_1h",
            Self::Day => "conn_metrics_1d",
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct BucketWrite {
    pub kind: BucketKind,
    pub metric: ConnectMetric,
    pub bucket_report_time: u64,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct PersistenceBatch {
    pub summary_metrics: Vec<ConnectMetric>,
    pub bucket_writes: Vec<BucketWrite>,
}

impl PersistenceBatch {
    pub(crate) fn is_empty(&self) -> bool {
        self.summary_metrics.is_empty() && self.bucket_writes.is_empty()
    }

    pub(crate) fn op_count(&self) -> usize {
        self.summary_metrics.len() + self.bucket_writes.len()
    }

    pub(crate) fn extend(&mut self, other: Self) {
        self.summary_metrics.extend(other.summary_metrics);
        self.bucket_writes.extend(other.bucket_writes);
    }

    fn push_summary(&mut self, metric: ConnectMetric) {
        self.summary_metrics.push(metric);
    }

    fn push_bucket(&mut self, kind: BucketKind, metric: ConnectMetric, bucket_report_time: u64) {
        self.bucket_writes.push(BucketWrite { kind, metric, bucket_report_time });
    }
}

#[derive(Clone)]
pub(crate) struct FlowState {
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

pub(crate) type FlowCache = Arc<RwLock<HashMap<ConnectKey, FlowState>>>;

fn finalize_state_batch(state: &mut FlowState, mark_disabled: bool, batch: &mut PersistenceBatch) {
    if state.finalized {
        return;
    }

    let mut metric = state.last_metric.clone();
    if mark_disabled {
        metric.status = ConnectStatusType::Disabled;
        state.last_metric.status = ConnectStatusType::Disabled;
        state.realtime.status = ConnectStatusType::Disabled;
    }

    batch.push_bucket(BucketKind::Minute, metric.clone(), minute_slot(metric.report_time));
    batch.push_bucket(
        BucketKind::Hour,
        metric.clone(),
        bucket_start(metric.report_time, MS_PER_HOUR),
    );
    batch.push_bucket(
        BucketKind::Day,
        metric.clone(),
        bucket_start(metric.report_time, MS_PER_DAY),
    );
    batch.push_summary(metric);
    state.finalized = true;
}

pub(crate) fn process_connect_metric(
    flow_cache: &FlowCache,
    metric: ConnectMetric,
    second_window_ms: u64,
    second_ring_cap: usize,
) -> PersistenceBatch {
    let curr_minute_slot = minute_slot(metric.report_time);
    let curr_hour_refresh_slot = hour_refresh_slot(metric.report_time);
    let curr_day_refresh_slot = day_refresh_slot(metric.report_time);

    let mut batch = PersistenceBatch::default();
    let mut cache = flow_cache.write().expect("metric flow cache poisoned");
    match cache.entry(metric.key.clone()) {
        std::collections::hash_map::Entry::Occupied(mut entry) => {
            let state = entry.get_mut();
            if metric.report_time < state.last_metric.report_time {
                return batch;
            }

            let previous_minute_bucket = minute_slot(state.last_metric.report_time);
            let previous_hour_bucket = bucket_start(state.last_metric.report_time, MS_PER_HOUR);
            let previous_day_bucket = bucket_start(state.last_metric.report_time, MS_PER_DAY);

            if curr_minute_slot > state.last_minute_slot {
                batch.push_bucket(BucketKind::Minute, metric.clone(), previous_minute_bucket);
                batch.push_summary(metric.clone());
                state.last_minute_slot = curr_minute_slot;
            }
            if curr_hour_refresh_slot > state.last_hour_refresh_slot {
                batch.push_bucket(BucketKind::Hour, metric.clone(), previous_hour_bucket);
                state.last_hour_refresh_slot = curr_hour_refresh_slot;
            }
            if curr_day_refresh_slot > state.last_day_refresh_slot {
                batch.push_bucket(BucketKind::Day, metric.clone(), previous_day_bucket);
                state.last_day_refresh_slot = curr_day_refresh_slot;
            }

            let should_finalize = metric.status == ConnectStatusType::Disabled;
            state.update_from_metric(metric, second_window_ms, second_ring_cap);
            if should_finalize {
                finalize_state_batch(state, true, &mut batch);
            }
        }
        std::collections::hash_map::Entry::Vacant(entry) => {
            let should_finalize = metric.status == ConnectStatusType::Disabled;
            let mut state = FlowState::new(metric, second_window_ms, second_ring_cap);
            if should_finalize {
                finalize_state_batch(&mut state, true, &mut batch);
            }
            entry.insert(state);
        }
    }

    batch
}

#[derive(Default)]
pub(crate) struct FlowCacheStats {
    pub active_flows: usize,
    pub finalized_flows: usize,
    pub finalized_in_run: usize,
    pub second_ring_points: usize,
}

pub(crate) fn cleanup_flow_cache(
    flow_cache: &FlowCache,
    now_ms: u64,
    second_window_ms: u64,
) -> (FlowCacheStats, PersistenceBatch) {
    let stale_cutoff = now_ms.saturating_sub(STALE_TIMEOUT_MS);
    let window_cutoff = now_ms.saturating_sub(second_window_ms);

    let mut cache = flow_cache.write().expect("metric flow cache poisoned");
    let mut expired_keys = Vec::new();
    let mut stats = FlowCacheStats::default();
    let mut batch = PersistenceBatch::default();

    for (key, state) in cache.iter_mut() {
        if !state.finalized && state.realtime.last_report_time < stale_cutoff {
            finalize_state_batch(state, true, &mut batch);
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

    (stats, batch)
}

pub(crate) fn finalize_all_flows(flow_cache: &FlowCache) -> PersistenceBatch {
    let mut cache = flow_cache.write().expect("metric flow cache poisoned");
    let mut batch = PersistenceBatch::default();
    for state in cache.values_mut() {
        finalize_state_batch(state, true, &mut batch);
    }
    batch
}

pub(crate) fn collect_connect_infos(
    flow_cache: &FlowCache,
    now_ms: u64,
) -> Vec<ConnectRealtimeStatus> {
    let cache = flow_cache.read().expect("metric flow cache poisoned");
    let mut infos: Vec<_> = cache
        .values()
        .filter(|state| state.is_active(now_ms))
        .map(|state| state.realtime.clone())
        .collect();
    infos.sort_by(|a, b| b.last_report_time.cmp(&a.last_report_time));
    infos
}

pub(crate) fn collect_realtime_ip_stats(
    flow_cache: &FlowCache,
    now_ms: u64,
    is_src: bool,
) -> Vec<IpRealtimeStat> {
    let cache = flow_cache.read().expect("metric flow cache poisoned");
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

pub(crate) fn second_points_by_key(
    flow_cache: &FlowCache,
    key: &ConnectKey,
    cutoff: u64,
) -> Vec<ConnectMetricPoint> {
    let cache = flow_cache.read().expect("metric flow cache poisoned");
    cache.get(key).map(|state| state.second_points_since(cutoff)).unwrap_or_default()
}
