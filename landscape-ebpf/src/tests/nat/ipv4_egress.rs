use std::{
    mem::MaybeUninit,
    net::{IpAddr, Ipv6Addr},
    str::FromStr,
};

use libbpf_rs::{
    skel::{OpenSkel, SkelBuilder as _},
    ProgramInput,
};
use zerocopy::IntoBytes;

use crate::{map_setting::add_wan_ip, nat::v2::land_nat_v2::LandNatV2SkelBuilder, tests::TestSkb};

pub fn handle_ipv4_egress(mut payload: Vec<u8>) {
    let landscape_builder = LandNatV2SkelBuilder::default();
    let mut open_object = MaybeUninit::uninit();
    let landscape_open = landscape_builder.open(&mut open_object).unwrap();

    let landscape_skel = landscape_open.load().unwrap();

    let ifindex = 6;

    add_wan_ip(
        &landscape_skel.maps.wan_ip_binding,
        ifindex,
        IpAddr::V6(Ipv6Addr::from_str("2409:8888:6666:4f21::").unwrap()),
        None,
        60,
    );
    let handle_ipv4_egress = landscape_skel.progs.egress_nat;

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
    let result = handle_ipv4_egress.test_run(input).expect("test_run failed");

    println!("return_value = {}", result.return_value as i32);
    println!("duration = {:?}", result.duration);
    println!("packet_out = {:?}", packet_out);
    crate::tests::check::analyze(&packet_out);
}

#[cfg(test)]
pub mod tests {
    use crate::tests::nat::ipv4_egress::handle_ipv4_egress;
    use crate::tests::nat::package::*;

    #[test]
    fn tcp_syn() {
        handle_ipv4_egress(ipv4_tcp_syn());
    }
}
