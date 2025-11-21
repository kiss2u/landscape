use std::{
    os::fd::{AsFd, AsRawFd},
    path::Path,
};

use libbpf_rs::{libbpf_sys, MapCore, MapFlags, MapHandle, MapType};

use crate::{
    route::lan::flow_lan_bpf::types::{rt_cache_key, rt_cache_value},
    LandscapeMapPath, MAP_PATHS,
};

const DNS_MATCH_MAX_ENTRIES: u32 = 65536;
const WAN_CACHE: u32 = 0;
const LAN_CACHE: u32 = 1;

fn create_route_cache_inner_map<P>(path: P, name: String, cache_type: u32)
where
    P: AsRef<Path>,
{
    tracing::debug!("rt_cache_map at: {:?}", path.as_ref().display());
    #[allow(clippy::needless_update)]
    let opts = libbpf_sys::bpf_map_create_opts {
        sz: size_of::<libbpf_sys::bpf_map_create_opts>() as libbpf_sys::size_t,
        ..Default::default()
    };

    let key_size = size_of::<rt_cache_key>() as u32;
    let value_size = size_of::<rt_cache_value>() as u32;

    let map = MapHandle::create(
        MapType::LruHash,
        Some(name),
        key_size,
        value_size,
        DNS_MATCH_MAX_ENTRIES,
        &opts,
    )
    .unwrap();

    let map_fd = map.as_fd().as_raw_fd();
    let flow_dns_match_map = libbpf_rs::MapHandle::from_pinned_path(path).unwrap();

    let key_value = unsafe { plain::as_bytes(&cache_type) };

    let value_value = unsafe { plain::as_bytes(&map_fd) };

    if let Err(e) = flow_dns_match_map.update(key_value, value_value, MapFlags::ANY) {
        let last_os_error = std::io::Error::last_os_error();
        tracing::error!("Last OS error: {:?}", last_os_error);
        tracing::error!("Last OS error: {e:?}");
    }
}

pub(crate) fn init_route_wan_cache_inner_map(path: &LandscapeMapPath) {
    create_route_cache_inner_map(&path.rt_cache_map, "rt_cache_wan".to_string(), WAN_CACHE);
}

pub(crate) fn init_route_lan_cache_inner_map(path: &LandscapeMapPath) {
    create_route_cache_inner_map(&path.rt_cache_map, "rt_cache_lan".to_string(), LAN_CACHE);
}

pub fn recreate_route_wan_cache_inner_map() {
    create_route_cache_inner_map(&MAP_PATHS.rt_cache_map, "rt_cache_wan".to_string(), WAN_CACHE);
}

/// 在修改了 DNS 规则， DST IP 规则。 Flow Match 规则后调用
/// 使缓存失效
pub fn recreate_route_lan_cache_inner_map() {
    create_route_cache_inner_map(&MAP_PATHS.rt_cache_map, "rt_cache_lan".to_string(), LAN_CACHE);
}
