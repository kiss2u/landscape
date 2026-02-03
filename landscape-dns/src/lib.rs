use hickory_proto::rr::{Record, RecordType};
use landscape_common::config::dns::LandscapeDnsRecordType;
use landscape_common::{
    config::dns::FilterResult,
    flow::{DnsRuntimeMarkInfo, FlowMarkInfo},
};

pub use landscape_common::dns::check::{
    CheckChainDnsResult, CheckDnsReq, CheckDnsResult, LandscapeRecord as CommonRecord,
};
use lru::LruCache;
use std::{collections::HashSet, path::PathBuf, time::Instant};

pub fn to_common_records(records: Vec<Record>) -> Vec<CommonRecord> {
    records
        .into_iter()
        .map(|r| {
            let data = format!("{}", r.data());
            CommonRecord {
                name: r.name().to_string(),
                rr_type: r.record_type().to_string(),
                ttl: r.ttl(),
                data,
            }
        })
        .collect()
}

pub(crate) mod connection;

pub mod listener;
pub mod server;

const DEFAULT_ENABLE_IP_VALIDATION: bool = false;

static RESOLVER_CONF: &'static str = "/etc/resolv.conf";
static RESOLVER_CONF_LD_BACK: &'static str = "/etc/resolv.conf.ld_back";

fn check_resolver_conf() {
    let resolver_file = PathBuf::from(RESOLVER_CONF);
    let resolver_file_back = PathBuf::from(RESOLVER_CONF_LD_BACK);
    let new_content = "nameserver 127.0.0.1\n";

    if resolver_file.is_symlink() {
        // 如果是符号链接，直接删除
        std::fs::remove_file(&resolver_file).unwrap();
    } else if resolver_file.exists() {
        if resolver_file.is_file() {
            // 如果是普通文件，检查备份文件
            if resolver_file_back.exists() {
                std::fs::remove_file(&resolver_file).unwrap();
            } else {
                let Ok(()) = std::fs::rename(&resolver_file, &resolver_file_back) else {
                    tracing::error!("move {resolver_file:?} error, Skip it");
                    return;
                };
            }
        } else {
            panic!("other kind file");
        }
    }

    // 写入新内容到 /etc/resolv.conf
    std::fs::write(&resolver_file, new_content).unwrap();
}

pub fn convert_record_type(record_type: LandscapeDnsRecordType) -> RecordType {
    match record_type {
        LandscapeDnsRecordType::A => RecordType::A,
        LandscapeDnsRecordType::AAAA => RecordType::AAAA,
    }
}

#[derive(Clone, Debug)]
pub struct CacheDNSItem {
    rdatas: Vec<Record>,
    insert_time: Instant,
    mark: DnsRuntimeMarkInfo,
    filter: FilterResult,
    min_ttl: u32,
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
