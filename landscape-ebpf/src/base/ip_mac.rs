use std::mem::MaybeUninit;

pub(crate) mod neigh_update {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/bpf_rs/neigh_update.skel.rs"));
}

use libbpf_rs::{
    skel::{OpenSkel, SkelBuilder},
    TracepointCategory,
};
use neigh_update::*;
use tokio::sync::oneshot;

use crate::{bpf_error::LdEbpfResult, MAP_PATHS};

pub fn neigh_update(service_status: oneshot::Receiver<()>) -> LdEbpfResult<()> {
    let mut open_object = MaybeUninit::zeroed();
    let builder = NeighUpdateSkelBuilder::default();
    let mut open_skel = builder.open(&mut open_object)?;

    open_skel.maps.ip_mac_v4.set_pin_path(&MAP_PATHS.ip_mac_v4).unwrap();
    open_skel.maps.ip_mac_v4.reuse_pinned_map(&MAP_PATHS.ip_mac_v4).unwrap();

    open_skel.maps.ip_mac_v6.set_pin_path(&MAP_PATHS.ip_mac_v6).unwrap();
    open_skel.maps.ip_mac_v6.reuse_pinned_map(&MAP_PATHS.ip_mac_v6).unwrap();

    let skel = open_skel.load()?;
    let trace_neigh_update = skel.progs.trace_neigh_update;

    let _link =
        trace_neigh_update.attach_tracepoint(TracepointCategory::Neigh, "neigh_update").unwrap();

    let _ = service_status.blocking_recv();
    Ok(())
}
