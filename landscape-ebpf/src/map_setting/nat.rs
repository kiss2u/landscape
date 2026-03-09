use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use landscape_common::iface::nat::StaticNatMappingItem;
use libbpf_rs::{MapCore, MapFlags};

use crate::{
    map_setting::share_map::types::{
        nat_mapping_value_v4, static_nat_mapping_key_v6, static_nat_mapping_value_v6,
    },
    LANDSCAPE_IPV6_TYPE, MAP_PATHS, NAT_MAPPING_EGRESS, NAT_MAPPING_INGRESS,
};

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct NatMappingKeyV4 {
    pub gress: u8,
    pub l4proto: u8,
    pub from_port: u16,
    pub from_addr: u32,
}

unsafe impl plain::Plain for NatMappingKeyV4 {}

#[derive(Debug)]
pub(crate) struct StaticNatMappingV4Item {
    pub wan_port: u16,
    pub lan_port: u16,
    pub lan_ip: Ipv4Addr,
    pub l4_protocol: u8,
}

#[derive(Debug)]
pub(crate) struct StaticNatMappingV6Item {
    pub wan_port: u16,
    pub lan_port: u16,
    pub lan_ip: Ipv6Addr,
    pub l4_protocol: u8,
}

pub(crate) fn add_static_nat4_mapping<'obj, T, I>(nat4_mappings: &T, mappings: I)
where
    T: MapCore,
    I: IntoIterator<Item = StaticNatMappingV4Item>,
    I::IntoIter: ExactSizeIterator,
{
    let mapping_iter = mappings.into_iter();
    if mapping_iter.len() == 0 {
        return;
    }

    let mut keys = vec![];
    let mut values = vec![];
    let counts = (mapping_iter.len() * 2) as u32;

    for static_mapping in mapping_iter {
        // INGRESS key: {INGRESS, proto, wan_port, 0}
        let ingress_mapping_key = NatMappingKeyV4 {
            gress: NAT_MAPPING_INGRESS,
            l4proto: static_mapping.l4_protocol,
            from_port: static_mapping.wan_port.to_be(),
            from_addr: 0, // addr=0 for static ingress
        };

        // EGRESS key: {EGRESS, proto, lan_port, lan_ip}
        let egress_mapping_key = NatMappingKeyV4 {
            gress: NAT_MAPPING_EGRESS,
            l4proto: static_mapping.l4_protocol,
            from_port: static_mapping.lan_port.to_be(),
            from_addr: static_mapping.lan_ip.to_bits().to_be(),
        };

        let mut ingress_mapping_value = nat_mapping_value_v4::default();
        let mut egress_mapping_value = nat_mapping_value_v4::default();

        ingress_mapping_value.port = static_mapping.lan_port.to_be();
        egress_mapping_value.port = static_mapping.wan_port.to_be();
        ingress_mapping_value.is_static = 1;
        egress_mapping_value.is_static = 1;

        let ipv4_addr = static_mapping.lan_ip;
        ingress_mapping_value.addr = ipv4_addr.to_bits().to_be();

        keys.extend_from_slice(unsafe { plain::as_bytes(&ingress_mapping_key) });
        values.extend_from_slice(unsafe { plain::as_bytes(&ingress_mapping_value) });

        keys.extend_from_slice(unsafe { plain::as_bytes(&egress_mapping_key) });
        values.extend_from_slice(unsafe { plain::as_bytes(&egress_mapping_value) });
    }

    if counts == 0 {
        return;
    }

    if let Err(e) = nat4_mappings.update_batch(&keys, &values, counts, MapFlags::ANY, MapFlags::ANY)
    {
        tracing::error!("counts: {counts:?}, update nat4_mappings error:{e:?}");
    }
}

pub(crate) fn add_static_nat6_mapping<'obj, T, I>(static_nat_mappings: &T, mappings: I)
where
    T: MapCore,
    I: IntoIterator<Item = StaticNatMappingV6Item>,
    I::IntoIter: ExactSizeIterator,
{
    let mapping_iter = mappings.into_iter();
    if mapping_iter.len() == 0 {
        return;
    }

    let mut keys = vec![];
    let mut values = vec![];
    let counts = (mapping_iter.len() * 2) as u32;

    for static_mapping in mapping_iter {
        let mut ingress_mapping_key = static_nat_mapping_key_v6 {
            prefixlen: 64, // current only match port
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

        keys.extend_from_slice(unsafe { plain::as_bytes(&ingress_mapping_key) });
        values.extend_from_slice(unsafe { plain::as_bytes(&ingress_mapping_value) });

        keys.extend_from_slice(unsafe { plain::as_bytes(&egress_mapping_key) });
        values.extend_from_slice(unsafe { plain::as_bytes(&egress_mapping_value) });
    }

    if counts == 0 {
        return;
    }

    if let Err(e) =
        static_nat_mappings.update_batch(&keys, &values, counts, MapFlags::ANY, MapFlags::ANY)
    {
        tracing::error!("update static_nat_mappings error:{e:?}");
    }
}

pub fn add_static_nat_mapping<I>(mappings: I)
where
    I: IntoIterator<Item = StaticNatMappingItem>,
    I::IntoIter: ExactSizeIterator,
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

    let nat4_mappings = libbpf_rs::MapHandle::from_pinned_path(&MAP_PATHS.nat4_mappings).unwrap();
    add_static_nat4_mapping(&nat4_mappings, v4_rules);
    let static_nat_mappings =
        libbpf_rs::MapHandle::from_pinned_path(&MAP_PATHS.nat6_static_mappings).unwrap();
    add_static_nat6_mapping(&static_nat_mappings, v6_rules);
}

pub fn del_static_nat_mapping<I>(mappings: I)
where
    I: IntoIterator<Item = StaticNatMappingItem>,
    I::IntoIter: ExactSizeIterator,
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
    let nat4_mappings = libbpf_rs::MapHandle::from_pinned_path(&MAP_PATHS.nat4_mappings).unwrap();
    del_static_nat4_mapping(&nat4_mappings, v4_rules);
    let static_nat_mappings =
        libbpf_rs::MapHandle::from_pinned_path(&MAP_PATHS.nat6_static_mappings).unwrap();
    del_static_nat6_mapping(&static_nat_mappings, v6_rules);
}

pub(crate) fn del_static_nat4_mapping<'obj, T, I>(nat4_mappings: &T, mappings: I)
where
    T: MapCore,
    I: IntoIterator<Item = StaticNatMappingV4Item>,
    I::IntoIter: ExactSizeIterator,
{
    let mapping_iter = mappings.into_iter();
    if mapping_iter.len() == 0 {
        return;
    }

    let mut keys = vec![];
    let counts = (mapping_iter.len() * 2) as u32;

    for static_mapping in mapping_iter {
        // INGRESS key: {INGRESS, proto, wan_port, 0}
        let ingress_mapping_key = NatMappingKeyV4 {
            gress: NAT_MAPPING_INGRESS,
            l4proto: static_mapping.l4_protocol,
            from_port: static_mapping.wan_port.to_be(),
            from_addr: 0,
        };

        // EGRESS key: {EGRESS, proto, lan_port, lan_ip}
        let egress_mapping_key = NatMappingKeyV4 {
            gress: NAT_MAPPING_EGRESS,
            l4proto: static_mapping.l4_protocol,
            from_port: static_mapping.lan_port.to_be(),
            from_addr: static_mapping.lan_ip.to_bits().to_be(),
        };

        keys.extend_from_slice(unsafe { plain::as_bytes(&ingress_mapping_key) });

        keys.extend_from_slice(unsafe { plain::as_bytes(&egress_mapping_key) });
    }

    if counts == 0 {
        return;
    }

    if let Err(e) = nat4_mappings.delete_batch(&keys, counts, MapFlags::ANY, MapFlags::ANY) {
        tracing::error!("delete nat4_mappings error:{e:?}");
    }
}

pub(crate) fn del_static_nat6_mapping<'obj, T, I>(static_nat_mappings: &T, mappings: I)
where
    T: MapCore,
    I: IntoIterator<Item = StaticNatMappingV6Item>,
    I::IntoIter: ExactSizeIterator,
{
    let mapping_iter = mappings.into_iter();
    if mapping_iter.len() == 0 {
        return;
    }

    let mut keys = vec![];
    let counts = (mapping_iter.len() * 2) as u32;

    for static_mapping in mapping_iter {
        let mut ingress_mapping_key = static_nat_mapping_key_v6 {
            prefixlen: 64, // current only match port
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

        let ipv6_addr = static_mapping.lan_ip;
        ingress_mapping_key.l3_protocol = LANDSCAPE_IPV6_TYPE;
        egress_mapping_key.l3_protocol = LANDSCAPE_IPV6_TYPE;
        egress_mapping_key.addr.bytes = ipv6_addr.to_bits().to_be_bytes();

        keys.extend_from_slice(unsafe { plain::as_bytes(&ingress_mapping_key) });

        keys.extend_from_slice(unsafe { plain::as_bytes(&egress_mapping_key) });
    }

    if counts == 0 {
        return;
    }

    if let Err(e) = static_nat_mappings.delete_batch(&keys, counts, MapFlags::ANY, MapFlags::ANY) {
        tracing::error!("update static_nat_mappings error:{e:?}");
    }
}
