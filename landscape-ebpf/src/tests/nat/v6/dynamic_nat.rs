use std::{
    mem::MaybeUninit,
    net::{IpAddr, Ipv6Addr},
    str::FromStr,
};

use etherparse::PacketBuilder;
use landscape_common::net::MacAddr;
use libbpf_rs::{
    skel::{OpenSkel, SkelBuilder as _},
    MapCore, MapFlags, ProgramInput,
};
use zerocopy::IntoBytes;

use crate::{
    map_setting::add_wan_ip,
    nat::v2::land_nat_v2::{
        types::{inet6_addr, nat_timer_key_v6, nat_timer_value_v6},
        LandNatV2SkelBuilder,
    },
    tests::TestSkb,
};

const IFINDEX: u32 = 6;

fn wan_ip() -> Ipv6Addr {
    Ipv6Addr::from_str("2409:8888:6666:4f21::").unwrap()
}

fn lan_host() -> Ipv6Addr {
    Ipv6Addr::from_str("fd00:1234:5678:abc5::200").unwrap()
}

fn remote() -> Ipv6Addr {
    Ipv6Addr::from_str("2001:db8:2::1").unwrap()
}

fn build_ipv6_tcp(src: Ipv6Addr, dst: Ipv6Addr, src_port: u16, dst_port: u16) -> Vec<u8> {
    let builder = PacketBuilder::ethernet2(
        [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF],
        [0x11, 0x22, 0x33, 0x44, 0x55, 0x66],
    )
    .ipv6(src.octets(), dst.octets(), 64)
    .tcp(src_port, dst_port, 0x12345678, 65535);

    let payload = [0u8; 0];
    let mut buf = Vec::with_capacity(builder.size(payload.len()));
    builder.write(&mut buf, &payload).unwrap();
    buf
}

/// Pre-populate a v6 CT entry in nat6_conn_timer.
fn add_ct6_entry<T: MapCore>(
    timer_map: &T,
    l4proto: u8,
    client_suffix: [u8; 8],
    client_port: u16,
    id_byte: u8,
    client_prefix: [u8; 8],
    trigger_addr: Ipv6Addr,
    trigger_port: u16,
) {
    let key = nat_timer_key_v6 {
        client_suffix,
        client_port: client_port.to_be(),
        id_byte,
        l4_protocol: l4proto,
    };
    let mut value = nat_timer_value_v6 {
        server_status: 1,
        client_status: 1,
        is_allow_reuse: 1,
        ..Default::default()
    };
    value.trigger_addr = inet6_addr { bytes: trigger_addr.octets() };
    value.trigger_port = trigger_port.to_be();
    value.client_prefix = client_prefix;

    let key_bytes = unsafe { plain::as_bytes(&key) };
    let value_bytes = unsafe { plain::as_bytes(&value) };
    timer_map.update(key_bytes, value_bytes, MapFlags::ANY).expect("failed to insert v6 CT entry");
}

#[cfg(test)]
mod tests {
    use super::*;

    // LAN host suffix (last 8 bytes of fd00:1234:5678:abc5::200)
    const CLIENT_SUFFIX: [u8; 8] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00];
    // LAN host prefix (first 8 bytes of fd00:1234:5678:abc5::200)
    const CLIENT_PREFIX: [u8; 8] = [0xfd, 0x00, 0x12, 0x34, 0x56, 0x78, 0xab, 0xc5];
    // id_byte = byte[7] of LAN host (0xc5) & npt_id_mask (0x0F) = 0x05
    const ID_BYTE: u8 = 0x05;

    // Expected NPT-translated prefix (first 8 bytes of 2409:8888:6666:4f25::)
    const WAN_NPT_PREFIX: [u8; 8] = [0x24, 0x09, 0x88, 0x88, 0x66, 0x66, 0x4f, 0x25];

    // Test: Dynamic NAT — TCP egress (no static mapping)
    // fd00:1234:5678:abc5::200:12345 → Remote:443
    // Pre-populate CT (simulating previous connection)
    // Expected: src prefix → 2409:8888:6666:4f25 (NPT), ret=-1
    #[test]
    fn tcp_egress_dynamic() {
        let mut landscape_builder = LandNatV2SkelBuilder::default();
        let pin_root = crate::tests::nat::isolated_pin_root("nat-v6-dynamic");
        landscape_builder.object_builder_mut().pin_root_path(&pin_root).unwrap();
        let mut open_object = MaybeUninit::uninit();
        let landscape_open = landscape_builder.open(&mut open_object).unwrap();
        let landscape_skel = landscape_open.load().unwrap();

        add_wan_ip(
            &landscape_skel.maps.wan_ip_binding,
            IFINDEX,
            IpAddr::V6(wan_ip()),
            None,
            60,
            Some(MacAddr::broadcast()),
        );

        // No static mapping — dynamic NAT only.
        // Pre-populate CT to simulate an existing connection.
        add_ct6_entry(
            &landscape_skel.maps.nat6_conn_timer,
            6,
            CLIENT_SUFFIX,
            12345,
            ID_BYTE,
            CLIENT_PREFIX,
            remote(),
            443,
        );

        let mut pkt = build_ipv6_tcp(lan_host(), remote(), 12345, 443);
        let mut ctx = TestSkb::default();
        ctx.ifindex = IFINDEX;

        let mut packet_out = vec![0u8; pkt.len()];
        let input = ProgramInput {
            data_in: Some(&mut pkt),
            context_in: Some(ctx.as_mut_bytes()),
            data_out: Some(&mut packet_out),
            ..Default::default()
        };

        let result = landscape_skel.progs.nat_v6_egress.test_run(input).expect("test_run failed");

        let ret = result.return_value as i32;
        println!("return_value = {ret}");
        crate::tests::check::analyze(&packet_out);

        assert_eq!(ret, -1, "egress should return TC_ACT_UNSPEC(-1)");

        let pkt_out =
            etherparse::PacketHeaders::from_ethernet_slice(&packet_out).expect("parse output");
        if let Some(etherparse::NetHeaders::Ipv6(ipv6, _)) = pkt_out.net {
            let src: Ipv6Addr = ipv6.source.into();
            // Prefix should be NPT-translated to WAN prefix
            assert_eq!(
                &src.octets()[..8],
                &WAN_NPT_PREFIX,
                "src prefix should be NPT-translated to WAN prefix"
            );
            // Suffix should be preserved
            assert_eq!(&src.octets()[8..], &CLIENT_SUFFIX, "src suffix should be preserved");
        } else {
            panic!("expected IPv6 header in output");
        }
        if let Some(etherparse::TransportHeader::Tcp(tcp)) = pkt_out.transport {
            assert_eq!(tcp.source_port, 12345, "src_port should be unchanged");
        } else {
            panic!("expected TCP transport header in output");
        }
    }
}
