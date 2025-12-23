use std::mem::MaybeUninit;

pub(crate) mod route_wan {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/bpf_rs/route_wan.skel.rs"));
}

use libbpf_rs::{
    skel::{OpenSkel, SkelBuilder},
    TC_EGRESS, TC_INGRESS,
};
use route_wan::*;
use tokio::sync::oneshot;

use crate::{
    bpf_error::LdEbpfResult, landscape::TcHookProxy, MAP_PATHS, WAN_ROUTE_EGRESS_PRIORITY,
    WAN_ROUTE_INGRESS_PRIORITY,
};

pub fn route_wan(
    ifindex: u32,
    has_mac: bool,
    service_status: oneshot::Receiver<()>,
) -> LdEbpfResult<()> {
    let mut open_object = MaybeUninit::zeroed();
    let firewall_builder = RouteWanSkelBuilder::default();
    let mut open_skel = firewall_builder.open(&mut open_object).unwrap();

    // 检索匹配规则 MAP
    open_skel.maps.flow_match_map.set_pin_path(&MAP_PATHS.flow_match_map).unwrap();
    open_skel.maps.flow_match_map.reuse_pinned_map(&MAP_PATHS.flow_match_map).unwrap();

    open_skel.maps.wan_ipv4_binding.set_pin_path(&MAP_PATHS.wan_ip).unwrap();
    open_skel.maps.wan_ipv4_binding.reuse_pinned_map(&MAP_PATHS.wan_ip).unwrap();

    open_skel.maps.rt4_lan_map.set_pin_path(&MAP_PATHS.rt4_lan_map).unwrap();
    open_skel.maps.rt4_lan_map.reuse_pinned_map(&MAP_PATHS.rt4_lan_map).unwrap();

    open_skel.maps.rt6_lan_map.set_pin_path(&MAP_PATHS.rt6_lan_map).unwrap();
    open_skel.maps.rt6_lan_map.reuse_pinned_map(&MAP_PATHS.rt6_lan_map).unwrap();

    open_skel.maps.rt4_target_map.set_pin_path(&MAP_PATHS.rt4_target_map).unwrap();
    open_skel.maps.rt4_target_map.reuse_pinned_map(&MAP_PATHS.rt4_target_map).unwrap();

    open_skel.maps.rt6_target_map.set_pin_path(&MAP_PATHS.rt6_target_map).unwrap();
    open_skel.maps.rt6_target_map.reuse_pinned_map(&MAP_PATHS.rt6_target_map).unwrap();

    open_skel.maps.flow4_dns_map.set_pin_path(&MAP_PATHS.flow4_dns_map).unwrap();
    open_skel.maps.flow4_dns_map.reuse_pinned_map(&MAP_PATHS.flow4_dns_map).unwrap();

    open_skel.maps.flow6_dns_map.set_pin_path(&MAP_PATHS.flow6_dns_map).unwrap();
    open_skel.maps.flow6_dns_map.reuse_pinned_map(&MAP_PATHS.flow6_dns_map).unwrap();

    open_skel.maps.flow4_ip_map.set_pin_path(&MAP_PATHS.flow4_ip_map).unwrap();
    open_skel.maps.flow4_ip_map.reuse_pinned_map(&MAP_PATHS.flow4_ip_map).unwrap();

    open_skel.maps.flow6_ip_map.set_pin_path(&MAP_PATHS.flow6_ip_map).unwrap();
    open_skel.maps.flow6_ip_map.reuse_pinned_map(&MAP_PATHS.flow6_ip_map).unwrap();

    open_skel.maps.rt4_cache_map.set_pin_path(&MAP_PATHS.rt4_cache_map).unwrap();
    open_skel.maps.rt4_cache_map.reuse_pinned_map(&MAP_PATHS.rt4_cache_map).unwrap();

    open_skel.maps.rt6_cache_map.set_pin_path(&MAP_PATHS.rt6_cache_map).unwrap();
    open_skel.maps.rt6_cache_map.reuse_pinned_map(&MAP_PATHS.rt6_cache_map).unwrap();

    open_skel.maps.ip_mac_v4.set_pin_path(&MAP_PATHS.ip_mac_v4).unwrap();
    open_skel.maps.ip_mac_v4.reuse_pinned_map(&MAP_PATHS.ip_mac_v4).unwrap();

    open_skel.maps.ip_mac_v6.set_pin_path(&MAP_PATHS.ip_mac_v6).unwrap();
    open_skel.maps.ip_mac_v6.reuse_pinned_map(&MAP_PATHS.ip_mac_v6).unwrap();

    let rodata_data =
        open_skel.maps.rodata_data.as_deref_mut().expect("`rodata` is not memery mapped");

    if !has_mac {
        rodata_data.current_l3_offset = 0;
    }

    let skel = open_skel.load().unwrap();

    let wan_route_ingress = skel.progs.route_wan_ingress;
    let wan_route_egress = skel.progs.route_wan_egress;

    let mut wan_route_ingress_hook = TcHookProxy::new(
        &wan_route_ingress,
        ifindex as i32,
        TC_INGRESS,
        WAN_ROUTE_INGRESS_PRIORITY,
    );

    let mut wan_route_egress_hook =
        TcHookProxy::new(&wan_route_egress, ifindex as i32, TC_EGRESS, WAN_ROUTE_EGRESS_PRIORITY);

    wan_route_ingress_hook.attach();
    wan_route_egress_hook.attach();

    let _ = service_status.blocking_recv();

    drop(wan_route_ingress_hook);
    drop(wan_route_egress_hook);
    Ok(())
}
