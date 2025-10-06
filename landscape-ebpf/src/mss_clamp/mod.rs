pub(crate) mod mss_clamp {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/bpf_rs/mss_clamp.skel.rs"));
}

use std::mem::MaybeUninit;

use libbpf_rs::{
    skel::{OpenSkel, SkelBuilder},
    TC_EGRESS, TC_INGRESS,
};
use mss_clamp::*;
use tokio::sync::oneshot;

use crate::{landscape::TcHookProxy, MSS_CLAMP_EGRESS_PRIORITY, MSS_CLAMP_INGRESS_PRIORITY};

pub fn run_mss_clamp(
    ifindex: i32,
    mtu_size: u16,
    has_mac: bool,
    service_status: oneshot::Receiver<()>,
) {
    let landscape_builder = MssClampSkelBuilder::default();

    let mut open_object = MaybeUninit::uninit();
    let mut landscape_open = landscape_builder.open(&mut open_object).unwrap();

    let rodata_data =
        landscape_open.maps.rodata_data.as_deref_mut().expect("`rodata` is not memery mapped");
    if !has_mac {
        rodata_data.current_l3_offset = 0;
    }

    rodata_data.mtu_size = mtu_size;
    let landscape_skel = landscape_open.load().unwrap();

    let mss_clamp_egress = landscape_skel.progs.clamp_egress;
    let mss_clamp_ingress = landscape_skel.progs.clamp_ingress;

    let mut mss_clamp_egress_hook =
        TcHookProxy::new(&mss_clamp_egress, ifindex, TC_EGRESS, MSS_CLAMP_EGRESS_PRIORITY);
    let mut mss_clamp_ingress_hook =
        TcHookProxy::new(&mss_clamp_ingress, ifindex, TC_INGRESS, MSS_CLAMP_INGRESS_PRIORITY);

    mss_clamp_egress_hook.attach();
    mss_clamp_ingress_hook.attach();

    let _ = service_status.blocking_recv();

    drop(mss_clamp_egress_hook);
    drop(mss_clamp_ingress_hook);
}
