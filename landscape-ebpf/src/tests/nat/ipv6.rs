use std::{
    mem::MaybeUninit,
    net::{IpAddr, Ipv6Addr},
    str::FromStr,
};

use libbpf_rs::{
    skel::{OpenSkel, SkelBuilder as _},
    MapCore, MapFlags, ProgramInput,
};
use zerocopy::IntoBytes;

use crate::{
    nat::v2::land_nat_v2::{
        types::{u_inet_addr, wan_ip_info_key, wan_ip_info_value},
        LandNatV2SkelBuilder,
    },
    tests::TestSkb,
    LANDSCAPE_IPV4_TYPE, LANDSCAPE_IPV6_TYPE,
};

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

pub fn handle_ipv6_egress(mut payload: Vec<u8>) {
    let landscape_builder = LandNatV2SkelBuilder::default();
    let mut open_object = MaybeUninit::uninit();
    let landscape_open = landscape_builder.open(&mut open_object).unwrap();

    let landscape_skel = landscape_open.load().unwrap();

    let ifindex = 6;

    add_wan_ip(
        &landscape_skel.maps.wan_ipv4_binding,
        ifindex,
        IpAddr::V6(Ipv6Addr::from_str("2409:8888:6666:4f21::").unwrap()),
        None,
        60,
    );
    let handle_ipv6_egress = landscape_skel.progs.handle_ipv6_egress;

    let mut ctx = TestSkb::default();
    ctx.ifindex = ifindex;

    let mut packet_out = vec![0 as u8; payload.len()];
    let input = ProgramInput {
        data_in: Some(&mut payload),
        context_in: Some(ctx.as_mut_bytes()),
        context_out: None,
        data_out: Some(&mut packet_out),
        ..Default::default()
    };
    let result = handle_ipv6_egress.test_run(input).expect("test_run failed");

    println!("return_value = {}", result.return_value as i32);
    println!("duration = {:?}", result.duration);
    println!("packet_out = {:?}", packet_out);
    crate::tests::check::analyze(&packet_out);
}

#[cfg(test)]
pub mod tests {
    use crate::tests::nat::ipv6::handle_ipv6_egress;
    use crate::tests::nat::package::*;

    #[test]
    fn ipv6_icmp_too_big() {
        handle_ipv6_egress(build_ipv6_icmp_too_big());
    }

    #[test]
    fn ipv6_icmp_request() {
        handle_ipv6_egress(build_ipv6_icmp_request());
    }
}
