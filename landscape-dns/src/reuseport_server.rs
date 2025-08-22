use std::{
    collections::HashMap,
    net::{IpAddr, Ipv6Addr, SocketAddr, SocketAddrV6},
    str::FromStr,
    sync::Arc,
};

use hickory_proto::{
    op::ResponseCode,
    rr::{
        rdata::{A, AAAA},
        RData, Record, RecordType,
    },
};
use hickory_resolver::{
    config::{NameServerConfigGroup, ResolverConfig, ResolverOpts},
    Resolver,
};
use hickory_server::ServerFuture;
use landscape_common::{
    config::dns::{DnsUpstreamType, DomainConfig},
    dns::{DNSRuleInitInfo, DnsUpstreamMode, RedirectInfo, RuleHandlerInfo},
    flow::DnsRuntimeMarkInfo,
    service::DefaultWatchServiceStatus,
};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::{
    connection::{MarkConnectionProvider, MarkRuntimeProvider},
    reuseport_server::{matcher::DomainMatcher, request::LandscapeDnsRequestHandle},
};

mod listener;
mod matcher;
mod request;

#[derive(Clone)]
pub struct LandscapeReusePortDnsServer {
    pub status: DefaultWatchServiceStatus,
    flow_dns_server: Arc<Mutex<HashMap<u32, (LandscapeDnsRequestHandle, CancellationToken)>>>,
    pub addr: SocketAddr,
}

impl LandscapeReusePortDnsServer {
    pub fn new(listen_port: u16) -> Self {
        crate::check_resolver_conf();
        let status = DefaultWatchServiceStatus::new();
        Self {
            status,
            flow_dns_server: Arc::new(Mutex::new(HashMap::new())),
            addr: SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, listen_port, 0, 0)),
        }
    }

    pub fn get_status(&self) -> &DefaultWatchServiceStatus {
        &self.status
    }

    pub async fn init_server(&self, infos: Vec<DNSRuleInitInfo>) {
        let dns_rules: Vec<DNSRuleInitInfo> =
            infos.into_iter().filter(|rule| rule.enable).collect();

        let mut groups: HashMap<u32, Vec<DNSRuleInitInfo>> = HashMap::new();

        for rule in dns_rules.into_iter() {
            groups.entry(rule.flow_id).or_default().push(rule);
        }

        for (flow_id, rules) in groups {
            self.refresh_flow_server(flow_id, rules).await;
        }
    }

    pub async fn refresh_flow_server(&self, flow_id: u32, infos: Vec<DNSRuleInitInfo>) {
        {
            let mut lock = self.flow_dns_server.lock().await;
            if let Some((old_handler, _)) = lock.get_mut(&flow_id) {
                old_handler.renew_rules(infos).await;
                return;
            }
        }

        let handler = LandscapeDnsRequestHandle::new(infos, flow_id);
        let token = start_dns_server(flow_id, self.addr, handler.clone()).await;

        {
            let mut lock = self.flow_dns_server.lock().await;
            lock.insert(flow_id, (handler, token));
        }
    }
}

pub async fn start_dns_server(
    flow_id: u32,
    addr: SocketAddr,
    handler: LandscapeDnsRequestHandle,
) -> CancellationToken {
    let Ok((udp, sock_fd)) = listener::create_udp_socket(addr).await else {
        tracing::error!("[flow: {flow_id}]: create udp socket error");
        let result = CancellationToken::new();
        result.cancel();
        return result;
    };

    landscape_ebpf::map_setting::dns::setting_dns_sock_map(sock_fd, flow_id);
    landscape_ebpf::dns_dispatcher::attach_reuseport_ebpf(sock_fd).unwrap();
    let mut server = ServerFuture::new(handler);
    server.register_socket(udp);

    let token = server.shutdown_token().clone();

    tokio::spawn(async move {
        if let Err(e) = server.block_until_done().await {
            tracing::error!("[flow: {flow_id}]: server down, error: {e:?}");
        } else {
            tracing::info!("[flow: {flow_id}]: server down");
        }
    });

    token
}

pub struct FlowDnsServer {
    redirect_matcher: DomainMatcher<RedirectInfo>,
    matcher: DomainMatcher<RuleHandlerInfo>,
    resolver_map: HashMap<Uuid, Resolver<MarkConnectionProvider>>,
    default_resolver: Option<RuleHandlerInfo>,
    // cache: Arc<Mutex<DNSCache>>,
}

impl FlowDnsServer {
    pub fn new(infos: Vec<DNSRuleInitInfo>) -> Self {
        let (matcher, resolver_map, default_resolver) = Self::build_rules(infos);
        FlowDnsServer {
            redirect_matcher: DomainMatcher::new(HashMap::new()),
            matcher,
            resolver_map,
            default_resolver,
        }
    }

    pub fn refresh_rule(&mut self, infos: Vec<DNSRuleInitInfo>) {
        let (matcher, resolver_map, default_resolver) = Self::build_rules(infos);
        self.matcher = matcher;
        self.resolver_map = resolver_map;
        self.default_resolver = default_resolver;
    }

    fn build_rules(
        mut infos: Vec<DNSRuleInitInfo>,
    ) -> (
        DomainMatcher<RuleHandlerInfo>,
        HashMap<Uuid, Resolver<MarkConnectionProvider>>,
        Option<RuleHandlerInfo>,
    ) {
        // 排序，优先级值大的规则在前, 这样遍历时就会被 优先级值小的 ( 高 ) 覆盖
        infos.sort_by(|a, b| b.index.cmp(&a.index));

        let mut match_map: HashMap<DomainConfig, Arc<RuleHandlerInfo>> = HashMap::new();
        let mut resolver_map: HashMap<Uuid, Resolver<MarkConnectionProvider>> = HashMap::new();

        let mut default_resolver = None;
        for each in infos {
            let resolver_id = Uuid::new_v4();
            let resolver = new_resolver(&each.resolve_mode, each.flow_id);
            resolver_map.insert(resolver_id, resolver);

            let info = RuleHandlerInfo {
                rule_id: each.id,
                flow_id: each.flow_id,
                resolver_id,
                mark: DnsRuntimeMarkInfo { mark: each.mark, priority: each.index as u16 },
                filter: each.filter,
            };

            if each.source.is_empty() {
                default_resolver = Some(info);
            } else {
                let info = Arc::new(info);

                for each_source in each.source {
                    match_map.insert(each_source, info.clone());
                }
            }
        }

        (DomainMatcher::new(match_map), resolver_map, default_resolver)
    }

    pub fn redirect_lookup(&self, domain: &str, query_type: RecordType) -> Vec<Record> {
        if let Some(info) = self.redirect_matcher.match_value(domain) {
            let mut result = vec![];
            for ip in &info.result_ip {
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

            return result;
        }
        vec![]
    }

    pub async fn lookup(
        &self,
        domain: &str,
        query_type: RecordType,
    ) -> Result<(Vec<Record>, &RuleHandlerInfo), ResponseCode> {
        let info = self.matcher.match_value(&domain).or(self.default_resolver.as_ref());
        if let Some(info) = info {
            if let Some(resolver) = self.resolver_map.get(&info.resolver_id) {
                return match resolver.lookup(domain, query_type).await {
                    Ok(lookup) => Ok((lookup.records().to_vec(), info)),
                    Err(e) => {
                        let result = if e.is_no_records_found() {
                            ResponseCode::NoError
                        } else {
                            tracing::error!(
                                "[flow_id: {:?}] DNS resolution failed for {}: {}",
                                info.flow_id,
                                domain,
                                e
                            );
                            ResponseCode::ServFail
                        };
                        Err(result)
                    }
                };
            }
        }

        Err(ResponseCode::ServFail)
    }
}

pub fn new_resolver(
    resolve_mode: &DnsUpstreamMode,
    mark_value: u32,
) -> Resolver<MarkConnectionProvider> {
    let resolve_config = match resolve_mode {
        DnsUpstreamMode::Upstream { upstream, ips, port } => {
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

            ResolverConfig::from_parts(None, vec![], name_server)
        }
        DnsUpstreamMode::Cloudflare { mode } => {
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
            ResolverConfig::from_parts(None, vec![], server)
        }
    };

    let mut options = ResolverOpts::default();
    options.cache_size = 0;
    options.num_concurrent_reqs = 1;
    options.preserve_intermediates = true;
    // options.use_hosts_file = ResolveHosts::Never;
    let resolver = Resolver::builder_with_config(
        resolve_config,
        MarkConnectionProvider::new(MarkRuntimeProvider::new(mark_value)),
    )
    .with_options(options)
    .build();
    resolver
}
