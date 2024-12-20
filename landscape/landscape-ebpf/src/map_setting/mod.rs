use std::{mem::MaybeUninit, net::Ipv4Addr, path::PathBuf};

use libbpf_rs::{
    skel::{OpenSkel, SkelBuilder},
    MapCore, MapFlags,
};
mod share_map {
    include!(concat!(env!("OUT_DIR"), "/share_map.skel.rs"));
}

use share_map::*;
use types::{ipv4_lpm_key, ipv4_mark_action};

use crate::{BLOCK_IP_MAP_PING_PATH, PACKET_MARK_MAP_PING_PATH, WAN_IP_MAP_PING_PATH};

pub fn add_wan_ip(ifindex: u32, addr: Ipv4Addr) {
    println!("setting wan index - 1: {ifindex:?} addr:{addr:?}");
    let mut landscape_builder = ShareMapSkelBuilder::default();
    landscape_builder.obj_builder.debug(true);
    let mut open_object = MaybeUninit::uninit();
    let mut landscape_open = landscape_builder.open(&mut open_object).unwrap();
    // landscape_open.maps.wan_ipv4_binding.set_autocreate(autocreate)
    if let Err(e) =
        landscape_open.maps.wan_ipv4_binding.reuse_pinned_map(PathBuf::from(WAN_IP_MAP_PING_PATH))
    {
        println!("error: {e:?}");
    }
    landscape_open.maps.wan_ipv4_binding.set_pin_path(PathBuf::from(WAN_IP_MAP_PING_PATH)).unwrap();
    let landscape_skel = landscape_open.load().unwrap();
    // if !landscape_skel.maps.wan_ipv4_binding.is_pinned() {
    //     if let Err(e) =
    //         landscape_skel.maps.wan_ipv4_binding.pin(PathBuf::from(WAN_IP_MAP_PING_PATH))
    //     {
    //         println!("pin error: {e:?}");
    //     }
    // }
    let addr: u32 = addr.into();
    if let Err(e) = landscape_skel.maps.wan_ipv4_binding.update(
        &ifindex.to_le_bytes(),
        &addr.to_be_bytes(),
        MapFlags::ANY,
    ) {
        println!("setting wan ip error:{e:?}");
    } else {
        println!("setting wan index: {ifindex:?} addr:{addr:?}");
    }
}

pub fn del_wan_ip(ifindex: u32) {
    println!("del wan index - 1: {ifindex:?}");
    let mut landscape_builder = ShareMapSkelBuilder::default();
    landscape_builder.obj_builder.debug(true);
    let mut open_object = MaybeUninit::uninit();
    let mut landscape_open = landscape_builder.open(&mut open_object).unwrap();
    if let Err(e) =
        landscape_open.maps.wan_ipv4_binding.set_pin_path(PathBuf::from(WAN_IP_MAP_PING_PATH))
    {
        println!("error: {e:?}");
    }
    let landscape_skel = landscape_open.load().unwrap();
    if let Err(e) = landscape_skel.maps.wan_ipv4_binding.delete(&ifindex.to_le_bytes()) {
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
    let landscape_builder = ShareMapSkelBuilder::default();
    // landscape_builder.obj_builder.debug(true);
    let mut open_object = MaybeUninit::uninit();
    let mut landscape_open = landscape_builder.open(&mut open_object).unwrap();
    if let Err(e) =
        landscape_open.maps.packet_mark_map.set_pin_path(PathBuf::from(PACKET_MARK_MAP_PING_PATH))
    {
        println!("error: {e:?}");
    }
    let landscape_skel = landscape_open.load().unwrap();
    for (ip, prefixlen) in ips.into_iter() {
        let addr: u32 = ip.into();
        let data = ipv4_lpm_key { prefixlen, addr: addr.to_be() };
        let block_ip_key = unsafe { plain::as_bytes(&data) };
        let mark_action = ipv4_mark_action { mark };
        let mark_action = unsafe { plain::as_bytes(&mark_action) };
        if let Err(e) =
            landscape_skel.maps.packet_mark_map.update(block_ip_key, mark_action, MapFlags::ANY)
        {
            println!("add block ip error:{e:?}");
        }
    }
}

pub fn del_ips_mark(ips: Vec<(Ipv4Addr, u32)>) {
    let landscape_builder = ShareMapSkelBuilder::default();
    // landscape_builder.obj_builder.debug(true);
    let mut open_object = MaybeUninit::uninit();
    let mut landscape_open = landscape_builder.open(&mut open_object).unwrap();
    if let Err(e) =
        landscape_open.maps.packet_mark_map.set_pin_path(PathBuf::from(PACKET_MARK_MAP_PING_PATH))
    {
        println!("error: {e:?}");
    }
    let landscape_skel = landscape_open.load().unwrap();
    for (ip, prefixlen) in ips.into_iter() {
        let addr: u32 = ip.into();
        let data = ipv4_lpm_key { prefixlen, addr: addr.to_be() };
        let block_ip_key = unsafe { plain::as_bytes(&data) };
        if let Err(e) = landscape_skel.maps.packet_mark_map.delete(block_ip_key) {
            println!("add block ip error:{e:?}");
        }
    }
}
