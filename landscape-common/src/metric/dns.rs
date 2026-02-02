use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(export, export_to = "common/metric/dns.d.ts")]
pub struct DnsMetric {
    pub flow_id: u32,
    pub domain: String,
    pub query_type: String,
    pub response_code: String,
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
    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub domain: Option<String>,
    pub src_ip: Option<String>,
    pub sort_key: Option<DnsSortKey>,
    pub sort_order: Option<crate::metric::connect::SortOrder>,
}

#[derive(Debug, Clone, Deserialize, Serialize, TS)]
#[ts(export, export_to = "common/metric/dns.d.ts")]
pub struct DnsHistoryResponse {
    pub items: Vec<DnsMetric>,
    pub total: usize,
}

