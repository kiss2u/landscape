use landscape_common::flow::{FlowEbpfMatchPair, FlowEntryMatchMode, FlowEntryRule};
use libbpf_rs::{MapCore, MapFlags};

use crate::{
    route::lan::flow_route_bpf::types::flow_match_key, LANDSCAPE_IPV4_TYPE, LANDSCAPE_IPV6_TYPE,
    MAP_PATHS,
};

const FLOW_ENTRY_MODE_MAC: u8 = 0;
const FLOW_ENTRY_MODE_IP: u8 = 1;

impl Into<flow_match_key> for FlowEntryRule {
    fn into(self) -> flow_match_key {
        let mut match_key = flow_match_key::default();
        match self.mode {
            FlowEntryMatchMode::Mac { mac_addr } => {
                match_key.prefixlen = 80;
                match_key.l3_protocol = 0;
                match_key.is_match_ip = FLOW_ENTRY_MODE_MAC;
                match_key.__anon_flow_match_key_1.mac.mac = mac_addr.octets();
            }
            FlowEntryMatchMode::Ip { ip, prefix_len } => {
                match_key.prefixlen = 32 + prefix_len as u32;
                match_key.is_match_ip = FLOW_ENTRY_MODE_IP;
                match ip {
                    std::net::IpAddr::V4(ipv4_addr) => {
                        match_key.l3_protocol = LANDSCAPE_IPV4_TYPE;
                        match_key.__anon_flow_match_key_1.src_addr.ip = ipv4_addr.to_bits().to_be();
                    }
                    std::net::IpAddr::V6(ipv6_addr) => {
                        match_key.l3_protocol = LANDSCAPE_IPV6_TYPE;
                        match_key.__anon_flow_match_key_1.src_addr.bits =
                            ipv6_addr.to_bits().to_be_bytes();
                    }
                }
            }
        }
        match_key
    }
}

/// 更新匹配规则到 flow 的映射
pub fn update_flow_match_rule(rules: Vec<FlowEbpfMatchPair>) {
    if rules.is_empty() {
        return;
    }

    let flow_match_map = libbpf_rs::MapHandle::from_pinned_path(&MAP_PATHS.flow_match_map).unwrap();
    let mut keys = vec![];
    let mut values = vec![];
    let counts = rules.len() as u32;

    for FlowEbpfMatchPair { entry_rule, flow_id } in rules.into_iter() {
        let match_key: flow_match_key = entry_rule.into();
        keys.extend_from_slice(unsafe { plain::as_bytes(&match_key) });
        values.extend_from_slice(unsafe { plain::as_bytes(&flow_id) });
    }
    if let Err(e) =
        flow_match_map.update_batch(&keys, &values, counts, MapFlags::ANY, MapFlags::ANY)
    {
        tracing::error!("update_flow_match_rule error:{e:?}");
    }
    crate::map_setting::route::cache::recreate_route_lan_cache_inner_map();
}

/// 删除匹配规则到 flow 的映射
pub fn del_flow_match_rule(rules: Vec<FlowEntryRule>) {
    if rules.is_empty() {
        return;
    }

    let flow_match_map = libbpf_rs::MapHandle::from_pinned_path(&MAP_PATHS.flow_match_map).unwrap();
    let mut keys = vec![];
    let counts = rules.len() as u32;

    for entry_rule in rules.into_iter() {
        let match_key: flow_match_key = entry_rule.into();
        keys.extend_from_slice(unsafe { plain::as_bytes(&match_key) });
    }
    if let Err(e) = flow_match_map.delete_batch(&keys, counts, MapFlags::ANY, MapFlags::ANY) {
        tracing::error!("del_flow_match_rule error:{e:?}");
    }
    crate::map_setting::route::cache::recreate_route_lan_cache_inner_map();
}
