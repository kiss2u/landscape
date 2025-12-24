use landscape_common::{
    config::FlowId,
    route::{LanRouteInfo, RouteTargetInfo},
};
use libbpf_rs::{MapCore, MapFlags};

use crate::{
    map_setting::share_map::types::{route_target_info_v6, route_target_key_v6},
    route::lan_v2::route_lan::types::{
        lan_route_info_v4, lan_route_info_v6, lan_route_key_v4, lan_route_key_v6,
        route_target_info_v4, route_target_key_v4,
    },
    MAP_PATHS,
};

pub mod cache;

pub fn add_lan_route(lan_info: LanRouteInfo) {
    // TODO
    let rt_lan_map = libbpf_rs::MapHandle::from_pinned_path(&MAP_PATHS.rt4_lan_map).unwrap();
    add_lan_route_inner_v4(&rt_lan_map, &lan_info);
    let rt_lan_map = libbpf_rs::MapHandle::from_pinned_path(&MAP_PATHS.rt6_lan_map).unwrap();
    add_lan_route_inner_v6(&rt_lan_map, &lan_info);
}

pub(crate) fn add_lan_route_inner_v4<'obj, T>(rt_lan_map: &T, lan_info: &LanRouteInfo)
where
    T: MapCore,
{
    let mut key = lan_route_key_v4::default();
    let mut value = lan_route_info_v4::default();

    key.prefixlen = lan_info.prefix as u32;
    match lan_info.iface_ip {
        std::net::IpAddr::V4(ipv4_addr) => {
            key.addr = ipv4_addr.to_bits().to_be();
            value.addr = ipv4_addr.to_bits().to_be();
        }
        std::net::IpAddr::V6(_) => {
            return;
        }
    }
    let key = unsafe { plain::as_bytes(&key) };

    value.ifindex = lan_info.ifindex;
    if let Some(mac) = lan_info.mac {
        value.mac_addr = mac.octets();
        value.has_mac = std::mem::MaybeUninit::new(true);
    } else {
        value.has_mac = std::mem::MaybeUninit::new(false);
    }

    match lan_info.mode {
        landscape_common::route::LanRouteMode::Reachable => {
            value.is_next_hop = std::mem::MaybeUninit::new(false);
        }
        landscape_common::route::LanRouteMode::NextHop { next_hop_ip } => {
            value.is_next_hop = std::mem::MaybeUninit::new(true);

            match next_hop_ip {
                std::net::IpAddr::V4(ipv4_addr) => {
                    value.addr = ipv4_addr.to_bits().to_be();
                }
                std::net::IpAddr::V6(_) => {
                    return;
                }
            }
        }
    }

    let value = unsafe { plain::as_bytes(&value) };

    if let Err(e) = rt_lan_map.update(&key, &value, MapFlags::ANY) {
        tracing::error!("add lan config error:{e:?}");
    }
}

pub(crate) fn add_lan_route_inner_v6<'obj, T>(rt_lan_map: &T, lan_info: &LanRouteInfo)
where
    T: MapCore,
{
    let mut key = lan_route_key_v6::default();
    let mut value = lan_route_info_v6::default();

    key.prefixlen = lan_info.prefix as u32;
    match lan_info.iface_ip {
        std::net::IpAddr::V4(_) => {
            return;
        }
        std::net::IpAddr::V6(ipv6_addr) => {
            key.addr.bytes = ipv6_addr.to_bits().to_be_bytes();
            value.addr.bytes = ipv6_addr.to_bits().to_be_bytes();
        }
    }
    let key = unsafe { plain::as_bytes(&key) };

    value.ifindex = lan_info.ifindex;
    if let Some(mac) = lan_info.mac {
        value.mac_addr = mac.octets();
        value.has_mac = std::mem::MaybeUninit::new(true);
    } else {
        value.has_mac = std::mem::MaybeUninit::new(false);
    }

    match lan_info.mode {
        landscape_common::route::LanRouteMode::Reachable => {
            value.is_next_hop = std::mem::MaybeUninit::new(false);
        }
        landscape_common::route::LanRouteMode::NextHop { next_hop_ip } => {
            value.is_next_hop = std::mem::MaybeUninit::new(true);

            match next_hop_ip {
                std::net::IpAddr::V4(_) => {
                    return;
                }
                std::net::IpAddr::V6(ipv6_addr) => {
                    value.addr.bytes = ipv6_addr.to_bits().to_be_bytes();
                }
            }
        }
    }

    let value = unsafe { plain::as_bytes(&value) };

    if let Err(e) = rt_lan_map.update(&key, &value, MapFlags::ANY) {
        tracing::error!("add lan config error:{e:?}");
    }
}

pub fn del_lan_route(lan_info: LanRouteInfo) {
    let rt_lan_map = libbpf_rs::MapHandle::from_pinned_path(&MAP_PATHS.rt4_lan_map).unwrap();
    del_lan_route_inner_v4(&rt_lan_map, &lan_info);
    let rt_lan_map = libbpf_rs::MapHandle::from_pinned_path(&MAP_PATHS.rt6_lan_map).unwrap();
    del_lan_route_inner_v6(&rt_lan_map, &lan_info);
}

pub(crate) fn del_lan_route_inner_v4<'obj, T>(rt_lan_map: &T, lan_info: &LanRouteInfo)
where
    T: MapCore,
{
    let mut key = lan_route_key_v4::default();
    key.prefixlen = lan_info.prefix as u32;
    match lan_info.iface_ip {
        std::net::IpAddr::V4(ipv4_addr) => {
            key.addr = ipv4_addr.to_bits().to_be();
        }
        std::net::IpAddr::V6(_) => {
            return;
        }
    }
    let key = unsafe { plain::as_bytes(&key) };

    if let Err(e) = rt_lan_map.delete(&key) {
        tracing::error!("del lan config error:{e:?}");
    }
}

pub(crate) fn del_lan_route_inner_v6<'obj, T>(rt_lan_map: &T, lan_info: &LanRouteInfo)
where
    T: MapCore,
{
    let mut key = lan_route_key_v6::default();
    key.prefixlen = lan_info.prefix as u32;
    match lan_info.iface_ip {
        std::net::IpAddr::V4(_) => {
            return;
        }
        std::net::IpAddr::V6(ipv6_addr) => {
            key.addr.bytes = ipv6_addr.to_bits().to_be_bytes();
        }
    }
    let key = unsafe { plain::as_bytes(&key) };

    if let Err(e) = rt_lan_map.delete(&key) {
        tracing::error!("del lan config error:{e:?}");
    }
}

pub fn add_wan_route(flow_id: FlowId, wan_info: RouteTargetInfo) {
    let rt_target_map = libbpf_rs::MapHandle::from_pinned_path(&MAP_PATHS.rt4_target_map).unwrap();
    add_wan_route_inner_v4(&rt_target_map, flow_id, &wan_info);
    let rt_target_map = libbpf_rs::MapHandle::from_pinned_path(&MAP_PATHS.rt6_target_map).unwrap();
    add_wan_route_inner_v6(&rt_target_map, flow_id, &wan_info);
}

pub(crate) fn add_wan_route_inner_v4<'obj, T>(
    rt_target_map: &T,
    flow_id: FlowId,
    wan_info: &RouteTargetInfo,
) where
    T: MapCore,
{
    let mut key = route_target_key_v4::default();
    key.flow_id = flow_id;

    let mut value = route_target_info_v4::default();
    value.ifindex = wan_info.ifindex;
    if wan_info.is_docker {
        value.is_docker = 1;
    } else {
        value.is_docker = 0;
    };

    match wan_info.gateway_ip {
        std::net::IpAddr::V4(ipv4_addr) => value.gate_addr = ipv4_addr.to_bits().to_be(),
        std::net::IpAddr::V6(_) => {
            return;
        }
    }

    match wan_info.mac {
        Some(mac) => {
            value.has_mac = 1;
            value.mac = mac.octets();
        }
        None => {
            value.has_mac = 0;
        }
    }

    let key = unsafe { plain::as_bytes(&key) };
    let value = unsafe { plain::as_bytes(&value) };

    if let Err(e) = rt_target_map.update(&key, &value, MapFlags::ANY) {
        tracing::error!("add wan config error:{e:?}");
    }
}

pub(crate) fn add_wan_route_inner_v6<'obj, T>(
    rt_target_map: &T,
    flow_id: FlowId,
    wan_info: &RouteTargetInfo,
) where
    T: MapCore,
{
    let mut key = route_target_key_v6::default();
    key.flow_id = flow_id;

    let mut value = route_target_info_v6::default();
    value.ifindex = wan_info.ifindex;
    if wan_info.is_docker {
        value.is_docker = 1;
    } else {
        value.is_docker = 0;
    };

    match wan_info.gateway_ip {
        std::net::IpAddr::V4(_) => {
            return;
        }
        std::net::IpAddr::V6(ipv6_addr) => {
            value.gate_addr.bytes = ipv6_addr.to_bits().to_be_bytes()
        }
    }

    match wan_info.mac {
        Some(mac) => {
            value.has_mac = 1;
            value.mac = mac.octets();
        }
        None => {
            value.has_mac = 0;
        }
    }

    let key = unsafe { plain::as_bytes(&key) };
    let value = unsafe { plain::as_bytes(&value) };

    if let Err(e) = rt_target_map.update(&key, &value, MapFlags::ANY) {
        tracing::error!("add wan config error:{e:?}");
    }
}

pub fn del_ipv6_wan_route(flow_id: FlowId) {
    let rt_target_map = libbpf_rs::MapHandle::from_pinned_path(&MAP_PATHS.rt6_target_map).unwrap();
    del_wan_route_v6(&rt_target_map, flow_id);
}

pub fn del_ipv4_wan_route(flow_id: FlowId) {
    let rt_target_map = libbpf_rs::MapHandle::from_pinned_path(&MAP_PATHS.rt4_target_map).unwrap();
    del_wan_route_v4(&rt_target_map, flow_id);
}

fn del_wan_route_v4<'obj, T>(rt_target_map: &T, flow_id: FlowId)
where
    T: MapCore,
{
    let mut key = route_target_key_v4::default();
    key.flow_id = flow_id;

    let key = unsafe { plain::as_bytes(&key) };

    if let Err(e) = rt_target_map.delete(&key) {
        tracing::error!("del wan config error:{e:?}");
    }
}

fn del_wan_route_v6<'obj, T>(rt_target_map: &T, flow_id: FlowId)
where
    T: MapCore,
{
    let mut key = route_target_key_v6::default();
    key.flow_id = flow_id;

    let key = unsafe { plain::as_bytes(&key) };

    if let Err(e) = rt_target_map.delete(&key) {
        tracing::error!("del wan config error:{e:?}");
    }
}
