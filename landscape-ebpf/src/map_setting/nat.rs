use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use landscape_common::iface::nat::{RuntimeStaticNatMappingConfig, StaticNatMappingItem};
use libbpf_rs::{MapCore, MapFlags};

use crate::bpf_error::LdEbpfResult;
use crate::{
    map_setting::share_map::types::{
        nat_mapping_value_v4_v3, static_nat_mapping_key_v6, static_nat_mapping_value_v6,
    },
    LANDSCAPE_IPV6_TYPE, MAP_PATHS, NAT_MAPPING_EGRESS, NAT_MAPPING_INGRESS,
};

use super::{apply_raw_map_diff, diff_raw_map, snapshot_raw_map, RawEbpfMapEntries};

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct NatMappingKeyV4 {
    pub gress: u8,
    pub l4proto: u8,
    pub from_port: u16,
    pub from_addr: u32,
}

unsafe impl plain::Plain for NatMappingKeyV4 {}

#[derive(Debug, Clone, Copy)]
pub struct StaticNatMappingV4Item {
    pub wan_port: u16,
    pub lan_port: u16,
    pub lan_ip: Ipv4Addr,
    pub l4_protocol: u8,
}

#[derive(Debug)]
pub struct StaticNatMappingV6Item {
    pub wan_port: u16,
    pub lan_port: u16,
    pub lan_ip: Ipv6Addr,
    pub l4_protocol: u8,
}

pub fn build_static_nat4_entries(configs: &[RuntimeStaticNatMappingConfig]) -> RawEbpfMapEntries {
    let mut entries = RawEbpfMapEntries::new();
    for config in configs {
        let Some(lan_ip) = config.lan_ipv4 else {
            continue;
        };
        for l4_protocol in &config.ipv4_l4_protocol {
            for pair in &config.mapping_pair_ports {
                insert_static_nat4_item_entries(
                    &mut entries,
                    StaticNatMappingV4Item {
                        wan_port: pair.wan_port,
                        lan_port: pair.lan_port,
                        lan_ip,
                        l4_protocol: *l4_protocol,
                    },
                );
            }
        }
    }
    entries
}

pub fn build_static_nat6_entries(configs: &[RuntimeStaticNatMappingConfig]) -> RawEbpfMapEntries {
    let mut entries = RawEbpfMapEntries::new();
    for config in configs {
        let Some(lan_ip) = config.lan_ipv6 else {
            continue;
        };
        for l4_protocol in &config.ipv6_l4_protocol {
            for pair in &config.mapping_pair_ports {
                insert_static_nat6_item_entries(
                    &mut entries,
                    StaticNatMappingV6Item {
                        wan_port: pair.wan_port,
                        lan_port: pair.lan_port,
                        lan_ip,
                        l4_protocol: *l4_protocol,
                    },
                );
            }
        }
    }
    entries
}

pub fn reconcile_static_nat4_entries(desired: RawEbpfMapEntries) -> LdEbpfResult<()> {
    let nat4_st_map = libbpf_rs::MapHandle::from_pinned_path(&MAP_PATHS.nat4_st_map)?;
    reconcile_raw_map(&nat4_st_map, desired)
}

pub fn reconcile_static_nat6_entries(desired: RawEbpfMapEntries) -> LdEbpfResult<()> {
    let static_nat_mappings =
        libbpf_rs::MapHandle::from_pinned_path(&MAP_PATHS.nat6_static_mappings)?;
    reconcile_raw_map(&static_nat_mappings, desired)
}

pub fn reconcile_static_nat4_map(configs: &[RuntimeStaticNatMappingConfig]) -> LdEbpfResult<()> {
    reconcile_static_nat4_entries(build_static_nat4_entries(configs))
}

pub fn reconcile_static_nat6_map(configs: &[RuntimeStaticNatMappingConfig]) -> LdEbpfResult<()> {
    reconcile_static_nat6_entries(build_static_nat6_entries(configs))
}

fn reconcile_raw_map<M: MapCore>(map: &M, desired: RawEbpfMapEntries) -> LdEbpfResult<()> {
    let current = snapshot_raw_map(map)?;
    let diff = diff_raw_map(&current, &desired);
    apply_raw_map_diff(map, diff)
}

fn insert_static_nat4_item_entries(
    entries: &mut RawEbpfMapEntries,
    static_mapping: StaticNatMappingV4Item,
) {
    let ingress_mapping_key = NatMappingKeyV4 {
        gress: NAT_MAPPING_INGRESS,
        l4proto: static_mapping.l4_protocol,
        from_port: static_mapping.wan_port.to_be(),
        from_addr: 0,
    };

    let egress_mapping_key = NatMappingKeyV4 {
        gress: NAT_MAPPING_EGRESS,
        l4proto: static_mapping.l4_protocol,
        from_port: static_mapping.lan_port.to_be(),
        from_addr: static_mapping.lan_ip.to_bits().to_be(),
    };

    let mut ingress_mapping_value = nat_mapping_value_v4_v3::default();
    let mut egress_mapping_value = nat_mapping_value_v4_v3::default();

    ingress_mapping_value.port = static_mapping.lan_port.to_be();
    ingress_mapping_value.addr = static_mapping.lan_ip.to_bits().to_be();
    ingress_mapping_value.is_static = 1;

    egress_mapping_value.port = static_mapping.wan_port.to_be();
    egress_mapping_value.is_static = 1;

    entries.insert(
        unsafe { plain::as_bytes(&ingress_mapping_key) }.to_vec(),
        unsafe { plain::as_bytes(&ingress_mapping_value) }.to_vec(),
    );
    entries.insert(
        unsafe { plain::as_bytes(&egress_mapping_key) }.to_vec(),
        unsafe { plain::as_bytes(&egress_mapping_value) }.to_vec(),
    );
}

fn insert_static_nat6_item_entries(
    entries: &mut RawEbpfMapEntries,
    static_mapping: StaticNatMappingV6Item,
) {
    let mut ingress_mapping_key = static_nat_mapping_key_v6 {
        prefixlen: 64,
        port: static_mapping.wan_port.to_be(),
        gress: NAT_MAPPING_INGRESS,
        l4_protocol: static_mapping.l4_protocol,
        ..Default::default()
    };

    let mut egress_mapping_key = static_nat_mapping_key_v6 {
        prefixlen: 192,
        port: static_mapping.lan_port.to_be(),
        gress: NAT_MAPPING_EGRESS,
        l4_protocol: static_mapping.l4_protocol,
        ..Default::default()
    };

    let mut ingress_mapping_value = static_nat_mapping_value_v6::default();
    let mut egress_mapping_value = static_nat_mapping_value_v6::default();

    ingress_mapping_value.port = static_mapping.lan_port.to_be();
    egress_mapping_value.port = static_mapping.wan_port.to_be();
    ingress_mapping_value.is_static = 1;
    egress_mapping_value.is_static = 1;

    let ipv6_addr = static_mapping.lan_ip;
    ingress_mapping_key.l3_protocol = LANDSCAPE_IPV6_TYPE;
    egress_mapping_key.l3_protocol = LANDSCAPE_IPV6_TYPE;
    egress_mapping_key.addr.bytes = ipv6_addr.to_bits().to_be_bytes();
    ingress_mapping_value.addr.bytes = ipv6_addr.to_bits().to_be_bytes();

    entries.insert(
        unsafe { plain::as_bytes(&ingress_mapping_key) }.to_vec(),
        unsafe { plain::as_bytes(&ingress_mapping_value) }.to_vec(),
    );
    entries.insert(
        unsafe { plain::as_bytes(&egress_mapping_key) }.to_vec(),
        unsafe { plain::as_bytes(&egress_mapping_value) }.to_vec(),
    );
}

pub(crate) fn add_static_nat4_mapping<'obj, T, I>(nat4_st_map: &T, mappings: I)
where
    T: MapCore,
    I: IntoIterator<Item = StaticNatMappingV4Item>,
    I::IntoIter: ExactSizeIterator,
{
    let desired = raw_static_nat4_entries_from_items(mappings);
    if desired.is_empty() {
        return;
    }
    if let Err(e) = update_raw_entries(nat4_st_map, desired) {
        tracing::error!("update nat4_st_map error:{e:?}");
    }
}

pub fn add_static_nat4_mapping_v3<'obj, T, I>(nat4_st_map: &T, mappings: I)
where
    T: MapCore,
    I: IntoIterator<Item = StaticNatMappingV4Item>,
    I::IntoIter: ExactSizeIterator,
{
    add_static_nat4_mapping(nat4_st_map, mappings)
}

pub fn add_static_nat6_mapping<'obj, T, I>(static_nat_mappings: &T, mappings: I)
where
    T: MapCore,
    I: IntoIterator<Item = StaticNatMappingV6Item>,
    I::IntoIter: ExactSizeIterator,
{
    let desired = raw_static_nat6_entries_from_items(mappings);
    if desired.is_empty() {
        return;
    }
    if let Err(e) = update_raw_entries(static_nat_mappings, desired) {
        tracing::error!("update static_nat_mappings error:{e:?}");
    }
}

pub(crate) fn del_static_nat4_mapping<'obj, T, I>(nat4_st_map: &T, mappings: I)
where
    T: MapCore,
    I: IntoIterator<Item = StaticNatMappingV4Item>,
    I::IntoIter: ExactSizeIterator,
{
    let desired = raw_static_nat4_entries_from_items(mappings);
    if desired.is_empty() {
        return;
    }
    if let Err(e) = delete_raw_keys(nat4_st_map, desired.into_keys().collect()) {
        tracing::error!("delete nat4_st_map error:{e:?}");
    }
}

pub(crate) fn del_static_nat6_mapping<'obj, T, I>(static_nat_mappings: &T, mappings: I)
where
    T: MapCore,
    I: IntoIterator<Item = StaticNatMappingV6Item>,
    I::IntoIter: ExactSizeIterator,
{
    let desired = raw_static_nat6_entries_from_items(mappings);
    if desired.is_empty() {
        return;
    }
    if let Err(e) = delete_raw_keys(static_nat_mappings, desired.into_keys().collect()) {
        tracing::error!("delete static_nat_mappings error:{e:?}");
    }
}

pub fn add_static_nat_mapping<I>(mappings: I)
where
    I: IntoIterator<Item = StaticNatMappingItem>,
    I::IntoIter: ExactSizeIterator,
{
    let (v4_rules, v6_rules) = split_static_nat_items(mappings);
    let nat4_st_map = libbpf_rs::MapHandle::from_pinned_path(&MAP_PATHS.nat4_st_map).unwrap();
    add_static_nat4_mapping_v3(&nat4_st_map, v4_rules);
    let static_nat_mappings =
        libbpf_rs::MapHandle::from_pinned_path(&MAP_PATHS.nat6_static_mappings).unwrap();
    add_static_nat6_mapping(&static_nat_mappings, v6_rules);
}

pub fn del_static_nat_mapping<I>(mappings: I)
where
    I: IntoIterator<Item = StaticNatMappingItem>,
    I::IntoIter: ExactSizeIterator,
{
    let (v4_rules, v6_rules) = split_static_nat_items(mappings);
    let nat4_st_map = libbpf_rs::MapHandle::from_pinned_path(&MAP_PATHS.nat4_st_map).unwrap();
    del_static_nat4_mapping(&nat4_st_map, v4_rules);
    let static_nat_mappings =
        libbpf_rs::MapHandle::from_pinned_path(&MAP_PATHS.nat6_static_mappings).unwrap();
    del_static_nat6_mapping(&static_nat_mappings, v6_rules);
}

fn split_static_nat_items<I>(
    mappings: I,
) -> (Vec<StaticNatMappingV4Item>, Vec<StaticNatMappingV6Item>)
where
    I: IntoIterator<Item = StaticNatMappingItem>,
{
    let mut v4_rules = vec![];
    let mut v6_rules = vec![];

    for mapping in mappings {
        match mapping.lan_ip {
            IpAddr::V4(ipv4_addr) => {
                v4_rules.push(StaticNatMappingV4Item {
                    wan_port: mapping.wan_port,
                    lan_port: mapping.lan_port,
                    lan_ip: ipv4_addr,
                    l4_protocol: mapping.l4_protocol,
                });
            }
            IpAddr::V6(ipv6_addr) => {
                v6_rules.push(StaticNatMappingV6Item {
                    wan_port: mapping.wan_port,
                    lan_port: mapping.lan_port,
                    lan_ip: ipv6_addr,
                    l4_protocol: mapping.l4_protocol,
                });
            }
        }
    }

    (v4_rules, v6_rules)
}

fn raw_static_nat4_entries_from_items<I>(mappings: I) -> RawEbpfMapEntries
where
    I: IntoIterator<Item = StaticNatMappingV4Item>,
{
    let mut entries = RawEbpfMapEntries::new();
    for mapping in mappings {
        insert_static_nat4_item_entries(&mut entries, mapping);
    }
    entries
}

fn raw_static_nat6_entries_from_items<I>(mappings: I) -> RawEbpfMapEntries
where
    I: IntoIterator<Item = StaticNatMappingV6Item>,
{
    let mut entries = RawEbpfMapEntries::new();
    for mapping in mappings {
        insert_static_nat6_item_entries(&mut entries, mapping);
    }
    entries
}

fn update_raw_entries<M: MapCore>(map: &M, entries: RawEbpfMapEntries) -> LdEbpfResult<()> {
    let entry_count = entries.len() as u32;
    let key_len: usize = entries.keys().map(Vec::len).sum();
    let value_len: usize = entries.values().map(Vec::len).sum();
    let mut keys = Vec::with_capacity(key_len);
    let mut values = Vec::with_capacity(value_len);
    for (key, value) in entries {
        keys.extend_from_slice(&key);
        values.extend_from_slice(&value);
    }
    map.update_batch(&keys, &values, entry_count, MapFlags::ANY, MapFlags::ANY)?;
    Ok(())
}

fn delete_raw_keys<M: MapCore>(map: &M, raw_keys: Vec<Vec<u8>>) -> LdEbpfResult<()> {
    let key_count = raw_keys.len() as u32;
    let key_len: usize = raw_keys.iter().map(Vec::len).sum();
    let mut keys = Vec::with_capacity(key_len);
    for key in raw_keys {
        keys.extend_from_slice(&key);
    }
    map.delete_batch(&keys, key_count, MapFlags::ANY, MapFlags::ANY)?;
    Ok(())
}
