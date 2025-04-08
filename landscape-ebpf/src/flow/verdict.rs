use std::mem::MaybeUninit;

mod flow_verdict_bpf {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/bpf_rs/flow_verdict.skel.rs"));
}

use flow_verdict_bpf::*;
use libbpf_rs::{
    skel::{OpenSkel, SkelBuilder},
    TC_INGRESS,
};
use tokio::sync::oneshot;

use crate::{bpf_error::LdEbpfResult, landscape::TcHookProxy, MAP_PATHS, MARK_EGRESS_PRIORITY};

pub fn attach_verdict_flow(
    ifindex: u32,
    has_mac: bool,
    service_status: oneshot::Receiver<()>,
) -> LdEbpfResult<()> {
    let mut open_object = MaybeUninit::zeroed();
    let firewall_builder = FlowVerdictSkelBuilder::default();
    let mut open_skel = firewall_builder.open(&mut open_object)?;

    open_skel.maps.flow_v_dns_map.set_pin_path(&MAP_PATHS.flow_verdict_dns_map)?;
    open_skel.maps.flow_v_dns_map.reuse_pinned_map(&MAP_PATHS.flow_verdict_dns_map)?;

    open_skel.maps.flow_v_ip_map.set_pin_path(&MAP_PATHS.flow_verdict_ip_map)?;
    open_skel.maps.flow_v_ip_map.reuse_pinned_map(&MAP_PATHS.flow_verdict_ip_map)?;

    if !has_mac {
        open_skel.maps.rodata_data.current_eth_net_offset = 0;
    }

    let skel = open_skel.load()?;
    let flow_ingress = skel.progs.flow_ingress;
    let flow_egress = skel.progs.flow_egress;

    let mut flow_ingress_hook =
        TcHookProxy::new(&flow_ingress, ifindex as i32, TC_INGRESS, MARK_EGRESS_PRIORITY);

    let mut flow_egress_hook =
        TcHookProxy::new(&flow_egress, ifindex as i32, TC_INGRESS, MARK_EGRESS_PRIORITY);

    flow_ingress_hook.attach();
    flow_egress_hook.attach();
    let _ = service_status.blocking_recv();
    drop(flow_ingress_hook);
    drop(flow_egress_hook);
    Ok(())
}
