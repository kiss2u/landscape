use std::{mem::MaybeUninit, net::Ipv6Addr, str::FromStr};

use libbpf_rs::{
    skel::{OpenSkel, SkelBuilder as _},
    MapCore, MapFlags, ProgramInput,
};
use zerocopy::IntoBytes;

use crate::{
    nat::v2::land_nat_v2::{
        types::{ipv6_prefix_mapping_key, ipv6_prefix_mapping_value},
        LandNatV2SkelBuilder,
    },
    tests::TestSkb,
};

pub fn split_ipv6_addr(addr: Ipv6Addr) -> ([u8; 8], [u8; 9]) {
    let octets = addr.octets(); // 16 字节，大端序（高位在前）
    println!("octets: {octets:02x?}");

    // 高 64 bit（前 8 字节）
    let mut prefix = [0u8; 8];
    prefix.copy_from_slice(&octets[0..8]);

    // 低 68 bit = 后 8 字节 + 4 bit（半字节）
    let mut suffix = [0u8; 9];
    // 先拷贝低 8 字节
    suffix[1..9].copy_from_slice(&octets[8..16]);

    suffix[0] = octets[7] & 0x0F;

    (prefix, suffix)
}

fn add_ipv6_mapping<'obj, T>(
    mapping_map: &T,
    client_port: u16,
    fake_prefix: Ipv6Addr,
    fake_suffix: Ipv6Addr,
    l4_protocol: u8,
) where
    T: MapCore,
{
    let mut key = ipv6_prefix_mapping_key::default();
    let mut value = ipv6_prefix_mapping_value::default();
    let (prefix, _) = split_ipv6_addr(fake_prefix);
    let (_, suffix) = split_ipv6_addr(fake_suffix);
    println!("prefix: {prefix:02x?}, suffix: {suffix:02x?}");
    key.id_byte = suffix[0];
    key.client_port = client_port.to_be();
    let mut fixed_array = [0u8; 8];
    fixed_array.copy_from_slice(&suffix[1..9]);
    key.client_suffix = fixed_array;
    key.l4_protocol = l4_protocol;

    value.client_prefix = prefix;
    value.is_allow_reuse = 1;
    println!("key: {key:?}");

    let key = unsafe { plain::as_bytes(&key) };
    let value = unsafe { plain::as_bytes(&value) };

    if let Err(e) = mapping_map.update(key, value, MapFlags::ANY) {
        println!("setting wan ip error:{e:?}");
    }
}

pub fn handle_ipv6_ingress(mut payload: Vec<u8>) {
    let builder = LandNatV2SkelBuilder::default();
    let mut open_object = MaybeUninit::uninit();
    let open = builder.open(&mut open_object).unwrap();

    let skel = open.load().unwrap();

    let ifindex = 6;

    let handle_ipv6_ingress = skel.progs.handle_ipv6_ingress;

    add_ipv6_mapping(
        &skel.maps.ip6_client_map,
        0x5edf,
        Ipv6Addr::from_str("2409:8888:6666:4f21::").unwrap(),
        Ipv6Addr::from_str("2001:db8:1::1").unwrap(),
        58,
    );

    add_ipv6_mapping(
        &skel.maps.ip6_client_map,
        0xd4fe,
        Ipv6Addr::from_str("2409:8888:6666:4f22::").unwrap(),
        Ipv6Addr::from_str("2001:da8:d800::1043").unwrap(),
        58,
    );

    add_ipv6_mapping(
        &skel.maps.ip6_client_map,
        0xd4fe,
        Ipv6Addr::from_str("2409:8888:6666:4f22::").unwrap(),
        Ipv6Addr::from_str("2001:4860:4860:8889:91a1:bbcc:ddee:ff11").unwrap(),
        58,
    );

    add_ipv6_mapping(
        &skel.maps.ip6_client_map,
        80,
        Ipv6Addr::from_str("2409:8888:6666:4f22::").unwrap(),
        Ipv6Addr::from_str("2001:da8:d800::1043").unwrap(),
        6,
    );
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
    let result = handle_ipv6_ingress.test_run(input).expect("test_run failed");

    println!("return_value = {}", result.return_value as i32);
    println!("duration = {:?}", result.duration);
    println!("packet_out = {:?}", packet_out);
    crate::tests::check::analyze(&packet_out);
    // std::thread::sleep(std::time::Duration::from_secs(100));
}

#[cfg(test)]
pub mod tests {

    use crate::tests::nat::{ipv6_ingress::handle_ipv6_ingress, package::*};

    #[test]
    fn ipv6_icmp_too_big() {
        handle_ipv6_ingress(build_ipv6_icmp_too_big());
    }

    #[test]
    fn ipv6_icmp_request() {
        handle_ipv6_ingress(build_ipv6_icmp_request());
    }

    #[test]
    fn ipv6_icmp_reply() {
        handle_ipv6_ingress(build_ipv6_icmp_reply());
    }

    #[test]
    fn tcp_syn() {
        handle_ipv6_ingress(build_tcp_syn());
    }

    #[test]
    fn tcp_syn_but_checksum_incorrect() {
        handle_ipv6_ingress(build_tcp_syn_but_checksum_incorrect());
    }
}
