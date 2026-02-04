use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, TS, PartialEq, Eq, Default)]
#[ts(export, export_to = "common/metric/dns.d.ts")]
#[serde(rename_all = "lowercase")]
pub enum DnsResultStatus {
    Local,    // 重定向有值
    Block,    // 重定向空值
    Hit,      // 命中缓存
    NxDomain, // 域名不存在
    Filter,   // 被过滤 (OnlyIPv4/OnlyIPv6)
    #[default]
    Normal, // 正常透传
    Error,    // 异常
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(export, export_to = "common/metric/dns.d.ts")]
pub struct DnsMetric {
    pub flow_id: u32,
    pub domain: String,
    pub query_type: String,
    pub response_code: String,
    pub status: DnsResultStatus,
    #[ts(type = "number")]
    pub report_time: u64,
    pub duration_ms: u32,
    pub src_ip: IpAddr,
    pub answers: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, TS)]
#[ts(export, export_to = "common/metric/dns.d.ts")]
#[serde(rename_all = "lowercase")]
pub enum DnsSortKey {
    #[default]
    Time,
    Domain,
    Duration,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, TS)]
#[ts(export, export_to = "common/metric/dns.d.ts")]
pub struct DnsHistoryQueryParams {
    #[ts(optional)]
    pub start_time: Option<u64>,
    #[ts(optional)]
    pub end_time: Option<u64>,
    #[ts(optional)]
    pub limit: Option<usize>,
    #[ts(optional)]
    pub offset: Option<usize>,
    #[ts(optional)]
    pub domain: Option<String>,
    #[ts(optional)]
    pub src_ip: Option<String>,
    #[ts(optional)]
    pub query_type: Option<String>,
    #[ts(optional)]
    pub status: Option<DnsResultStatus>,
    #[ts(optional)]
    pub min_duration_ms: Option<u32>,
    #[ts(optional)]
    pub max_duration_ms: Option<u32>,
    #[ts(optional)]
    pub sort_key: Option<DnsSortKey>,
    #[ts(optional)]
    pub sort_order: Option<crate::metric::connect::SortOrder>,
}

#[derive(Debug, Clone, Deserialize, Serialize, TS)]
#[ts(export, export_to = "common/metric/dns.d.ts")]
pub struct DnsHistoryResponse {
    pub items: Vec<DnsMetric>,
    pub total: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize, TS)]
#[ts(export, export_to = "common/metric/dns.d.ts")]
pub struct DnsSummaryResponse {
    pub total_queries: usize,
    pub total_effective_queries: usize,
    pub cache_hit_count: usize,
    pub hit_count_v4: usize,
    pub hit_count_v6: usize,
    pub hit_count_other: usize,
    pub total_v4: usize,
    pub total_v6: usize,
    pub total_other: usize,
    pub block_count: usize,
    pub filter_count: usize,
    pub nxdomain_count: usize,
    pub error_count: usize,
    pub avg_duration_ms: f64,
    pub p50_duration_ms: f64,
    pub p95_duration_ms: f64,
    pub p99_duration_ms: f64,
    pub max_duration_ms: f64,
    pub top_clients: Vec<DnsStatEntry>,
    pub top_domains: Vec<DnsStatEntry>,
    pub top_blocked: Vec<DnsStatEntry>,
    pub slowest_domains: Vec<DnsStatEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize, TS)]
#[ts(export, export_to = "common/metric/dns.d.ts")]
pub struct DnsStatEntry {
    pub name: String,
    pub count: usize,
    #[ts(optional)]
    pub value: Option<f64>,
}
