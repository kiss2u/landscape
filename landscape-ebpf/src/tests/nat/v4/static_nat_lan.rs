use std::{
    mem::MaybeUninit,
    net::{IpAddr, Ipv4Addr},
};

use etherparse::PacketBuilder;
use landscape_common::net::MacAddr;
use libbpf_rs::{
    skel::{OpenSkel, SkelBuilder as _},
    MapCore, MapFlags, ProgramInput,
};
use zerocopy::IntoBytes;

use crate::{
    map_setting::{
        add_wan_ip,
        nat::StaticNatMappingV4Item,
        share_map::types::{inet4_addr, inet4_pair, nat_timer_key_v4, nat_timer_value_v4},
    },
    nat::v2::land_nat_v2::LandNatV2SkelBuilder,
    tests::TestSkb,
};

const WAN_IP: Ipv4Addr = Ipv4Addr::new(203, 0, 113, 1);
const LAN_HOST: Ipv4Addr = Ipv4Addr::new(192, 168, 1, 100);
const REMOTE_IP: Ipv4Addr = Ipv4Addr::new(10, 0, 0, 1);
const IFINDEX: u32 = 6;

fn build_ipv4_tcp(src: Ipv4Addr, dst: Ipv4Addr, src_port: u16, dst_port: u16) -> Vec<u8> {
    let builder = PacketBuilder::ethernet2(
        [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF],
        [0x11, 0x22, 0x33, 0x44, 0x55, 0x66],
    )
    .ipv4(src.octets(), dst.octets(), 64)
    .tcp(src_port, dst_port, 0x12345678, 65535);

    let payload = [0u8; 0];
    let mut buf = Vec::with_capacity(builder.size(payload.len()));
    builder.write(&mut buf, &payload).unwrap();
    buf
}

/// Pre-populate a CT entry in nat4_mapping_timer.
fn add_ct_entry<T: MapCore>(
    timer_map: &T,
    l4proto: u8,
    src_addr: Ipv4Addr,
    src_port: u16,
    dst_addr: Ipv4Addr,
    dst_port: u16,
) {
    let key = nat_timer_key_v4 {
        l4proto,
        _pad: [0; 3],
        wan_ifindex: IFINDEX,
        pair_ip: inet4_pair {
            src_addr: inet4_addr { addr: src_addr.to_bits().to_be() },
            dst_addr: inet4_addr { addr: dst_addr.to_bits().to_be() },
            src_port: src_port.to_be(),
            dst_port: dst_port.to_be(),
        },
    };
    let value = nat_timer_value_v4 {
        server_status: 1,
        client_status: 1,
        ..Default::default()
    };
    let key_bytes = unsafe { plain::as_bytes(&key) };
    let value_bytes = unsafe { plain::as_bytes(&value) };
    timer_map.update(key_bytes, value_bytes, MapFlags::ANY).expect("failed to insert CT entry");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map_setting::nat::add_static_nat4_mapping;

    // Test 2a: Static NAT to LAN host — TCP ingress
    // Static mapping: WAN:8080 → 192.168.1.100:80 (TCP)
    // Ingress: 10.0.0.1:9999 → 203.0.113.1:8080
    // Expected: dst changed to 192.168.1.100:80, ret = TC_ACT_UNSPEC(-1)
    #[test]
    fn tcp_ingress_lan_host() {
        let mut landscape_builder = LandNatV2SkelBuilder::default();
        let pin_root = crate::tests::nat::isolated_pin_root("nat-v4-static-lan");
        landscape_builder.object_builder_mut().pin_root_path(&pin_root).unwrap();
        let mut open_object = MaybeUninit::uninit();
        let landscape_open = landscape_builder.open(&mut open_object).unwrap();
        let landscape_skel = landscape_open.load().unwrap();

        add_wan_ip(
            &landscape_skel.maps.wan_ip_binding,
            IFINDEX,
            IpAddr::V4(WAN_IP),
            None,
            24,
            Some(MacAddr::broadcast()),
        );

        add_static_nat4_mapping(
            &landscape_skel.maps.nat4_mappings,
            vec![StaticNatMappingV4Item {
                wan_ifindex: 0,
                wan_port: 8080,
                lan_port: 80,
                lan_ip: LAN_HOST,
                l4_protocol: 6,
            }],
        );

        // Pre-populate CT: {remote:remote_port, WAN_IP:wan_port}
        add_ct_entry(&landscape_skel.maps.nat4_mapping_timer, 6, REMOTE_IP, 9999, WAN_IP, 8080);

        let mut pkt = build_ipv4_tcp(REMOTE_IP, WAN_IP, 9999, 8080);
        let mut ctx = TestSkb::default();
        ctx.ifindex = IFINDEX;

        let mut packet_out = vec![0u8; pkt.len()];
        let input = ProgramInput {
            data_in: Some(&mut pkt),
            context_in: Some(ctx.as_mut_bytes()),
            data_out: Some(&mut packet_out),
            ..Default::default()
        };

        let result = landscape_skel.progs.nat_v4_ingress.test_run(input).expect("test_run failed");

        let ret = result.return_value as i32;
        println!("return_value = {ret}");
        crate::tests::check::analyze(&packet_out);

        assert_eq!(ret, -1, "ingress should return TC_ACT_UNSPEC(-1)");

        let pkt_out = etherparse::PacketHeaders::from_ethernet_slice(&packet_out)
            .expect("parse output packet");
        if let Some(etherparse::NetHeaders::Ipv4(ipv4, _)) = pkt_out.net {
            let dst: Ipv4Addr = ipv4.destination.into();
            assert_eq!(dst, LAN_HOST, "dst_ip should be rewritten to LAN host");
        } else {
            panic!("expected IPv4 header in output");
        }
        if let Some(etherparse::TransportHeader::Tcp(tcp)) = pkt_out.transport {
            assert_eq!(tcp.destination_port, 80, "dst_port should be rewritten to 80");
        } else {
            panic!("expected TCP transport header in output");
        }
    }

    // Test 2b: Static NAT to LAN host — TCP egress
    // Egress: 192.168.1.100:80 → 10.0.0.1:9999
    // Expected: src changed to 203.0.113.1:8080, ret = TC_ACT_UNSPEC(-1)
    #[test]
    fn tcp_egress_lan_host() {
        let mut landscape_builder = LandNatV2SkelBuilder::default();
        let pin_root = crate::tests::nat::isolated_pin_root("nat-v4-static-lan");
        landscape_builder.object_builder_mut().pin_root_path(&pin_root).unwrap();
        let mut open_object = MaybeUninit::uninit();
        let landscape_open = landscape_builder.open(&mut open_object).unwrap();
        let landscape_skel = landscape_open.load().unwrap();

        add_wan_ip(
            &landscape_skel.maps.wan_ip_binding,
            IFINDEX,
            IpAddr::V4(WAN_IP),
            None,
            24,
            Some(MacAddr::broadcast()),
        );

        add_static_nat4_mapping(
            &landscape_skel.maps.nat4_mappings,
            vec![StaticNatMappingV4Item {
                wan_ifindex: 0,
                wan_port: 8080,
                lan_port: 80,
                lan_ip: LAN_HOST,
                l4_protocol: 6,
            }],
        );

        // Pre-populate CT: {remote:remote_port, WAN_IP:wan_port}
        add_ct_entry(&landscape_skel.maps.nat4_mapping_timer, 6, REMOTE_IP, 9999, WAN_IP, 8080);

        let mut pkt = build_ipv4_tcp(LAN_HOST, REMOTE_IP, 80, 9999);
        let mut ctx = TestSkb::default();
        ctx.ifindex = IFINDEX;

        let mut packet_out = vec![0u8; pkt.len()];
        let input = ProgramInput {
            data_in: Some(&mut pkt),
            context_in: Some(ctx.as_mut_bytes()),
            data_out: Some(&mut packet_out),
            ..Default::default()
        };

        let result = landscape_skel.progs.nat_v4_egress.test_run(input).expect("test_run failed");

        let ret = result.return_value as i32;
        println!("return_value = {ret}");
        crate::tests::check::analyze(&packet_out);

        assert_eq!(ret, -1, "egress should return TC_ACT_UNSPEC(-1)");

        let pkt_out = etherparse::PacketHeaders::from_ethernet_slice(&packet_out)
            .expect("parse output packet");
        if let Some(etherparse::NetHeaders::Ipv4(ipv4, _)) = pkt_out.net {
            let src: Ipv4Addr = ipv4.source.into();
            assert_eq!(src, WAN_IP, "src_ip should be rewritten to WAN IP");
        } else {
            panic!("expected IPv4 header in output");
        }
        if let Some(etherparse::TransportHeader::Tcp(tcp)) = pkt_out.transport {
            assert_eq!(tcp.source_port, 8080, "src_port should be rewritten to 8080");
        } else {
            panic!("expected TCP transport header in output");
        }
    }
}
