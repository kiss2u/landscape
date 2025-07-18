use hickory_proto::{
    op::ResponseCode,
    rr::{
        rdata::{A, AAAA},
        RData, Record, RecordType,
    },
};
use hickory_resolver::{
    config::{NameServerConfigGroup, ResolveHosts, ResolverConfig, ResolverOpts},
    Resolver,
};
use landscape_common::{
    config::dns::{
        DNSResolveMode, DNSRuntimeRule, DnsUpstreamType, DomainConfig, DomainMatchType,
        FilterResult,
    },
    flow::{mark::FlowDnsMark, DnsRuntimeMarkInfo},
};
use matcher::DomainMatcher;
use std::net::IpAddr;
use std::str::FromStr;

use crate::connection::{MarkConnectionProvider, MarkRuntimeProvider};

mod matcher;

#[derive(Debug)]
pub struct CacheResolver {
    pub flow_id: u32,
    pub resolver: Resolver<MarkConnectionProvider>,
}

impl CacheResolver {
    pub fn new(resolve: ResolverConfig, mark: &FlowDnsMark, flow_id: u32) -> Self {
        let mark_value = match mark.clone() {
            // 转发时候使用目标 flow 进行标记 DNS 请求
            FlowDnsMark::Redirect { flow_id } => flow_id as u32,
            // 忽略流的配置
            FlowDnsMark::Direct => 0,
            // 其余情况使用 当前规则所属的 flow 进行标记
            _ => flow_id,
        };

        let mut options = ResolverOpts::default();
        options.cache_size = 0;
        options.preserve_intermediates = false;
        options.use_hosts_file = ResolveHosts::Never;
        let resolver = Resolver::builder_with_config(
            resolve,
            MarkConnectionProvider::new(MarkRuntimeProvider::new(mark_value)),
        )
        .with_options(options)
        .build();
        CacheResolver { resolver, flow_id: mark_value }
    }
}

#[derive(Debug)]
pub enum ResolverType {
    RedirectResolver(Vec<IpAddr>),
    CacheResolver(CacheResolver),
}
impl ResolverType {
    pub fn new(config: &DNSRuntimeRule, flow_id: u32) -> Self {
        match &config.resolve_mode {
            DNSResolveMode::Redirect { ips } => ResolverType::RedirectResolver(ips.clone()),
            DNSResolveMode::Upstream { upstream, ips, port } => {
                let name_server = match upstream {
                    DnsUpstreamType::Plaintext => {
                        NameServerConfigGroup::from_ips_clear(ips, port.unwrap_or(53), true)
                    }
                    DnsUpstreamType::Tls { domain } => NameServerConfigGroup::from_ips_tls(
                        ips,
                        port.unwrap_or(843),
                        domain.to_string(),
                        true,
                    ),
                    DnsUpstreamType::Https { domain } => NameServerConfigGroup::from_ips_https(
                        ips,
                        port.unwrap_or(443),
                        domain.to_string(),
                        true,
                    ),
                };

                let resolve = ResolverConfig::from_parts(None, vec![], name_server);

                ResolverType::CacheResolver(CacheResolver::new(resolve, &config.mark, flow_id))
            }
            DNSResolveMode::Cloudflare { mode } => {
                let server = match mode {
                    landscape_common::config::dns::CloudflareMode::Plaintext => {
                        NameServerConfigGroup::cloudflare()
                    }
                    landscape_common::config::dns::CloudflareMode::Tls => {
                        NameServerConfigGroup::cloudflare_tls()
                    }
                    landscape_common::config::dns::CloudflareMode::Https => {
                        NameServerConfigGroup::cloudflare_https()
                    }
                };
                let resolve = ResolverConfig::from_parts(None, vec![], server);
                ResolverType::CacheResolver(CacheResolver::new(resolve, &config.mark, flow_id))
            }
        }
    }

    pub async fn lookup(
        &self,
        domain: &str,
        query_type: RecordType,
    ) -> Result<Vec<Record>, ResponseCode> {
        match self {
            ResolverType::RedirectResolver(result_ip) => {
                let mut result = vec![];
                for ip in result_ip {
                    let rdata_ip = match (ip, &query_type) {
                        (IpAddr::V4(ip), RecordType::A) => Some(RData::A(A(*ip))),
                        (IpAddr::V6(ip), RecordType::AAAA) => Some(RData::AAAA(AAAA(*ip))),
                        _ => None,
                    };

                    if let Some(rdata) = rdata_ip {
                        result.push(Record::from_rdata(
                            hickory_resolver::Name::from_str(domain).unwrap(),
                            300,
                            rdata,
                        ));
                    }
                }

                Ok(result)
            }
            ResolverType::CacheResolver(resolver) => {
                match resolver.resolver.lookup(domain, query_type).await {
                    Ok(lookup) => Ok(lookup.records().to_vec()),
                    Err(e) => {
                        let result = if e.is_no_records_found() {
                            ResponseCode::NoError
                        } else {
                            tracing::error!(
                                "[flow_id: {:?}] DNS resolution failed for {}: {}",
                                resolver.flow_id,
                                domain,
                                e
                            );
                            ResponseCode::ServFail
                        };
                        Err(result)
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
/// 与规则是 1:1 创建的
pub struct ResolutionRule {
    // 启动之后配置的 matcher
    matcher: DomainMatcher,
    //
    config: DNSRuntimeRule,

    mark: DnsRuntimeMarkInfo,

    resolver: ResolverType,
}

impl ResolutionRule {
    pub fn new(config: DNSRuntimeRule, flow_id: u32) -> Self {
        let span = tracing::info_span!("dns_rule", flow_id = flow_id);
        let _ = span.enter();

        let matcher = DomainMatcher::new(config.source.clone());

        let resolver = ResolverType::new(&config, flow_id);

        let mark = DnsRuntimeMarkInfo {
            mark: config.mark.clone(),
            priority: config.index as u16,
        };
        ResolutionRule { matcher, config, resolver, mark }
    }

    pub fn mark(&self) -> &DnsRuntimeMarkInfo {
        &self.mark
    }

    pub fn filter_mode(&self) -> FilterResult {
        self.config.filter.clone()
    }

    pub fn get_runtime_config(&self) -> DNSRuntimeRule {
        self.config.clone()
    }

    /// 确定是不是当前规则进行处理
    pub fn is_match(&self, domain: &str) -> bool {
        let match_result = if self.config.source.is_empty() {
            true
        } else {
            let domain =
                if let Some(stripped) = domain.strip_suffix('.') { stripped } else { domain };
            self.matcher.is_match(domain)
        };
        match_result
    }

    pub async fn lookup(
        &self,
        domain: &str,
        query_type: RecordType,
    ) -> Result<Vec<Record>, ResponseCode> {
        // TODO: do fiter in here
        self.resolver.lookup(domain, query_type).await
    }
}

// pub fn convert_config_to_runtime_rule(
//     config: &DNSRuleConfig,
//     geo_file: &HashMap<String, Vec<DomainConfig>>,
// ) -> Vec<DomainConfig> {
//     let mut all_domain_rules = vec![];
//     for each in config.source.iter() {
//         match each {
//             RuleSource::GeoKey(config_key) => {
//                 if let Some(domains) = geo_file.get(&key.to_uppercase()) {
//                     all_domain_rules.extend(domains.iter().cloned());
//                 }
//             }
//             RuleSource::Config(c) => {
//                 // all_domain_rules.extend(vec.iter().cloned());
//                 all_domain_rules.push(c.clone());
//             }
//         }
//     }
//     all_domain_rules
// }
