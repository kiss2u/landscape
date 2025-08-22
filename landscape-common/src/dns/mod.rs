use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

use crate::config::dns::{CloudflareMode, DnsUpstreamType, DomainConfig, FilterResult, RuleSource};
use crate::flow::mark::FlowMark;
use crate::flow::DnsRuntimeMarkInfo;

#[derive(Debug)]
pub struct RuleHandlerInfo {
    pub rule_id: Option<Uuid>,
    pub flow_id: u32,
    pub resolver_id: Uuid,
    pub mark: DnsRuntimeMarkInfo,
    pub filter: FilterResult,
}

#[derive(Debug)]
pub struct RedirectInfo {
    pub result_ip: Vec<IpAddr>,
}

impl RuleHandlerInfo {
    pub fn mark(&self) -> &DnsRuntimeMarkInfo {
        &self.mark
    }

    pub fn filter_mode(&self) -> FilterResult {
        self.filter.clone()
    }
}

pub struct DnsServerInitInfo {
    pub rules: HashMap<DomainConfig, Arc<RuleHandlerInfo>>,
    pub redirect_rules: HashMap<DomainConfig, Arc<RedirectInfo>>,
    pub resolver_configs: Vec<DnsResolverConfig>,
}

pub struct DnsResolverConfig {
    pub id: Uuid,
    pub resolve_mode: DnsUpstreamMode,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DNSRuleInitInfo {
    pub id: Option<Uuid>,
    pub name: String,
    /// 优先级
    pub index: u32,
    /// 是否启用
    pub enable: bool,
    /// 过滤模式
    pub filter: FilterResult,
    /// 解析模式
    pub resolve_mode: DnsUpstreamMode,
    /// 流量标记
    pub mark: FlowMark,
    /// 匹配规则列表
    pub source: Vec<DomainConfig>,
    /// 匹配规则列表 key
    pub source_key: Vec<RuleSource>,

    pub flow_id: u32,
}

impl Default for DNSRuleInitInfo {
    fn default() -> Self {
        Self {
            id: Default::default(),
            name: Default::default(),
            index: 1,
            enable: true,
            filter: Default::default(),
            resolve_mode: Default::default(),
            mark: Default::default(),
            source: Default::default(),
            source_key: Default::default(),
            flow_id: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export, export_to = "common/dns.d.ts")]
#[serde(tag = "t")]
#[serde(rename_all = "snake_case")]
pub enum DnsUpstreamMode {
    Upstream { upstream: DnsUpstreamType, ips: Vec<IpAddr>, port: Option<u16> },
    Cloudflare { mode: CloudflareMode },
}

impl Default for DnsUpstreamMode {
    fn default() -> Self {
        DnsUpstreamMode::Cloudflare { mode: CloudflareMode::Tls }
    }
}
