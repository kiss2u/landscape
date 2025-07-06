use std::mem::MaybeUninit;

mod flow_lan_bpf {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/bpf_rs/flow_lan.skel.rs"));
}

use flow_lan_bpf::*;
use libbpf_rs::{
    skel::{OpenSkel, SkelBuilder},
    TC_INGRESS,
};
use tokio::sync::oneshot;

use crate::{
    bpf_error::LdEbpfResult, landscape::TcHookProxy, MAP_PATHS, WAN_ROUTE_INGRESS_PRIORITY,
};

pub fn wan_route_attach(
    ifindex: u32,
    has_mac: bool,
    service_status: oneshot::Receiver<()>,
) -> LdEbpfResult<()> {
    let mut open_object = MaybeUninit::zeroed();
    let firewall_builder = FlowLanSkelBuilder::default();
    let mut open_skel = firewall_builder.open(&mut open_object)?;

    // 检索匹配规则 MAP
    open_skel.maps.flow_match_map.set_pin_path(&MAP_PATHS.flow_match_map)?;
    open_skel.maps.flow_match_map.reuse_pinned_map(&MAP_PATHS.flow_match_map)?;

    // 分流发送目标 MAP
    open_skel.maps.flow_target_map.set_pin_path(&MAP_PATHS.flow_target_map)?;
    open_skel.maps.flow_target_map.reuse_pinned_map(&MAP_PATHS.flow_target_map)?;

    open_skel.maps.rt_lan_map.set_pin_path(&MAP_PATHS.rt_lan_map)?;
    open_skel.maps.rt_lan_map.reuse_pinned_map(&MAP_PATHS.rt_lan_map)?;

    open_skel.maps.rt_target_map.set_pin_path(&MAP_PATHS.rt_target_map)?;
    open_skel.maps.rt_target_map.reuse_pinned_map(&MAP_PATHS.rt_target_map)?;

    open_skel.maps.ip_mac_tab.set_pin_path(&MAP_PATHS.ip_mac_tab)?;
    open_skel.maps.ip_mac_tab.reuse_pinned_map(&MAP_PATHS.ip_mac_tab)?;

    open_skel.maps.flow_v_dns_map.set_pin_path(&MAP_PATHS.flow_verdict_dns_map)?;
    open_skel.maps.flow_v_dns_map.reuse_pinned_map(&MAP_PATHS.flow_verdict_dns_map)?;

    open_skel.maps.flow_v_ip_map.set_pin_path(&MAP_PATHS.flow_verdict_ip_map)?;
    open_skel.maps.flow_v_ip_map.reuse_pinned_map(&MAP_PATHS.flow_verdict_ip_map)?;

    if !has_mac {
        open_skel.maps.rodata_data.current_eth_net_offset = 0;
    }

    let skel = open_skel.load()?;
    let lan_route_ingress = skel.progs.wan_route_ingress;
    // let flow_egress = skel.progs.flow_egress;

    let mut flow_ingress_hook = TcHookProxy::new(
        &lan_route_ingress,
        ifindex as i32,
        TC_INGRESS,
        WAN_ROUTE_INGRESS_PRIORITY,
    );

    // let mut flow_egress_hook =
    //     TcHookProxy::new(&flow_egress, ifindex as i32, TC_EGRESS, LAN_ROUTE_EGRESS_PRIORITY);

    flow_ingress_hook.attach();
    // flow_egress_hook.attach();
    let _ = service_status.blocking_recv();
    drop(flow_ingress_hook);
    // drop(flow_egress_hook);
    Ok(())
}
