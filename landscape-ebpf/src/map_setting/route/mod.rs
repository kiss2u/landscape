use landscape_common::{
    config::FlowId,
    route::{LanRouteInfo, WanRouteInfo},
};
use libbpf_rs::{MapCore, MapFlags};

use crate::{
    map_setting::share_map::types::{lan_route_info, lan_route_key, wan_route_info, wan_route_key},
    LANDSCAPE_IPV4_TYPE, LANDSCAPE_IPV6_TYPE, MAP_PATHS,
};

pub fn add_lan_route(lan_info: LanRouteInfo) {
    let rt_lan_map = libbpf_rs::MapHandle::from_pinned_path(&MAP_PATHS.rt_lan_map).unwrap();
    let mut key = lan_route_key::default();
    key.prefixlen = lan_info.prefix as u32 + 32;
    match lan_info.iface_ip {
        std::net::IpAddr::V4(ipv4_addr) => {
            key.l3_protocol = LANDSCAPE_IPV4_TYPE;
            unsafe { key.addr.in6_u.u6_addr32[0] = ipv4_addr.to_bits().to_be() };
        }
        std::net::IpAddr::V6(ipv6_addr) => {
            key.l3_protocol = LANDSCAPE_IPV6_TYPE;
            key.addr.in6_u.u6_addr8 = ipv6_addr.to_bits().to_be_bytes()
        }
    }
    let key = unsafe { plain::as_bytes(&key) };

    let mut value = lan_route_info::default();
    value.ifindex = lan_info.ifindex;
    let value = unsafe { plain::as_bytes(&value) };

    if let Err(e) = rt_lan_map.update(&key, &value, MapFlags::ANY) {
        tracing::error!("add lan config error:{e:?}");
    }
}

pub fn del_lan_route(lan_info: LanRouteInfo) {
    let rt_lan_map = libbpf_rs::MapHandle::from_pinned_path(&MAP_PATHS.rt_lan_map).unwrap();
    let mut key = lan_route_key::default();
    key.prefixlen = lan_info.prefix as u32 + 32;
    match lan_info.iface_ip {
        std::net::IpAddr::V4(ipv4_addr) => {
            key.l3_protocol = LANDSCAPE_IPV4_TYPE;
            unsafe { key.addr.in6_u.u6_addr32[0] = ipv4_addr.to_bits().to_be() };
        }
        std::net::IpAddr::V6(ipv6_addr) => {
            key.l3_protocol = LANDSCAPE_IPV6_TYPE;
            key.addr.in6_u.u6_addr8 = ipv6_addr.to_bits().to_be_bytes()
        }
    }
    let key = unsafe { plain::as_bytes(&key) };

    if let Err(e) = rt_lan_map.delete(&key) {
        tracing::error!("del lan config error:{e:?}");
    }
}

pub fn add_wan_route(flow_id: FlowId, wan_info: WanRouteInfo) {
    let rt_wan_map = libbpf_rs::MapHandle::from_pinned_path(&MAP_PATHS.rt_wan_map).unwrap();

    let mut key = wan_route_key::default();
    key.flow_id = flow_id;

    let mut value = wan_route_info::default();
    value.ifindex = wan_info.ifindex;
    match wan_info.gateway_ip {
        std::net::IpAddr::V4(ipv4_addr) => {
            key.l3_protocol = LANDSCAPE_IPV4_TYPE;
            unsafe { value.gate_addr.in6_u.u6_addr32[0] = ipv4_addr.to_bits().to_be() };
        }
        std::net::IpAddr::V6(ipv6_addr) => {
            key.l3_protocol = LANDSCAPE_IPV6_TYPE;
            value.gate_addr.in6_u.u6_addr8 = ipv6_addr.to_bits().to_be_bytes()
        }
    }

    let key = unsafe { plain::as_bytes(&key) };
    let value = unsafe { plain::as_bytes(&value) };

    if let Err(e) = rt_wan_map.update(&key, &value, MapFlags::ANY) {
        tracing::error!("add lan config error:{e:?}");
    }
}
