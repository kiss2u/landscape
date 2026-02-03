use std::{net::IpAddr, str::FromStr as _};

use hickory_proto::op::ResponseCode;
use hickory_proto::rr::{
    rdata::{A, AAAA},
    RData, Record, RecordType,
};
use uuid::Uuid;

use landscape_common::dns::redirect::DNSRedirectRuntimeRule;
use landscape_common::{
    config::dns::{DNSRuntimeRule, FilterResult},
    flow::DnsRuntimeMarkInfo,
};

use crate::connection::LandscapeMarkDNSResolver;
use crate::server::matcher::DomainMatcher;
use crate::DEFAULT_ENABLE_IP_VALIDATION;

#[derive(Debug)]
pub struct RedirectSolution {
    pub id: Uuid,
    matcher: DomainMatcher,
    result_info: Vec<IpAddr>,
}

impl RedirectSolution {
    pub fn new(rule: DNSRedirectRuntimeRule) -> Self {
        let matcher = DomainMatcher::new(rule.match_rules);
        Self {
            matcher,
            id: rule.id,
            result_info: rule.result_info,
        }
    }

    pub fn is_match(&self, domain: &str) -> bool {
        let domain = if let Some(stripped) = domain.strip_suffix('.') { stripped } else { domain };
        self.matcher.is_match(domain)
    }

    pub fn lookup(&self, domain: &str, query_type: RecordType) -> Vec<Record> {
        let mut result = vec![];
        for ip in &self.result_info {
            let rdata_ip = match (ip, &query_type) {
                (IpAddr::V4(ip), RecordType::A) => Some(RData::A(A(*ip))),
                (IpAddr::V6(ip), RecordType::AAAA) => Some(RData::AAAA(AAAA(*ip))),
                _ => None,
            };

            if let Some(rdata) = rdata_ip {
                result.push(Record::from_rdata(
                    hickory_resolver::Name::from_str(domain).unwrap(),
                    10,
                    rdata,
                ));
            }
        }

        result
    }

    pub fn is_block(&self) -> bool {
        self.result_info.is_empty()
    }
}

#[derive(Debug)]
pub struct ResolutionRule {
    matcher: DomainMatcher,
    config: DNSRuntimeRule,
    mark: DnsRuntimeMarkInfo,
    resolver: LandscapeMarkDNSResolver,

    enable_ip_validation: bool,
}

impl ResolutionRule {
    pub fn new(config: DNSRuntimeRule, flow_id: u32) -> Self {
        let span = tracing::info_span!("dns_rule", flow_id = flow_id);
        let _ = span.enter();

        let matcher = DomainMatcher::new(config.source.clone());

        let enable_ip_validation =
            config.resolve_mode.enable_ip_validation.unwrap_or(DEFAULT_ENABLE_IP_VALIDATION);
        let resolver = crate::connection::create_resolver(
            flow_id,
            config.mark,
            config.bind_config.clone(),
            config.resolve_mode.clone(),
        );

        let mark = DnsRuntimeMarkInfo {
            mark: config.mark.clone(),
            priority: config.index as u16,
        };
        ResolutionRule {
            matcher,
            config,
            resolver,
            mark,
            enable_ip_validation,
        }
    }

    pub fn mark(&self) -> &DnsRuntimeMarkInfo {
        &self.mark
    }

    pub fn filter_mode(&self) -> FilterResult {
        self.config.filter.clone()
    }

    pub fn get_config_id(&self) -> Uuid {
        self.config.id
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
        match self.resolver.lookup(domain, query_type).await {
            Ok(lookup) => {
                let result = if self.enable_ip_validation {
                    lookup
                        .record_iter()
                        .filter(|ietm| match ietm.data() {
                            RData::A(A(ipv4)) => is_global_ipv4(ipv4),
                            RData::AAAA(AAAA(ipv6)) => is_global_ipv6(ipv6),
                            _ => true,
                        })
                        .cloned()
                        .collect()
                } else {
                    lookup.records().to_vec()
                };
                Ok(result)
            }
            Err(e) => {
                let code = if let Some(proto_err) = e.proto() {
                    match proto_err.kind() {
                        hickory_proto::ProtoErrorKind::NoRecordsFound { response_code, .. } => {
                            *response_code
                        }
                        _ => {
                            tracing::error!(
                                "[flow_id: {:?}, config: {}] DNS resolution failed (proto) for {}: {}",
                                self.config.flow_id,
                                self.config.resolve_mode.id,
                                domain,
                                e
                            );
                            ResponseCode::ServFail
                        }
                    }
                } else {
                    tracing::error!(
                        "[flow_id: {:?}, config: {}] DNS resolution failed (resolver) for {}: {}",
                        self.config.flow_id,
                        self.config.resolve_mode.id,
                        domain,
                        e
                    );
                    ResponseCode::ServFail
                };
                Err(code)
            }
        }
    }
}

// Copy from unstable feature
fn is_global_ipv4(addr: &std::net::Ipv4Addr) -> bool {
    !(addr.octets()[0] == 0
        || addr.is_private()
        || addr.is_loopback()
        || addr.is_link_local()
        || (addr.octets()[0] == 192
            && addr.octets()[1] == 0
            && addr.octets()[2] == 0
            && addr.octets()[3] != 9
            && addr.octets()[3] != 10)
        || addr.is_documentation()
        || addr.is_broadcast())
}

// Copy from unstable feature
fn is_global_ipv6(addr: &std::net::Ipv6Addr) -> bool {
    !(addr.is_unspecified()
            || addr.is_loopback()
            // IPv4-mapped Address (`::ffff:0:0/96`)
            || matches!(addr.segments(), [0, 0, 0, 0, 0, 0xffff, _, _])
            // IPv4-IPv6 Translat. (`64:ff9b:1::/48`)
            || matches!(addr.segments(), [0x64, 0xff9b, 1, _, _, _, _, _])
            // Discard-Only Address Block (`100::/64`)
            || matches!(addr.segments(), [0x100, 0, 0, 0, _, _, _, _])
            // IETF Protocol Assignments (`2001::/23`)
            || (matches!(addr.segments(), [0x2001, b, _, _, _, _, _, _] if b < 0x200)
                && !(
                    // Port Control Protocol Anycast (`2001:1::1`)
                    u128::from_be_bytes(addr.octets()) == 0x2001_0001_0000_0000_0000_0000_0000_0001
                    // Traversal Using Relays around NAT Anycast (`2001:1::2`)
                    || u128::from_be_bytes(addr.octets()) == 0x2001_0001_0000_0000_0000_0000_0000_0002
                    // AMT (`2001:3::/32`)
                    || matches!(addr.segments(), [0x2001, 3, _, _, _, _, _, _])
                    // AS112-v6 (`2001:4:112::/48`)
                    || matches!(addr.segments(), [0x2001, 4, 0x112, _, _, _, _, _])
                    // ORCHIDv2 (`2001:20::/28`)
                    // Drone Remote ID Protocol Entity Tags (DETs) Prefix (`2001:30::/28`)`
                    || matches!(addr.segments(), [0x2001, b, _, _, _, _, _, _] if b >= 0x20 && b <= 0x3F)
                ))
            // 6to4 (`2002::/16`) – it's not explicitly documented as globally reachable,
            // IANA says N/A.
            || matches!(addr.segments(), [0x2002, _, _, _, _, _, _, _])
            // Segment Routing (SRv6) SIDs (`5f00::/16`)
            || matches!(addr.segments(), [0x5f00, ..])
            || addr.is_unique_local()
            || addr.is_unicast_link_local())
}
