mod ns_proxy {
    include!(concat!(env!("OUT_DIR"), "/ns_proxy.skel.rs"));
}
use std::{
    fs::File,
    mem::MaybeUninit,
    net::Ipv4Addr,
    os::fd::{AsFd, AsRawFd},
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, sleep},
    time::Duration,
};

use libbpf_rs::{
    skel::{OpenSkel, SkelBuilder},
    TC_EGRESS, TC_INGRESS,
};
use nix::sched;
use ns_proxy::*;

use crate::landscape::TcHookProxy;
fn bump_memlock_rlimit() {
    let rlimit = libc::rlimit { rlim_cur: 128 << 20, rlim_max: 128 << 20 };

    if unsafe { libc::setrlimit(libc::RLIMIT_MEMLOCK, &rlimit) } != 0 {
        panic!("Failed to increase rlimit");
    }
}

pub fn run() {
    bump_memlock_rlimit();

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .unwrap();

    let mut landscape_builder = NsProxySkelBuilder::default();
    landscape_builder.obj_builder.debug(true);

    let target_addr = Ipv4Addr::from_str("223.6.6.6").unwrap();
    let target_addr: u32 = target_addr.into();

    let proxy_addr = Ipv4Addr::from_str("127.0.0.1").unwrap();
    let proxy_addr: u32 = proxy_addr.into();

    let mut open_object = MaybeUninit::uninit();
    let landscape_open = landscape_builder.open(&mut open_object).unwrap();
    landscape_open.maps.rodata_data.outer_ifindex = 7;
    landscape_open.maps.rodata_data.target_addr = target_addr.to_be();

    landscape_open.maps.rodata_data.proxy_addr = proxy_addr.to_be();
    landscape_open.maps.rodata_data.proxy_port = (12345 as u16).to_be();

    let mut landscape_skel = landscape_open.load().unwrap();

    let ns_ingress = landscape_skel.progs.ns_ingress;
    let ns_peer_ingress = landscape_skel.progs.ns_peer_ingress;
    let wan_egress = landscape_skel.progs.wan_egress;

    let current_ns = File::open("/proc/self/ns/net").unwrap();

    let mut ns_inner_ingress_hook = TcHookProxy::new(&ns_ingress, 6, TC_INGRESS, 2);
    let mut ns_outer_ingress_hook = TcHookProxy::new(&ns_peer_ingress, 7, TC_INGRESS, 2);
    let mut wan_egress_hook = TcHookProxy::new(&wan_egress, 2, TC_EGRESS, 2);

    wan_egress_hook.attach();
    ns_outer_ingress_hook.attach();
    enter_namespace("/var/run/netns/tpns").unwrap();
    ns_inner_ingress_hook.attach();
    sched::setns(&current_ns, sched::CloneFlags::CLONE_NEWNET).unwrap();
    while running.load(Ordering::SeqCst) {
        sleep(Duration::new(1, 0));
    }

    enter_namespace("/var/run/netns/tpns").unwrap();
    drop(ns_inner_ingress_hook);
    sched::setns(&current_ns, sched::CloneFlags::CLONE_NEWNET).unwrap();
    drop(ns_outer_ingress_hook);
    drop(wan_egress_hook);
}

fn enter_namespace(ns_path: &str) -> nix::Result<()> {
    let ns_file = File::open(ns_path).unwrap();
    sched::setns(ns_file, sched::CloneFlags::CLONE_NEWNET)?;
    Ok(())
}
