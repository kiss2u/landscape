use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

use hickory_proto::op::{Header, ResponseCode};
use hickory_server::{
    authority::MessageResponseBuilder,
    server::{Request, RequestHandler, ResponseHandler, ResponseInfo},
};

use crate::rule::{DNSRuleConfig, DomainConfig, ResolutionRule};

pub struct DnsServer {
    resolves: BTreeMap<u32, Arc<ResolutionRule>>,
}

impl DnsServer {
    pub fn new(
        dns_rules: Vec<DNSRuleConfig>,
        geo_map: HashMap<String, Vec<DomainConfig>>,
    ) -> DnsServer {
        let mut resolves = BTreeMap::new();

        for rule in dns_rules.into_iter() {
            // println!("dns_rules: {:?}", rule);
            resolves.insert(rule.index, Arc::new(ResolutionRule::new(rule, &geo_map)));
        }
        drop(geo_map);
        DnsServer { resolves }
    }
}

#[async_trait::async_trait]
impl RequestHandler for DnsServer {
    async fn handle_request<R: ResponseHandler>(
        &self,
        request: &Request,
        mut response_handle: R,
    ) -> ResponseInfo {
        let domain = request.query().name().to_string();
        let query_type = request.query().query_type();

        let response_builder = MessageResponseBuilder::from_message_request(request);
        let mut header = Header::response_from_request(request.header());
        header.set_response_code(ResponseCode::NoError);
        header.set_authoritative(true);
        header.set_recursion_available(true);

        let mut records = vec![];

        for (_index, resolver) in self.resolves.iter() {
            if resolver.is_match(&domain).await {
                records = if let Some(cached_rdata) = resolver.lookup(&domain, query_type).await {
                    cached_rdata
                } else {
                    // 如果缓存中没有有效记录，则调用 resolve_domain 进行 DNS 解析
                    match resolver.resolve_domain(&domain, query_type).await {
                        Ok(rdata_vec) => {
                            // 将解析结果插入缓存
                            resolver.insert(domain.clone(), query_type, rdata_vec.clone()).await;
                            rdata_vec
                        }
                        Err(error_code) => {
                            // 构建并返回错误响应
                            header.set_response_code(error_code);
                            let response = MessageResponseBuilder::from_message_request(request)
                                .build_no_records(header);
                            let result = response_handle.send_response(response).await;
                            return match result {
                                Err(e) => {
                                    log::error!("Request failed: {}", e);
                                    serve_failed()
                                }
                                Ok(info) => info,
                            };
                        }
                    }
                };
            }
        }

        // 如果没有找到记录，返回 NXDomain 响应
        if records.is_empty() {
            // header.set_response_code(ResponseCode::NXDomain);
            let response = response_builder.build_no_records(header);
            let result = response_handle.send_response(response).await;
            return match result {
                Err(e) => {
                    log::error!("Request failed: {}", e);
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
        match result {
            Err(e) => {
                log::error!("Request failed: {}", e);
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
