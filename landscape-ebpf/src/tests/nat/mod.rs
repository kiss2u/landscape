use std::{
    mem::MaybeUninit,
    net::{IpAddr, Ipv4Addr},
};

use landscape_common::net::MacAddr;
use libbpf_rs::{
    skel::{OpenSkel, SkelBuilder as _},
    ProgramInput,
};

use crate::nat::{
    land_nat::LandNatSkelBuilder, v2::land_nat_v2::LandNatV2SkelBuilder,
    v3::land_nat_v3::LandNatV3SkelBuilder,
};

mod ipv4_egress;

mod ipv6_egress;
mod ipv6_ingress;
mod package;

pub fn test_nat(mut syn_data: Vec<u8>, tcp_data: Vec<u8>) {
    let landscape_builder = LandNatSkelBuilder::default();
    let mut open_object = MaybeUninit::uninit();
    let landscape_open = landscape_builder.open(&mut open_object).unwrap();

    let landscape_skel = landscape_open.load().unwrap();

    let ifindex = 1;
    crate::map_setting::add_wan_ip(
        &landscape_skel.maps.wan_ip_binding,
        ifindex,
        IpAddr::V4(Ipv4Addr::new(192, 168, 101, 201)),
        None,
        24,
        Some(MacAddr::broadcast()),
    );

    let test_nat_read = landscape_skel.progs.egress_nat;

    let input = ProgramInput {
        data_in: Some(&mut syn_data),
        context_in: None,
        context_out: None,
        data_out: None,
        ..Default::default()
    };
    let result = test_nat_read.test_run(input).expect("test_run failed");

    println!("return_value = {}", result.return_value as i32);
    println!("duration = {:?}", result.duration);
    let repeat = 1_000;
    let input = ProgramInput {
        data_in: Some(&tcp_data),
        context_in: None,
        context_out: None,
        data_out: None,
        repeat,
        ..Default::default()
    };
    let result = test_nat_read.test_run(input).expect("test_run failed");

    println!("return_value = {}", result.return_value as i32);
    println!("duration = {:?}", result.duration);
}

pub fn test_nat_v2(mut syn_data: Vec<u8>, tcp_data: Vec<u8>) {
    let landscape_builder = LandNatV2SkelBuilder::default();
    let mut open_object = MaybeUninit::uninit();
    let landscape_open = landscape_builder.open(&mut open_object).unwrap();

    let landscape_skel = landscape_open.load().unwrap();
    let ifindex = 1;
    crate::map_setting::add_wan_ip(
        &landscape_skel.maps.wan_ip_binding,
        ifindex,
        IpAddr::V4(Ipv4Addr::new(192, 168, 101, 201)),
        None,
        24,
        Some(MacAddr::broadcast()),
    );

    let test_nat_read = landscape_skel.progs.egress_nat;

    let input = ProgramInput {
        data_in: Some(&mut syn_data),
        context_in: None,
        context_out: None,
        data_out: None,
        ..Default::default()
    };
    let result = test_nat_read.test_run(input).expect("test_run failed");

    println!("return_value = {}", result.return_value as i32);
    println!("duration = {:?}", result.duration);
    let repeat = 1_000;
    let input = ProgramInput {
        data_in: Some(&tcp_data),
        context_in: None,
        context_out: None,
        data_out: None,
        repeat,
        ..Default::default()
    };
    let result = test_nat_read.test_run(input).expect("test_run failed");

    println!("return_value = {}", result.return_value as i32);
    println!("duration = {:?}", result.duration);
}

pub fn test_nat_v3(mut syn_data: Vec<u8>, tcp_data: Vec<u8>) {
    let landscape_builder = LandNatV3SkelBuilder::default();
    let mut open_object = MaybeUninit::uninit();
    let landscape_open = landscape_builder.open(&mut open_object).unwrap();

    let landscape_skel = landscape_open.load().unwrap();

    let ifindex = 1;
    crate::map_setting::add_wan_ip(
        &landscape_skel.maps.wan_ip_binding,
        ifindex,
        IpAddr::V4(Ipv4Addr::new(192, 168, 101, 201)),
        None,
        24,
        Some(MacAddr::broadcast()),
    );

    let test_nat_read = landscape_skel.progs.egress_nat;

    let input = ProgramInput {
        data_in: Some(&mut syn_data),
        context_in: None,
        context_out: None,
        data_out: None,
        ..Default::default()
    };
    let result = test_nat_read.test_run(input).expect("test_run failed");

    println!("return_value = {}", result.return_value as i32);
    println!("duration = {:?}", result.duration);
    let repeat = 1_000;
    let input = ProgramInput {
        data_in: Some(&tcp_data),
        context_in: None,
        context_out: None,
        data_out: None,
        repeat,
        ..Default::default()
    };
    let result = test_nat_read.test_run(input).expect("test_run failed");

    println!("return_value = {}", result.return_value as i32);
    println!("duration = {:?}", result.duration);
}

#[cfg(test)]
pub mod tests {
    use crate::tests::nat::*;

    // cargo test --package landscape-ebpf --lib -- tests::test_nat::tests::test --exact --show-output
    #[test]
    fn test() {
        test_nat(super::package::ipv4_tcp_syn(), super::package::ipv4_tcp_data());
        test_nat_v2(super::package::ipv4_tcp_syn(), super::package::ipv4_tcp_data());
        test_nat_v3(super::package::ipv4_tcp_syn(), super::package::ipv4_tcp_data());
    }

    #[test]
    fn test2() {
        test_nat_v3(super::package::ipv4_tcp_syn(), super::package::ipv4_tcp_data());
    }
}
