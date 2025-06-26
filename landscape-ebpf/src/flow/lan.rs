use std::mem::MaybeUninit;

mod flow_lan_bpf {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/bpf_rs/flow_lan.skel.rs"));
}

use flow_lan_bpf::*;
use libbpf_rs::{
    skel::{OpenSkel, SkelBuilder},
    TC_EGRESS, TC_INGRESS,
};
use tokio::sync::oneshot;

use crate::{
    bpf_error::LdEbpfResult, landscape::TcHookProxy, LAN_FLOW_MARK_EGRESS_PRIORITY,
    LAN_FLOW_MARK_INGRESS_PRIORITY, MAP_PATHS,
};

pub fn attach_match_flow(
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

    if !has_mac {
        open_skel.maps.rodata_data.current_eth_net_offset = 0;
    }

    let skel = open_skel.load()?;
    let flow_ingress = skel.progs.flow_ingress;
    let flow_egress = skel.progs.flow_egress;

    let mut flow_ingress_hook =
        TcHookProxy::new(&flow_ingress, ifindex as i32, TC_INGRESS, LAN_FLOW_MARK_INGRESS_PRIORITY);

    let mut flow_egress_hook =
        TcHookProxy::new(&flow_egress, ifindex as i32, TC_EGRESS, LAN_FLOW_MARK_EGRESS_PRIORITY);

    flow_ingress_hook.attach();
    flow_egress_hook.attach();
    let _ = service_status.blocking_recv();
    drop(flow_ingress_hook);
    drop(flow_egress_hook);
    Ok(())
}
