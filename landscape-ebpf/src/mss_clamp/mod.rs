pub(crate) mod mss_clamp {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/bpf_rs/mss_clamp.skel.rs"));
}

use libbpf_rs::{
    skel::{OpenSkel, SkelBuilder},
    TC_EGRESS, TC_INGRESS,
};
use mss_clamp::*;

use crate::{
    bpf_error::LdEbpfResult,
    landscape::{OwnedOpenObject, TcHookProxy},
    MSS_CLAMP_EGRESS_PRIORITY, MSS_CLAMP_INGRESS_PRIORITY,
};

pub struct MssClampHandle {
    _backing: OwnedOpenObject,
    skel: Option<MssClampSkel<'static>>,
    ingress_hook: Option<TcHookProxy>,
    egress_hook: Option<TcHookProxy>,
}

unsafe impl Send for MssClampHandle {}
unsafe impl Sync for MssClampHandle {}

impl MssClampHandle {
    pub fn skel(&self) -> &MssClampSkel<'static> {
        self.skel.as_ref().expect("mss clamp skeleton missing")
    }

    pub fn skel_mut(&mut self) -> &mut MssClampSkel<'static> {
        self.skel.as_mut().expect("mss clamp skeleton missing")
    }
}

impl Drop for MssClampHandle {
    fn drop(&mut self) {
        self.ingress_hook.take();
        self.egress_hook.take();
        self.skel.take();
    }
}

pub fn run_mss_clamp(ifindex: i32, mtu_size: u16, has_mac: bool) -> LdEbpfResult<MssClampHandle> {
    let landscape_builder = MssClampSkelBuilder::default();
    let (backing, open_object) = OwnedOpenObject::new();
    let mut landscape_open =
        crate::bpf_ctx!(landscape_builder.open(open_object), "mss_clamp open skeleton failed")?;

    let rodata_data =
        landscape_open.maps.rodata_data.as_deref_mut().expect("`rodata` is not memery mapped");
    if !has_mac {
        rodata_data.current_l3_offset = 0;
    }

    rodata_data.mtu_size = mtu_size;
    let landscape_skel = crate::bpf_ctx!(landscape_open.load(), "mss_clamp load skeleton failed")?;

    let mut mss_clamp_egress_hook = TcHookProxy::new(
        &landscape_skel.progs.clamp_egress,
        ifindex,
        TC_EGRESS,
        MSS_CLAMP_EGRESS_PRIORITY,
    );
    let mut mss_clamp_ingress_hook = TcHookProxy::new(
        &landscape_skel.progs.clamp_ingress,
        ifindex,
        TC_INGRESS,
        MSS_CLAMP_INGRESS_PRIORITY,
    );

    mss_clamp_egress_hook.attach();
    mss_clamp_ingress_hook.attach();

    Ok(MssClampHandle {
        _backing: backing,
        skel: Some(landscape_skel),
        ingress_hook: Some(mss_clamp_ingress_hook),
        egress_hook: Some(mss_clamp_egress_hook),
    })
}
