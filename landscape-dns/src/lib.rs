use hickory_proto::rr::{Record, RecordType};
use landscape_common::{
    config::dns::FilterResult,
    flow::{DnsRuntimeMarkInfo, FlowMarkInfo},
};
use lru::LruCache;
use std::{collections::HashSet, time::Instant};

pub mod connection;
pub mod diff_server;
pub mod rule;
pub mod server;
pub mod socket;

#[derive(Clone)]
pub struct CacheDNSItem {
    rdatas: Vec<Record>,
    insert_time: Instant,
    mark: DnsRuntimeMarkInfo,
    filter: FilterResult,
}

impl CacheDNSItem {
    fn get_update_rules(&self) -> HashSet<FlowMarkInfo> {
        self.get_update_rules_with_mark(&self.mark)
    }

    fn get_update_rules_with_mark(&self, info: &DnsRuntimeMarkInfo) -> HashSet<FlowMarkInfo> {
        let mut result = HashSet::new();
        for rdata in self.rdatas.iter() {
            match rdata.data() {
                hickory_proto::rr::RData::A(a) => {
                    if info.mark.need_insert_in_ebpf_map() {
                        result.insert(FlowMarkInfo {
                            mark: info.mark.clone().into(),
                            ip: std::net::IpAddr::V4(a.0),
                            priority: info.priority,
                        });
                    }
                }
                hickory_proto::rr::RData::AAAA(a) => {
                    if info.mark.need_insert_in_ebpf_map() {
                        result.insert(FlowMarkInfo {
                            mark: info.mark.clone().into(),
                            ip: std::net::IpAddr::V6(a.0),
                            priority: info.priority,
                        });
                    }
                }
                _ => {}
            }
        }
        result
    }
}

pub type DNSCache = LruCache<(String, RecordType), Vec<CacheDNSItem>>;
