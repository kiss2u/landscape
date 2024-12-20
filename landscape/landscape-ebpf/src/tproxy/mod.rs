mod landscape_tproxy {
    include!(concat!(env!("OUT_DIR"), "/tproxy.skel.rs"));
}

use std::mem::MaybeUninit;
use std::net::Ipv4Addr;
use std::os::unix::io::AsFd as _;
use std::str::FromStr;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use libbpf_rs::skel::OpenSkel;
use libbpf_rs::skel::SkelBuilder;
use libbpf_rs::ErrorExt;
use libbpf_rs::TcHookBuilder;
use libbpf_rs::TC_EGRESS;
use libbpf_rs::TC_INGRESS;

use landscape_tproxy::*;

pub fn run() {
    // Install Ctrl-C handler
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .unwrap();

    let proxy_addr = Ipv4Addr::from_str("127.0.0.1").unwrap();
    let proxy_addr: u32 = proxy_addr.into();

    let target_addr = Ipv4Addr::from_str("223.6.6.6").unwrap();
    let target_addr: u32 = target_addr.into();

    let mut skel_builder = TproxySkelBuilder::default();
    skel_builder.obj_builder.debug(true);

    // Set constants
    let mut open_object = MaybeUninit::uninit();
    let open_skel = skel_builder.open(&mut open_object).unwrap();
    open_skel.maps.rodata_data.target_addr = target_addr.to_be();
    open_skel.maps.rodata_data.target_port = (2234 as u16).to_be();
    open_skel.maps.rodata_data.proxy_addr = proxy_addr.to_be();
    open_skel.maps.rodata_data.proxy_port = (12345 as u16).to_be();

    // Load into kernel
    let mut skel = open_skel.load().unwrap();

    // Set up and attach ingress TC hook
    let mut ingress = TcHookBuilder::new(skel.progs.tproxy.as_fd())
        .ifindex(4)
        .replace(true)
        .handle(1)
        .priority(1)
        .hook(TC_INGRESS);
    ingress.create().context("Failed to create ingress TC qdisc").unwrap();
    ingress.attach().context("Failed to attach ingress TC prog").unwrap();

    // Block until SIGINT
    while running.load(Ordering::SeqCst) {
        sleep(Duration::new(1, 0));
    }

    if let Err(e) = ingress.detach() {
        eprintln!("Failed to detach prog: {e}");
    }
    if let Err(e) = ingress.destroy() {
        eprintln!("Failed to destroy TC hook: {e}");
    }
}
