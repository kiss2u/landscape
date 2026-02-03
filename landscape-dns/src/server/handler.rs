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

use crate::{
    server::rule::{RedirectSolution, ResolutionRule},
    CacheDNSItem, CheckChainDnsResult, DNSCache,
};
use landscape_common::{
    config::dns::FilterResult,
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
}

impl DnsRequestHandler {
    pub fn new(
        info: ChainDnsServerInitInfo,
        flow_id: u32,
        msg_tx: Option<mpsc::Sender<DnsMetricMessage>>,
    ) -> DnsRequestHandler {
        let mut resolves = BTreeMap::new();
        for rule in info.dns_rules.into_iter() {
            resolves.insert(rule.index, ResolutionRule::new(rule, flow_id));
        }
        let cache = Cache::builder()
            .max_capacity(crate::DEFAULT_DNS_CACHE_CAPACITY)
            .time_to_live(Duration::from_secs(crate::DEFAULT_DNS_CACHE_TTL))
            .build();

        let redirect_solution =
            info.redirect_rules.into_iter().map(RedirectSolution::new).collect();

        DnsRequestHandler {
            resolves: Arc::new(ArcSwap::from_pointee(resolves)),
            cache: Arc::new(ArcSwap::from_pointee(cache)),
            flow_id,
            redirect_solution: Arc::new(ArcSwap::from_pointee(redirect_solution)),
            msg_tx,
        }
    }

    pub async fn renew_rules(&mut self, info: ChainDnsServerInitInfo) {
        let mut resolves = BTreeMap::new();
        for rule in info.dns_rules.into_iter() {
            resolves.insert(rule.index, ResolutionRule::new(rule, self.flow_id));
        }

        let new_cache: DNSCache = Cache::builder()
            .max_capacity(crate::DEFAULT_DNS_CACHE_CAPACITY)
            .time_to_live(Duration::from_secs(crate::DEFAULT_DNS_CACHE_TTL))
            .build();

        let current_cache = self.cache.load();
        
        // 遍历旧 Cache，迁移并更新有效数据
        let mut update_dns_mark_list: HashSet<FlowMarkInfo> = HashSet::new();

        for (key, value) in current_cache.iter() {
            let (domain, req_type) = &*key;
            'resolver: for (_index, resolver) in resolves.iter() {
                if resolver.is_match(&domain) {
                    let new_mark = resolver.mark().clone();
                    let mut cache_items = vec![];

                    for cache_item in value.iter() {
                        // 新配置是 NoMark 的排除
                        match (
                            cache_item.mark.mark.need_insert_in_ebpf_map(),
                            new_mark.mark.need_insert_in_ebpf_map(),
                        ) {
                            (true, true) => {
                                // 规则更新前后都需要写入 ebpf map
                                // 因为现在是创建新的 map 所以即使一样也需要更新
                                update_dns_mark_list
                                    .extend(cache_item.get_update_rules_with_mark(&new_mark));
                            }
                            (true, false) => {
                                // 原先已经写入了
                                // 现在不需要了, 由于 map 是重新创建的，所以这里不需要显式删除
                                // 只需要不加入 new_map 即可
                            }
                            (false, true) => {
                                // 原先没写入
                                // 目前需要缓存了
                                update_dns_mark_list
                                    .extend(cache_item.get_update_rules_with_mark(&new_mark));
                            }
                            _ => {}
                        }

                        let mut new_record = cache_item.clone();
                        new_record.mark = new_mark.clone();
                        new_record.filter = resolver.filter_mode();
                        new_record.min_ttl =
                            if new_record.min_ttl < 5 { new_record.min_ttl } else { 5 };
                        cache_items.push(new_record);
                    }
                    if !cache_items.is_empty() {
                        new_cache.insert((domain.clone(), req_type.clone()), Arc::new(cache_items)).await;
                    }
                    break 'resolver;
                }
            }
        }
        
        tracing::info!("add_dns_marks: {:?}", update_dns_mark_list);

        landscape_ebpf::map_setting::flow_dns::refreash_flow_dns_inner_map(
            self.flow_id,
            update_dns_mark_list.into_iter().collect(),
        );

        // 先更新规则
        self.resolves.store(Arc::new(resolves));
        // 再替换 Cache
        self.cache.store(Arc::new(new_cache));

        let redirect_solution =
            info.redirect_rules.into_iter().map(RedirectSolution::new).collect();
        self.redirect_solution.store(Arc::new(redirect_solution));
        
        landscape_ebpf::map_setting::route::cache::recreate_route_lan_cache_inner_map();
    }

    pub async fn check_domain(&self, domain: &str, query_type: RecordType) -> CheckChainDnsResult {
        let mut result = CheckChainDnsResult::default();

        let records = {
            let mut records = vec![];
            let redirect_list = self.redirect_solution.load();
            for each in redirect_list.iter() {
                if each.is_match(&domain) {
                    result.redirect_id = Some(each.id);
                    records = each.lookup(&domain, query_type);
                    break;
                }
            }
            records
        };

        if records.is_empty() {
            {
                let resolves = self.resolves.load();
                for (_index, resolver) in resolves.iter() {
                    if resolver.is_match(&domain) {
                        result.rule_id = Some(resolver.get_config_id());

                        match tokio::time::timeout(
                            tokio::time::Duration::from_secs(5),
                            resolver.lookup(&domain, query_type),
                        )
                        .await
                        {
                            Ok(Ok(rdata_vec)) => {
                                result.records = Some(crate::to_common_records(rdata_vec));
                            }
                            Ok(Err(_)) => {
                                // lookup 返回了错误
                            }
                            Err(_) => {
                                tracing::error!("check domain timeout")
                            }
                        }
                        break;
                    }
                }
                drop(resolves);
            }
        } else {
            result.records = Some(crate::to_common_records(records));
        }

        if let Some((records, _)) = self.lookup_cache(domain, query_type).await {
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
    ) -> Option<(Vec<Record>, FilterResult)> {
        let cache = self.cache.load();
        if let Some(records) = cache.get(&(domain.to_string(), query_type)).await {
            let mut is_expire = false;
            let mut valid_records: Vec<Record> = vec![];
            let mut ret_fiter = FilterResult::Unfilter;
            'a_record: for CacheDNSItem { rdatas, insert_time, filter, min_ttl, .. } in
                records.iter()
            {
                ret_fiter = filter.clone();
                let insert_time_elapsed = insert_time.elapsed().as_secs() as u32;
                let min_ttl = *min_ttl;
                if insert_time_elapsed > min_ttl {
                    is_expire = true;
                    break 'a_record;
                }
                valid_records.extend(rdatas.iter().cloned().map(|mut d| {
                    d.set_ttl(min_ttl - insert_time_elapsed);
                    d
                }));
            }

            if is_expire {
                // 如果发现过期，主动移除缓存（Lazy expiration）
                cache.invalidate(&(domain.to_string(), query_type)).await;
                return None;
            }

            // 如果有有效的记录，返回它们
            if !valid_records.is_empty() {
                return Some((valid_records, ret_fiter));
            }
        }
        None
    }

    pub async fn insert(
        &self,
        domain: &str,
        query_type: RecordType,
        rdata_ttl_vec: Vec<Record>,
        mark: &DnsRuntimeMarkInfo,
        filter: FilterResult,
    ) {
        let min_ttl = rdata_ttl_vec.iter().map(|r| r.ttl()).min().unwrap_or(0);
        if min_ttl == 0 {
            return;
        }
        let cache_item = CacheDNSItem {
            rdatas: rdata_ttl_vec,
            insert_time: Instant::now(),
            mark: mark.clone(),
            filter,
            min_ttl,
        };
        let update_dns_mark_list = cache_item.get_update_rules();

        let cache = self.cache.load();
        // 使用 Vec 包装并放入 Arc
        cache.insert((domain.to_string(), query_type), Arc::new(vec![cache_item])).await;

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
            let mut header = Header::response_from_request(request.header());
            header.set_response_code(ResponseCode::FormErr);
            let response =
                MessageResponseBuilder::from_message_request(request).build_no_records(header);
            let result = response_handle.send_response(response).await;
            return match result {
                Err(e) => {
                    tracing::error!("Request failed: {}", e);
                    serve_failed()
                }
                Ok(info) => info,
            };
        }

        // 先只处理第一个查询
        let req = &queries[0];
        let domain = req.name().to_string();
        let query_type = req.query_type();

        let response_builder = MessageResponseBuilder::from_message_request(request);
        let mut header = Header::response_from_request(request.header());
        header.set_response_code(ResponseCode::NoError);
        header.set_authoritative(true);
        header.set_recursion_available(true);

        let mut records = vec![];
        let mut redirect_records = None;
        let mut status = DnsResultStatus::Normal;

        {
            let redirect_list = self.redirect_solution.load();
            for each in redirect_list.iter() {
                if each.is_match(&domain) {
                    redirect_records = Some(each.lookup(&domain, query_type));

                    if each.is_block() {
                        status = DnsResultStatus::Block;
                    } else {
                        status = DnsResultStatus::Local;
                    }

                    break;
                }
            }
        };

        if let Some(redirect_records_inner) = redirect_records {
            records = redirect_records_inner;
        } else {
            if let Some((result, filter)) = self.lookup_cache(&domain, query_type).await {
                records = filter_result(result, &filter);
                status = DnsResultStatus::Hit;
            } else {
                {
                    let resolves = self.resolves.load();
                    let mut found = false;
                    for (_index, resolver) in resolves.iter() {
                        if resolver.is_match(&domain) {
                            found = true;
                            records = match resolver.lookup(&domain, query_type).await {
                                Ok(rdata_vec) => {
                                    if rdata_vec.len() > 0 {
                                        self.insert(
                                            &domain,
                                            query_type,
                                            rdata_vec.clone(),
                                            resolver.mark(),
                                            resolver.filter_mode(),
                                        )
                                        .await;
                                    }
                                    status = DnsResultStatus::Normal;
                                    filter_result(rdata_vec, &resolver.filter_mode())
                                }
                                Err(error_code) => {
                                    status = if error_code == ResponseCode::NXDomain {
                                        DnsResultStatus::NxDomain
                                    } else if error_code == ResponseCode::NoError {
                                        DnsResultStatus::Normal
                                    } else {
                                        DnsResultStatus::Error
                                    };
                                    // 构建并返回错误响应
                                    header.set_response_code(error_code);
                                    let response =
                                        MessageResponseBuilder::from_message_request(request)
                                            .build_no_records(header);

                                    // 发送指标 (由于提前退出，需要在这里发送指标以便记录错误)
                                    if let Some(msg_tx) = &self.msg_tx {
                                        let duration_ms = start_time.elapsed().as_millis() as u32;
                                        let dns_metric = DnsMetric {
                                            flow_id: self.flow_id,
                                            domain: domain.clone(),
                                            query_type: query_type.to_string(),
                                            response_code: header.response_code().to_string(),
                                            status,
                                            report_time:
                                                landscape_common::utils::time::get_current_time_ms()
                                                    .unwrap_or_default(),
                                            duration_ms,
                                            src_ip: request.src().ip(),
                                            answers: vec![],
                                        };
                                        let _ =
                                            msg_tx.try_send(DnsMetricMessage::Metric(dns_metric));
                                    }

                                    let result = response_handle.send_response(response).await;
                                    return match result {
                                        Err(e) => {
                                            tracing::error!("Request failed: {}", e);
                                            serve_failed()
                                        }
                                        Ok(info) => info,
                                    };
                                }
                            };
                            break;
                        }
                    }
                    if !found {
                        // 没有任何规则匹配
                        status = DnsResultStatus::Normal;
                    }
                    drop(resolves);
                }
            }
        }

        // 如果没有找到记录，返回空响应
        if records.is_empty() {
            let response = response_builder.build_no_records(header);
            let result = response_handle.send_response(response).await;

            // 发送空结果指标
            if let Some(msg_tx) = &self.msg_tx {
                let duration_ms = start_time.elapsed().as_millis() as u32;
                let dns_metric = DnsMetric {
                    flow_id: self.flow_id,
                    domain: domain.clone(),
                    query_type: query_type.to_string(),
                    response_code: header.response_code().to_string(),
                    status,
                    report_time: landscape_common::utils::time::get_current_time_ms()
                        .unwrap_or_default(),
                    duration_ms,
                    src_ip: request.src().ip(),
                    answers: vec![],
                };
                let _ = msg_tx.try_send(DnsMetricMessage::Metric(dns_metric));
            }

            return match result {
                Err(e) => {
                    tracing::error!("Record Empty and response error: {}", e);
                    serve_failed()
                }
                Ok(info) => info,
            };
        }

        let response = response_builder.build(
            header,
            records.iter(),
            vec![].into_iter(),
            vec![].into_iter(),
            vec![].into_iter(),
        );

        let result = response_handle.send_response(response).await;

        if let Some(msg_tx) = &self.msg_tx {
            let duration_ms = start_time.elapsed().as_millis() as u32;
            let dns_metric = DnsMetric {
                flow_id: self.flow_id,
                domain: domain.clone(),
                query_type: query_type.to_string(),
                response_code: header.response_code().to_string(),
                status,
                report_time: landscape_common::utils::time::get_current_time_ms()
                    .unwrap_or_default(),
                duration_ms,
                src_ip: request.src().ip(),
                answers: records.iter().map(|r| r.to_string()).collect(),
            };
            let _ = msg_tx.try_send(DnsMetricMessage::Metric(dns_metric));
        }

        match result {
            Err(e) => {
                tracing::error!("Request failed: {}", e);
                serve_failed()
            }
            Ok(info) => info,
        }
    }
}

fn serve_failed() -> ResponseInfo {
    let mut header = Header::new();
    header.set_response_code(ResponseCode::ServFail);
    header.into()
}

fn filter_result(un_filter_records: Vec<Record>, filter: &FilterResult) -> Vec<Record> {
    let mut valid_records = Vec::with_capacity(un_filter_records.len());
    for rdata in un_filter_records.into_iter() {
        if matches!(filter, FilterResult::Unfilter) {
            valid_records.push(rdata.clone());
        } else {
            match (rdata.record_type(), filter) {
                (RecordType::A, FilterResult::OnlyIPv4) => {
                    valid_records.push(rdata.clone());
                }
                (RecordType::A, FilterResult::OnlyIPv6)
                | (RecordType::AAAA, FilterResult::OnlyIPv4) => {
                    // 过滤
                }
                (RecordType::AAAA, FilterResult::OnlyIPv6) => {
                    valid_records.push(rdata.clone());
                }
                _ => {
                    valid_records.push(rdata.clone());
                }
            }
        }
    }
    valid_records
}
