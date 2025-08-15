use landscape_common::config::nat::StaticNatMappingConfig;
use libbpf_rs::{MapCore, MapFlags};

use crate::{
    map_setting::share_map::types::{nat_mapping_value, static_nat_mapping_key},
    LANDSCAPE_IPV4_TYPE, LANDSCAPE_IPV6_TYPE, MAP_PATHS, NAT_MAPPING_EGRESS, NAT_MAPPING_INGRESS,
};

pub fn add_static_nat_mapping(mappings: Vec<StaticNatMappingConfig>) {
    if mappings.len() == 0 {
        return;
    }

    let static_nat_mappings =
        libbpf_rs::MapHandle::from_pinned_path(&MAP_PATHS.static_nat_mappings).unwrap();
    let mut keys = vec![];
    let mut values = vec![];
    let counts = (mappings.len() * 2) as u32;

    for static_mapping in mappings.into_iter() {
        let mut ingress_mapping_key = static_nat_mapping_key {
            prefixlen: 64, // current only match port
            port: static_mapping.wan_port.to_be(),
            gress: NAT_MAPPING_INGRESS,
            l4_protocol: static_mapping.l4_protocol,
            ..Default::default()
        };

        let mut egress_mapping_key = static_nat_mapping_key {
            prefixlen: 192,
            port: static_mapping.lan_port.to_be(),
            gress: NAT_MAPPING_EGRESS,
            l4_protocol: static_mapping.l4_protocol,
            ..Default::default()
        };

        let mut ingress_mapping_value = nat_mapping_value::default();
        let mut egress_mapping_value = nat_mapping_value::default();

        ingress_mapping_value.port = static_mapping.lan_port.to_be();
        egress_mapping_value.port = static_mapping.wan_port.to_be();
        ingress_mapping_value.is_static = 1;
        egress_mapping_value.is_static = 1;

        match static_mapping.lan_ip {
            std::net::IpAddr::V4(ipv4_addr) => {
                ingress_mapping_key.l3_protocol = LANDSCAPE_IPV4_TYPE;
                egress_mapping_key.l3_protocol = LANDSCAPE_IPV4_TYPE;
                egress_mapping_key.addr.ip = ipv4_addr.to_bits().to_be();
                ingress_mapping_value.addr.ip = ipv4_addr.to_bits().to_be();
                if ipv4_addr.is_unspecified() {
                    egress_mapping_key.prefixlen = 64;
                }
            }
            std::net::IpAddr::V6(ipv6_addr) => {
                ingress_mapping_key.l3_protocol = LANDSCAPE_IPV6_TYPE;
                egress_mapping_key.l3_protocol = LANDSCAPE_IPV6_TYPE;
                egress_mapping_key.addr.bits = ipv6_addr.to_bits().to_be_bytes();
                ingress_mapping_value.addr.bits = ipv6_addr.to_bits().to_be_bytes();
            }
        }

        keys.extend_from_slice(unsafe { plain::as_bytes(&ingress_mapping_key) });
        values.extend_from_slice(unsafe { plain::as_bytes(&ingress_mapping_value) });

        keys.extend_from_slice(unsafe { plain::as_bytes(&egress_mapping_key) });
        values.extend_from_slice(unsafe { plain::as_bytes(&egress_mapping_value) });
    }

    if let Err(e) =
        static_nat_mappings.update_batch(&keys, &values, counts, MapFlags::ANY, MapFlags::ANY)
    {
        tracing::error!("update static_nat_mappings error:{e:?}");
    }
}

pub fn del_static_nat_mapping(mappings: Vec<StaticNatMappingConfig>) {
    if mappings.len() == 0 {
        return;
    }

    let static_nat_mappings =
        libbpf_rs::MapHandle::from_pinned_path(&MAP_PATHS.static_nat_mappings).unwrap();
    let mut keys = vec![];
    let counts = (mappings.len() * 2) as u32;

    for static_mapping in mappings.into_iter() {
        let mut ingress_mapping_key = static_nat_mapping_key {
            prefixlen: 64, // current only match port
            port: static_mapping.wan_port.to_be(),
            gress: NAT_MAPPING_INGRESS,
            l4_protocol: static_mapping.l4_protocol,
            ..Default::default()
        };

        let mut egress_mapping_key = static_nat_mapping_key {
            prefixlen: 192,
            port: static_mapping.lan_port.to_be(),
            gress: NAT_MAPPING_EGRESS,
            l4_protocol: static_mapping.l4_protocol,
            ..Default::default()
        };

        match static_mapping.lan_ip {
            std::net::IpAddr::V4(ipv4_addr) => {
                ingress_mapping_key.l3_protocol = LANDSCAPE_IPV4_TYPE;
                egress_mapping_key.l3_protocol = LANDSCAPE_IPV4_TYPE;
                egress_mapping_key.addr.ip = ipv4_addr.to_bits().to_be();
                if ipv4_addr.is_unspecified() {
                    egress_mapping_key.prefixlen = 64;
                }
            }
            std::net::IpAddr::V6(ipv6_addr) => {
                ingress_mapping_key.l3_protocol = LANDSCAPE_IPV6_TYPE;
                egress_mapping_key.l3_protocol = LANDSCAPE_IPV6_TYPE;
                egress_mapping_key.addr.bits = ipv6_addr.to_bits().to_be_bytes();
            }
        }

        keys.extend_from_slice(unsafe { plain::as_bytes(&ingress_mapping_key) });

        keys.extend_from_slice(unsafe { plain::as_bytes(&egress_mapping_key) });
    }

    if let Err(e) = static_nat_mappings.delete_batch(&keys, counts, MapFlags::ANY, MapFlags::ANY) {
        tracing::error!("update static_nat_mappings error:{e:?}");
    }
}
