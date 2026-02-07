use std::{
    collections::{BTreeMap, HashSet},
    sync::Arc,
    time::{Duration, Instant},
    vec,
};

use arc_swap::ArcSwap;
use hickory_proto::{
    op::{Header, ResponseCode},
    rr::{Record, RecordType},
};
use hickory_server::{
    authority::MessageResponseBuilder,
    server::{Request, RequestHandler, ResponseHandler, ResponseInfo},
};
use moka::future::Cache;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    server::rule::{RedirectSolution, ResolutionRule},
    CacheDNSItem, CheckChainDnsResult, DNSCache,
};
use landscape_common::{
    config::{dns::FilterResult, DnsRuntimeConfig},
    dns::ChainDnsServerInitInfo,
    event::DnsMetricMessage,
    flow::{DnsRuntimeMarkInfo, FlowMarkInfo},
    metric::dns::{DnsMetric, DnsResultStatus},
};

#[derive(Clone, Debug)]
pub struct DnsRequestHandler {
    redirect_solution: Arc<ArcSwap<Vec<RedirectSolution>>>,
    resolves: Arc<ArcSwap<BTreeMap<u32, ResolutionRule>>>,
    pub cache: Arc<ArcSwap<DNSCache>>,
    pub flow_id: u32,
    pub msg_tx: Option<mpsc::Sender<DnsMetricMessage>>,
    pub negative_cache_ttl: u32,
}

impl DnsRequestHandler {
    pub fn new(
        info: ChainDnsServerInitInfo,
        dns_config: DnsRuntimeConfig,
        flow_id: u32,
        msg_tx: Option<mpsc::Sender<DnsMetricMessage>>,
    ) -> DnsRequestHandler {
        let mut resolves = BTreeMap::new();
        for rule in info.dns_rules.into_iter() {
            resolves.insert(rule.index, ResolutionRule::new(rule, flow_id));
        }
        let cache = Cache::builder()
            .max_capacity(dns_config.cache_capacity as u64)
            .time_to_live(Duration::from_secs(dns_config.cache_ttl as u64))
            .build();

        let redirect_solution =
            info.redirect_rules.into_iter().map(RedirectSolution::new).collect();

        DnsRequestHandler {
            resolves: Arc::new(ArcSwap::from_pointee(resolves)),
            cache: Arc::new(ArcSwap::from_pointee(cache)),
            flow_id,
            redirect_solution: Arc::new(ArcSwap::from_pointee(redirect_solution)),
            msg_tx,
            negative_cache_ttl: dns_config.negative_cache_ttl,
        }
    }

    pub async fn renew_rules(
        &mut self,
        info: ChainDnsServerInitInfo,
        dns_config: DnsRuntimeConfig,
    ) {
        let mut resolves = BTreeMap::new();
        for rule in info.dns_rules.into_iter() {
            resolves.insert(rule.index, ResolutionRule::new(rule, self.flow_id));
        }

        let new_cache: DNSCache = Cache::builder()
            .max_capacity(dns_config.cache_capacity as u64)
            .time_to_live(Duration::from_secs(dns_config.cache_ttl as u64))
            .build();

        // Migrate valid cache items to new cache and collect eBPF updates
        let update_dns_mark_list = self.migrate_cache(&new_cache, &resolves).await;

        tracing::info!("add_dns_marks: {:?}", update_dns_mark_list);

        landscape_ebpf::map_setting::flow_dns::refreash_flow_dns_inner_map(
            self.flow_id,
            update_dns_mark_list.into_iter().collect(),
        );

        // Update local state
        self.resolves.store(Arc::new(resolves));
        self.cache.store(Arc::new(new_cache));

        let redirect_solution: Vec<_> =
            info.redirect_rules.into_iter().map(RedirectSolution::new).collect();
        self.redirect_solution.store(Arc::new(redirect_solution));
        self.negative_cache_ttl = dns_config.negative_cache_ttl;

        landscape_ebpf::map_setting::route::cache::recreate_route_lan_cache_inner_map();
    }

    async fn migrate_cache(
        &self,
        new_cache: &DNSCache,
        resolves: &BTreeMap<u32, ResolutionRule>,
    ) -> HashSet<FlowMarkInfo> {
        let mut update_dns_mark_list = HashSet::new();
        let current_cache = self.cache.load();

        for (key, value) in current_cache.iter() {
            let (domain, req_type) = &*key;
            'resolver: for (_index, resolver) in resolves.iter() {
                if resolver.is_match(&domain) {
                    let new_mark = resolver.mark().clone();
                    let cache_item = value;

                    let _already_mapped = cache_item.mark.mark.need_insert_in_ebpf_map();
                    let will_map = new_mark.mark.need_insert_in_ebpf_map();

                    if will_map {
                        update_dns_mark_list
                            .extend(cache_item.get_update_rules_with_mark(&new_mark));
                    }

                    let new_item = CacheDNSItem {
                        rdatas: cache_item.rdatas.clone(),
                        response_code: cache_item.response_code,
                        mark: new_mark.clone(),
                        insert_time: cache_item.insert_time,
                        min_ttl: if cache_item.min_ttl < 5 { cache_item.min_ttl } else { 5 },
                        filter: resolver.filter_mode(),
                    };

                    new_cache.insert((domain.clone(), req_type.clone()), Arc::new(new_item)).await;

                    break 'resolver;
                }
            }
        }
        update_dns_mark_list
    }

    pub fn lookup_redirects(
        &self,
        domain: &str,
        query_type: RecordType,
    ) -> Option<(Vec<Record>, DnsResultStatus, Option<Uuid>)> {
        let redirect_list = self.redirect_solution.load();
        for each in redirect_list.iter() {
            if each.is_match(domain) {
                let records = each.lookup(domain, query_type);
                let status =
                    if each.is_block() { DnsResultStatus::Block } else { DnsResultStatus::Local };
                return Some((records, status, Some(each.id)));
            }
        }
        None
    }

    pub async fn check_domain(&self, domain: &str, query_type: RecordType) -> CheckChainDnsResult {
        let mut result = CheckChainDnsResult::default();

        if let Some((records, _status, id)) = self.lookup_redirects(domain, query_type) {
            result.redirect_id = id;
            result.records = Some(crate::to_common_records(records));
        } else {
            let resolves = self.resolves.load();
            for (_index, resolver) in resolves.iter() {
                if resolver.is_match(domain) {
                    result.rule_id = Some(resolver.get_config_id());

                    if let Ok(Ok(rdata_vec)) = tokio::time::timeout(
                        Duration::from_secs(5),
                        resolver.lookup(domain, query_type),
                    )
                    .await
                    {
                        result.records = Some(crate::to_common_records(rdata_vec));
                    }
                    break;
                }
            }
        }

        if let Some((records, _, _)) = self.lookup_cache(domain, query_type).await {
            result.cache_records = Some(crate::to_common_records(records));
        }

        result
    }

    // 检查缓存并根据 TTL 判断是否过期
    // 不同的记录可能的过期时间不同
    pub async fn lookup_cache(
        &self,
        domain: &str,
        query_type: RecordType,
    ) -> Option<(Vec<Record>, FilterResult, ResponseCode)> {
        let cache = self.cache.load();
        if let Some(cache_item) = cache.get(&(domain.to_string(), query_type)).await {
            let CacheDNSItem {
                rdatas,
                response_code,
                insert_time,
                min_ttl,
                filter,
                ..
            } = &*cache_item;

            // 1. 检查过期
            let insert_time_elapsed = insert_time.elapsed().as_secs() as u32;
            if insert_time_elapsed > *min_ttl {
                // 如果发现过期，主动移除缓存（Lazy expiration）
                cache.invalidate(&(domain.to_string(), query_type)).await;
                return None;
            }

            // 2. 构造有效记录 (TTL 递减)
            // 如果 rdatas 为空（否定缓存），这里 valid_records 也会保持为空
            let valid_records = rdatas
                .iter()
                .cloned()
                .map(|mut d| {
                    d.set_ttl(*min_ttl - insert_time_elapsed);
                    d
                })
                .collect();

            return Some((valid_records, filter.clone(), *response_code));
        }
        None
    }

    pub async fn insert(
        &self,
        domain: &str,
        query_type: RecordType,
        rdata_ttl_vec: Vec<Record>,
        response_code: ResponseCode,
        mark: &DnsRuntimeMarkInfo,
        filter: FilterResult,
    ) {
        let min_ttl =
            rdata_ttl_vec.iter().map(|r| r.ttl()).min().unwrap_or(self.negative_cache_ttl);

        if min_ttl == 0 {
            return;
        }
        let cache_item = CacheDNSItem {
            rdatas: rdata_ttl_vec,
            response_code,
            mark: mark.clone(),
            insert_time: Instant::now(),
            min_ttl,
            filter,
        };
        let update_dns_mark_list = cache_item.get_update_rules();

        let cache = self.cache.load();
        cache.insert((domain.to_string(), query_type), Arc::new(cache_item)).await;

        // 将 mark 写入 mark ebpf map
        if mark.mark.need_insert_in_ebpf_map() {
            tracing::info!(
                "[flow_id: {}]setting ips: {:?}, Mark: {:?}",
                self.flow_id,
                update_dns_mark_list,
                mark
            );
            // TODO: 如果写入错误 返回错误后 向客户端返回查询错误
            landscape_ebpf::map_setting::flow_dns::update_flow_dns_rule(
                self.flow_id,
                update_dns_mark_list.into_iter().collect(),
            );
        }
    }

    fn send_metric(
        &self,
        domain: String,
        query_type: RecordType,
        response_code: ResponseCode,
        status: DnsResultStatus,
        start_time: Instant,
        src_ip: std::net::IpAddr,
        answers: Vec<String>,
    ) {
        if let Some(msg_tx) = &self.msg_tx {
            let dns_metric = DnsMetric {
                flow_id: self.flow_id,
                domain,
                query_type: query_type.to_string(),
                response_code: response_code.to_string(),
                status,
                report_time: landscape_common::utils::time::get_current_time_ms()
                    .unwrap_or_default(),
                duration_ms: start_time.elapsed().as_millis() as u32,
                src_ip,
                answers,
            };
            let _ = msg_tx.try_send(DnsMetricMessage::Metric(dns_metric));
        }
    }

    async fn send_error_response<R: ResponseHandler>(
        &self,
        request: &Request,
        mut response_handle: R,
        code: ResponseCode,
    ) -> ResponseInfo {
        let mut header = Header::response_from_request(request.header());
        header.set_response_code(code);
        header.set_recursion_available(true);
        header.set_authoritative(true);
        let response =
            MessageResponseBuilder::from_message_request(request).build_no_records(header);
        match response_handle.send_response(response).await {
            Ok(info) => info,
            Err(e) => {
                tracing::error!("Error response failed: {}", e);
                serve_failed(request.header())
            }
        }
    }
}

#[async_trait::async_trait]
impl RequestHandler for DnsRequestHandler {
    async fn handle_request<R: ResponseHandler>(
        &self,
        request: &Request,
        mut response_handle: R,
    ) -> ResponseInfo {
        let start_time = Instant::now();
        let queries = request.queries();
        if queries.is_empty() {
            return self.send_error_response(request, response_handle, ResponseCode::FormErr).await;
        }

        let req = &queries[0];
        let domain = req.name().to_string();
        let query_type = req.query_type();
        let src_ip = request.src().ip();

        let mut header = Header::response_from_request(request.header());
        header.set_response_code(ResponseCode::NoError);
        header.set_authoritative(true);
        header.set_recursion_available(true);

        let mut records = vec![];
        let mut status = DnsResultStatus::Normal;

        // 1. Redirects
        if let Some((redirect_records, redirect_status, _)) =
            self.lookup_redirects(&domain, query_type)
        {
            records = redirect_records;
            status = redirect_status;
        }
        // 2. Cache
        else if let Some((cached_records, filter, code)) =
            self.lookup_cache(&domain, query_type).await
        {
            header.set_response_code(code);
            if is_type_filtered(query_type, &filter) {
                status = DnsResultStatus::Filter;
            } else {
                records = filter_result(cached_records, &filter);
                status = DnsResultStatus::Hit;
            }
        }
        // 3. Resolution Rules (with Early Filter check)
        else {
            let resolves = self.resolves.load();
            let mut resolved = false;
            for (_index, resolver) in resolves.iter() {
                if resolver.is_match(&domain) {
                    resolved = true;
                    let filter = resolver.filter_mode();

                    // Early return if current query type is filtered by rule
                    if is_type_filtered(query_type, &filter) {
                        status = DnsResultStatus::Filter;
                        break;
                    }

                    match resolver.lookup(&domain, query_type).await {
                        Ok(rdata_vec) => {
                            self.insert(
                                &domain,
                                query_type,
                                rdata_vec.clone(),
                                ResponseCode::NoError,
                                resolver.mark(),
                                filter.clone(),
                            )
                            .await;

                            records = filter_result(rdata_vec, &filter);
                            status = DnsResultStatus::Normal;
                        }
                        Err(err) => {
                            let code = err.to_response_code();
                            status = match code {
                                ResponseCode::NXDomain => DnsResultStatus::NxDomain,
                                ResponseCode::NoError => DnsResultStatus::Normal,
                                _ => DnsResultStatus::Error,
                            };

                            if code == ResponseCode::NXDomain || code == ResponseCode::NoError {
                                self.insert(
                                    &domain,
                                    query_type,
                                    vec![],
                                    code,
                                    resolver.mark(),
                                    filter.clone(),
                                )
                                .await;
                            }
                            self.send_metric(
                                domain.clone(),
                                query_type,
                                code,
                                status,
                                start_time,
                                src_ip,
                                vec![],
                            );
                            return self.send_error_response(request, response_handle, code).await;
                        }
                    }
                    break;
                }
            }
            if !resolved {
                status = DnsResultStatus::Normal;
            }
        }

        // 4. Send Response
        let builder = MessageResponseBuilder::from_message_request(request);
        let result = if records.is_empty() {
            let response = builder.build_no_records(header);
            response_handle.send_response(response).await
        } else {
            let response = builder.build(
                header,
                records.iter(),
                vec![].into_iter(),
                vec![].into_iter(),
                vec![].into_iter(),
            );
            response_handle.send_response(response).await
        };
        let answers = records.iter().map(|r| r.to_string()).collect();
        self.send_metric(
            domain,
            query_type,
            header.response_code(),
            status,
            start_time,
            src_ip,
            answers,
        );

        match result {
            Ok(info) => info,
            Err(e) => {
                tracing::error!("Response failed: {}", e);
                serve_failed(request.header())
            }
        }
    }
}

fn serve_failed(req_header: &Header) -> ResponseInfo {
    let mut header = Header::response_from_request(req_header);
    header.set_response_code(ResponseCode::ServFail);
    header.set_recursion_available(true);
    header.set_authoritative(true);
    header.into()
}

fn filter_result(un_filter_records: Vec<Record>, filter: &FilterResult) -> Vec<Record> {
    if matches!(filter, FilterResult::Unfilter) {
        return un_filter_records;
    }
    un_filter_records
        .into_iter()
        .filter(|r| match (r.record_type(), filter) {
            (RecordType::A, FilterResult::OnlyIPv4) => true,
            (RecordType::A, FilterResult::OnlyIPv6) => false,
            (RecordType::AAAA, FilterResult::OnlyIPv4) => false,
            (RecordType::AAAA, FilterResult::OnlyIPv6) => true,
            _ => true,
        })
        .collect()
}

fn is_type_filtered(query_type: RecordType, filter: &FilterResult) -> bool {
    match (query_type, filter) {
        (RecordType::A, FilterResult::OnlyIPv6) => true,
        (RecordType::AAAA, FilterResult::OnlyIPv4) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hickory_proto::op::{Header, ResponseCode};
    use hickory_proto::rr::rdata::{A, AAAA};
    use hickory_proto::rr::{RData, Record, RecordType};
    use std::net::{Ipv4Addr, Ipv6Addr};
    use std::str::FromStr;

    #[test]
    fn test_serve_failed_flags() {
        let mut req_header = Header::new();
        req_header.set_id(0x1234);
        req_header.set_recursion_desired(true);

        let res_info = serve_failed(&req_header);

        // ResponseInfo derefs to Header in the version of hickory-server used
        assert_eq!(res_info.id(), 0x1234);
        assert_eq!(res_info.response_code(), ResponseCode::ServFail);
        assert!(res_info.recursion_available(), "RA flag must be true");
        assert!(res_info.authoritative(), "AA flag must be true");
    }

    #[test]
    fn test_is_type_filtered() {
        assert!(is_type_filtered(RecordType::A, &FilterResult::OnlyIPv6));
        assert!(!is_type_filtered(RecordType::AAAA, &FilterResult::OnlyIPv6));
        assert!(is_type_filtered(RecordType::AAAA, &FilterResult::OnlyIPv4));
        assert!(!is_type_filtered(RecordType::A, &FilterResult::OnlyIPv4));
        assert!(!is_type_filtered(RecordType::A, &FilterResult::Unfilter));
    }

    #[test]
    fn test_filter_result() {
        let name = hickory_resolver::Name::from_str("test.com.").unwrap();
        let records = vec![
            Record::from_rdata(name.clone(), 60, RData::A(A(Ipv4Addr::new(1, 1, 1, 1)))),
            Record::from_rdata(
                name.clone(),
                60,
                RData::AAAA(AAAA(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1))),
            ),
        ];

        let filtered_v4 = filter_result(records.clone(), &FilterResult::OnlyIPv4);
        assert_eq!(filtered_v4.len(), 1);
        assert_eq!(filtered_v4[0].record_type(), RecordType::A);

        let filtered_v6 = filter_result(records.clone(), &FilterResult::OnlyIPv6);
        assert_eq!(filtered_v6.len(), 1);
        assert_eq!(filtered_v6[0].record_type(), RecordType::AAAA);

        let filtered_none = filter_result(records.clone(), &FilterResult::Unfilter);
        assert_eq!(filtered_none.len(), 2);
    }
}
