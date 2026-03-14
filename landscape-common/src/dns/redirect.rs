use landscape_macro::LdApiError;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use uuid::Uuid;

use crate::config::ConfigId;

#[derive(thiserror::Error, Debug, LdApiError)]
#[api_error(crate_path = "crate")]
pub enum DnsRedirectError {
    #[error("DNS redirect rule '{0}' not found")]
    #[api_error(id = "dns_redirect.not_found", status = 404)]
    NotFound(ConfigId),
}

use crate::utils::id::gen_database_uuid;
use crate::utils::time::get_f64_timestamp;
use crate::{
    config::FlowId,
    database::repository::LandscapeDBStore,
    dns::rule::{DomainConfig, DomainMatchType, RuleSource},
};

pub const DEFAULT_STATIC_DNS_REDIRECT_TTL_SECS: u32 = 10;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum DnsRedirectAnswerMode {
    #[default]
    StaticIps,
    AllLocalIps,
}

impl DnsRedirectAnswerMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::StaticIps => "static_ips",
            Self::AllLocalIps => "all_local_ips",
        }
    }

    pub fn from_db_value(value: &str) -> Self {
        match value {
            "all_local_ips" => Self::AllLocalIps,
            _ => Self::StaticIps,
        }
    }
}

/// 用于定义 DNS 重定向的单元配置
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DNSRedirectRule {
    #[serde(default = "gen_database_uuid")]
    #[cfg_attr(feature = "openapi", schema(required = false))]
    pub id: Uuid,

    pub remark: String,

    pub enable: bool,

    pub match_rules: Vec<RuleSource>,

    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(required = true))]
    pub answer_mode: DnsRedirectAnswerMode,

    #[cfg_attr(feature = "openapi", schema(value_type = Vec<String>))]
    pub result_info: Vec<IpAddr>,

    pub apply_flows: Vec<FlowId>,

    #[serde(default = "get_f64_timestamp")]
    #[cfg_attr(feature = "openapi", schema(required = false))]
    pub update_at: f64,
}

impl LandscapeDBStore<Uuid> for DNSRedirectRule {
    fn get_id(&self) -> Uuid {
        self.id
    }
    fn get_update_at(&self) -> f64 {
        self.update_at
    }
    fn set_update_at(&mut self, ts: f64) {
        self.update_at = ts;
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum DynamicDnsRedirectScope {
    Global,
    Flow(FlowId),
}

impl DynamicDnsRedirectScope {
    pub fn applies_to_flow(&self, flow_id: FlowId) -> bool {
        match self {
            Self::Global => true,
            Self::Flow(scope_flow_id) => *scope_flow_id == flow_id,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(tag = "t")]
#[serde(rename_all = "snake_case")]
pub enum DynamicDnsMatch {
    Full(String),
    Domain(String),
}

impl From<DynamicDnsMatch> for DomainConfig {
    fn from(value: DynamicDnsMatch) -> Self {
        match value {
            DynamicDnsMatch::Full(value) => {
                DomainConfig { match_type: DomainMatchType::Full, value }
            }
            DynamicDnsMatch::Domain(value) => {
                DomainConfig { match_type: DomainMatchType::Domain, value }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DynamicDnsRedirectRecord {
    pub match_rule: DynamicDnsMatch,
    #[cfg_attr(feature = "openapi", schema(value_type = Vec<String>))]
    pub result_info: Vec<IpAddr>,
    pub ttl_secs: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DynamicDnsRedirectBatch {
    pub source_id: String,
    pub scope: DynamicDnsRedirectScope,
    pub records: Vec<DynamicDnsRedirectRecord>,
}

#[derive(Default, Debug)]
pub struct DNSRedirectRuntimeRule {
    pub redirect_id: Option<Uuid>,
    pub dynamic_redirect_source: Option<String>,
    pub answer_mode: DnsRedirectAnswerMode,
    pub match_rules: Vec<DomainConfig>,
    pub result_info: Vec<IpAddr>,
    pub ttl_secs: u32,
}
