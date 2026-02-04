use crate::config::dns::{default_flow_id, DNSRuleConfig, DNSRuntimeRule, FilterResult};
use crate::dns::config::{DnsBindConfig, DnsUpstreamConfig};
use crate::dns::redirect::DNSRedirectRuntimeRule;
use crate::utils::id::gen_database_uuid;
use crate::utils::time::get_f64_timestamp;

pub mod check;
pub mod config;
pub mod redirect;
pub mod upstream;

#[derive(Default, Debug)]
pub struct ChainDnsServerInitInfo {
    pub dns_rules: Vec<DNSRuntimeRule>,
    pub redirect_rules: Vec<DNSRedirectRuntimeRule>,
}

pub fn gen_default_dns_rule_and_upstream() -> (DNSRuleConfig, DnsUpstreamConfig) {
    let upstream = DnsUpstreamConfig::default();
    let rule = DNSRuleConfig {
        id: gen_database_uuid(),
        name: "Landscape Router default rule".into(),
        index: 10000,
        enable: true,
        filter: FilterResult::default(),
        mark: Default::default(),
        source: vec![],
        flow_id: default_flow_id(),
        update_at: get_f64_timestamp(),
        upstream_id: upstream.id,
        bind_config: DnsBindConfig::default(),
    };
    (rule, upstream)
}
