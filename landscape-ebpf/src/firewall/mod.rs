use libbpf_rs::{
    skel::{OpenSkel, SkelBuilder},
    TC_EGRESS, TC_INGRESS,
};

pub(crate) mod firewall_bpf {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/bpf_rs/firewall.skel.rs"));
}

use firewall_bpf::*;

use crate::{
    bpf_error::LdEbpfResult,
    landscape::{pin_and_reuse_map, OwnedOpenObject, TcHookProxy},
    FIREWALL_EGRESS_PRIORITY, FIREWALL_INGRESS_PRIORITY, MAP_PATHS,
};

pub struct FirewallHandle {
    _backing: OwnedOpenObject,
    skel: Option<FirewallSkel<'static>>,
    ingress_hook: Option<TcHookProxy>,
    egress_hook: Option<TcHookProxy>,
}

unsafe impl Send for FirewallHandle {}
unsafe impl Sync for FirewallHandle {}

impl FirewallHandle {
    pub fn skel(&self) -> &FirewallSkel<'static> {
        self.skel.as_ref().expect("firewall skeleton missing")
    }

    pub fn skel_mut(&mut self) -> &mut FirewallSkel<'static> {
        self.skel.as_mut().expect("firewall skeleton missing")
    }
}

impl Drop for FirewallHandle {
    fn drop(&mut self) {
        self.ingress_hook.take();
        self.egress_hook.take();
        self.skel.take();
    }
}

pub fn new_firewall(ifindex: i32, has_mac: bool) -> LdEbpfResult<FirewallHandle> {
    let firewall_builder = FirewallSkelBuilder::default();
    let (backing, open_object) = OwnedOpenObject::new();
    let mut open_skel =
        crate::bpf_ctx!(firewall_builder.open(open_object), "firewall open skeleton failed")?;
    let rodata_data =
        open_skel.maps.rodata_data.as_deref_mut().expect("`rodata` is not memery mapped");

    if !has_mac {
        rodata_data.current_l3_offset = 0;
    }

    crate::bpf_ctx!(
        pin_and_reuse_map(
            &mut open_skel.maps.firewall_block_ip4_map,
            &MAP_PATHS.firewall_ipv4_block
        ),
        "firewall prepare firewall_block_ip4_map failed"
    )?;
    crate::bpf_ctx!(
        pin_and_reuse_map(
            &mut open_skel.maps.firewall_block_ip6_map,
            &MAP_PATHS.firewall_ipv6_block
        ),
        "firewall prepare firewall_block_ip6_map failed"
    )?;
    crate::bpf_ctx!(
        pin_and_reuse_map(
            &mut open_skel.maps.firewall_conn_metric_events,
            &MAP_PATHS.firewall_conn_metric_events
        ),
        "firewall prepare firewall_conn_metric_events failed"
    )?;
    crate::bpf_ctx!(
        pin_and_reuse_map(
            &mut open_skel.maps.firewall_allow_rules_map,
            &MAP_PATHS.firewall_allow_rules_map
        ),
        "firewall prepare firewall_allow_rules_map failed"
    )?;

    let skel = crate::bpf_ctx!(open_skel.load(), "firewall load skeleton failed")?;

    let mut egress_firewall_hook =
        TcHookProxy::new(&skel.progs.egress_firewall, ifindex, TC_EGRESS, FIREWALL_EGRESS_PRIORITY);
    let mut ingress_firewall_hook = TcHookProxy::new(
        &skel.progs.ingress_firewall,
        ifindex,
        TC_INGRESS,
        FIREWALL_INGRESS_PRIORITY,
    );

    egress_firewall_hook.attach();
    ingress_firewall_hook.attach();

    Ok(FirewallHandle {
        _backing: backing,
        skel: Some(skel),
        ingress_hook: Some(ingress_firewall_hook),
        egress_hook: Some(egress_firewall_hook),
    })
}
