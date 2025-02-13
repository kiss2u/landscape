use std::{mem::MaybeUninit, net::Ipv4Addr};

use libbpf_rs::{
    skel::{OpenSkel, SkelBuilder},
    MapCore, MapFlags,
};

mod share_map {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/bpf_rs/share_map.skel.rs"));
}

use share_map::*;
use types::{ipv4_lpm_key, ipv4_mark_action};

use crate::{LandscapeMapPath, MAP_PATHS};

pub(crate) fn init_path(paths: LandscapeMapPath) {
    let mut landscape_builder = ShareMapSkelBuilder::default();
    landscape_builder.obj_builder.debug(true);
    let mut open_object = MaybeUninit::uninit();
    let mut landscape_open = landscape_builder.open(&mut open_object).unwrap();

    landscape_open.maps.wan_ipv4_binding.set_pin_path(&paths.wan_ip).unwrap();
    landscape_open.maps.packet_mark_map.set_pin_path(&paths.packet_mark).unwrap();
    landscape_open.maps.redirect_index_map.set_pin_path(&paths.redirect_index).unwrap();
    let _landscape_skel = landscape_open.load().unwrap();
}

pub fn add_wan_ip(ifindex: u32, addr: Ipv4Addr) {
    println!("add wan index - 1: {ifindex:?}");
    let wan_ipv4_binding = libbpf_rs::MapHandle::from_pinned_path(&MAP_PATHS.wan_ip).unwrap();

    let addr: u32 = addr.into();
    if let Err(e) =
        wan_ipv4_binding.update(&ifindex.to_le_bytes(), &addr.to_be_bytes(), MapFlags::ANY)
    {
        println!("setting wan ip error:{e:?}");
    } else {
        println!("setting wan index: {ifindex:?} addr:{addr:?}");
    }
}

pub fn del_wan_ip(ifindex: u32) {
    println!("del wan index - 1: {ifindex:?}");
    let wan_ipv4_binding = libbpf_rs::MapHandle::from_pinned_path(&MAP_PATHS.wan_ip).unwrap();
    if let Err(e) = wan_ipv4_binding.delete(&ifindex.to_le_bytes()) {
        println!("delete wan ip error:{e:?}");
    } else {
        println!("delete wan index: {ifindex:?}");
    }
}

// const BLOCK_DEFAULT_VALUE: u32 = 0;
// pub fn add_block_ip(ips: Vec<(Ipv4Addr, u32)>) {
//     let landscape_builder = ShareMapSkelBuilder::default();
//     // landscape_builder.obj_builder.debug(true);
//     let mut open_object = MaybeUninit::uninit();
//     let mut landscape_open = landscape_builder.open(&mut open_object).unwrap();
//     if let Err(e) =
//         landscape_open.maps.firewall_block_map.set_pin_path(PathBuf::from(BLOCK_IP_MAP_PING_PATH))
//     {
//         println!("error: {e:?}");
//     }
//     let landscape_skel = landscape_open.load().unwrap();
//     for (ip, prefixlen) in ips.into_iter() {
//         let addr: u32 = ip.into();
//         let data = ipv4_lpm_key { prefixlen, addr: addr.to_be() };
//         let block_ip_key = unsafe { plain::as_bytes(&data) };
//         if let Err(e) = landscape_skel.maps.firewall_block_map.update(
//             block_ip_key,
//             &BLOCK_DEFAULT_VALUE.octets(),
//             MapFlags::ANY,
//         ) {
//             println!("add block ip error:{e:?}");
//         }
//     }
// }

// pub fn del_block_ip(ips: Vec<(Ipv4Addr, u32)>) {
//     let landscape_builder = ShareMapSkelBuilder::default();
//     // landscape_builder.obj_builder.debug(true);
//     let mut open_object = MaybeUninit::uninit();
//     let mut landscape_open = landscape_builder.open(&mut open_object).unwrap();
//     if let Err(e) =
//         landscape_open.maps.firewall_block_map.set_pin_path(PathBuf::from(BLOCK_IP_MAP_PING_PATH))
//     {
//         println!("error: {e:?}");
//     }
//     let landscape_skel = landscape_open.load().unwrap();
//     for (ip, prefixlen) in ips.into_iter() {
//         let addr: u32 = ip.into();
//         let data = ipv4_lpm_key { prefixlen, addr: addr.to_be() };
//         let block_ip_key = unsafe { plain::as_bytes(&data) };
//         if let Err(e) = landscape_skel.maps.firewall_block_map.delete(block_ip_key) {
//             println!("add block ip error:{e:?}");
//         }
//     }
// }

pub fn add_ips_mark(ips: Vec<(Ipv4Addr, u32)>, mark: u32) {
    let packet_mark_map = libbpf_rs::MapHandle::from_pinned_path(&MAP_PATHS.packet_mark).unwrap();

    for (ip, prefixlen) in ips.into_iter() {
        let addr: u32 = ip.into();
        let data = ipv4_lpm_key { prefixlen, addr: addr.to_be() };
        let block_ip_key = unsafe { plain::as_bytes(&data) };
        let mark_action = ipv4_mark_action { mark };
        let mark_action = unsafe { plain::as_bytes(&mark_action) };
        if let Err(e) = packet_mark_map.update(block_ip_key, mark_action, MapFlags::ANY) {
            println!("add block ip error:{e:?}");
        }
    }
}

pub fn del_ips_mark(ips: Vec<(Ipv4Addr, u32)>) {
    let packet_mark_map = libbpf_rs::MapHandle::from_pinned_path(&MAP_PATHS.packet_mark).unwrap();

    for (ip, prefixlen) in ips.into_iter() {
        let addr: u32 = ip.into();
        let data = ipv4_lpm_key { prefixlen, addr: addr.to_be() };
        let block_ip_key = unsafe { plain::as_bytes(&data) };
        if let Err(e) = packet_mark_map.delete(block_ip_key) {
            println!("add block ip error:{e:?}");
        }
    }
}

pub fn add_redirect_iface_pair(redirect_index: u8, ifindex: u32) {
    let redirect_index_map =
        libbpf_rs::MapHandle::from_pinned_path(&MAP_PATHS.redirect_index).unwrap();
    let key = [redirect_index];
    let value = ifindex.to_le_bytes();
    if let Err(e) = redirect_index_map.update(&key, &value, MapFlags::ANY) {
        println!("add block ip error:{e:?}");
    }
}

pub fn del_redirect_iface_pair(redirect_index: u8) {
    let redirect_index_map =
        libbpf_rs::MapHandle::from_pinned_path(&MAP_PATHS.redirect_index).unwrap();

    let key = [redirect_index];
    if let Err(e) = redirect_index_map.delete(&key) {
        println!("add block ip error:{e:?}");
    }
}
