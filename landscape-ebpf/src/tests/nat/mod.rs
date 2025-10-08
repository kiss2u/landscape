use std::{mem::MaybeUninit, net::IpAddr};

use libbpf_rs::{
    skel::{OpenSkel, SkelBuilder as _},
    MapCore, MapFlags, ProgramInput,
};

use crate::nat::{land_nat::LandNatSkelBuilder, v2::land_nat_v2::LandNatV2SkelBuilder};

use crate::{
    nat::v2::land_nat_v2::types::{u_inet_addr, wan_ip_info_key, wan_ip_info_value},
    LANDSCAPE_IPV4_TYPE, LANDSCAPE_IPV6_TYPE,
};

mod ipv6_egress;
mod ipv6_ingress;
mod package;

pub fn test_nat(mut payload: Vec<u8>) {
    let landscape_builder = LandNatSkelBuilder::default();
    let mut open_object = MaybeUninit::uninit();
    let landscape_open = landscape_builder.open(&mut open_object).unwrap();

    let landscape_skel = landscape_open.load().unwrap();

    let test_nat_read = landscape_skel.progs.test_nat_read;
    let repeat = 1000;

    let input = ProgramInput {
        data_in: Some(&mut payload),
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

pub fn test_nat_v2(mut payload: Vec<u8>) {
    let landscape_builder = LandNatV2SkelBuilder::default();
    let mut open_object = MaybeUninit::uninit();
    let landscape_open = landscape_builder.open(&mut open_object).unwrap();

    let landscape_skel = landscape_open.load().unwrap();

    let test_nat_read = landscape_skel.progs.test_nat_read;
    let repeat = 1000;

    let input = ProgramInput {
        data_in: Some(&mut payload),
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

fn add_wan_ip<'obj, T>(
    wan_ipv4_binding: &T,
    ifindex: u32,
    addr: IpAddr,
    gateway: Option<IpAddr>,
    mask: u8,
) where
    T: MapCore,
{
    let mut key = wan_ip_info_key::default();
    let mut value = wan_ip_info_value::default();
    key.ifindex = ifindex;
    value.mask = mask;

    match addr {
        std::net::IpAddr::V4(ipv4_addr) => {
            value.addr.ip = ipv4_addr.to_bits().to_be();
            key.l3_protocol = LANDSCAPE_IPV4_TYPE;
        }
        std::net::IpAddr::V6(ipv6_addr) => {
            value.addr = u_inet_addr { bits: ipv6_addr.to_bits().to_be_bytes() };
            key.l3_protocol = LANDSCAPE_IPV6_TYPE;
        }
    };

    match gateway {
        Some(std::net::IpAddr::V4(ipv4_addr)) => {
            value.gateway.ip = ipv4_addr.to_bits().to_be();
        }
        Some(std::net::IpAddr::V6(ipv6_addr)) => {
            value.gateway = u_inet_addr { bits: ipv6_addr.to_bits().to_be_bytes() };
        }
        None => {}
    };

    let key = unsafe { plain::as_bytes(&key) };
    let value = unsafe { plain::as_bytes(&value) };

    if let Err(e) = wan_ipv4_binding.update(key, value, MapFlags::ANY) {
        tracing::error!("setting wan ip error:{e:?}");
    } else {
        tracing::info!("setting wan index: {ifindex:?} addr:{addr:?}");
    }
}

#[cfg(test)]
pub mod tests {
    use crate::tests::nat::{test_nat, test_nat_v2};

    // cargo test --package landscape-ebpf --lib -- tests::test_nat::tests::test --exact --show-output
    #[test]
    fn test() {
        test_nat(crate::tests::dummpy_tcp_pkg());
        test_nat_v2(crate::tests::dummpy_tcp_pkg());
    }
}
