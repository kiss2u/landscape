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

fn build_ipv4_udp(src: Ipv4Addr, dst: Ipv4Addr, src_port: u16, dst_port: u16) -> Vec<u8> {
    let builder = PacketBuilder::ethernet2(
        [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF],
        [0x11, 0x22, 0x33, 0x44, 0x55, 0x66],
    )
    .ipv4(src.octets(), dst.octets(), 64)
    .udp(src_port, dst_port);

    let payload = [0u8; 8];
    let mut buf = Vec::with_capacity(builder.size(payload.len()));
    builder.write(&mut buf, &payload).unwrap();
    buf
}

/// Pre-populate a CT (connection tracking) entry in nat4_mapping_timer.
/// This is needed because bpf_timer_init fails in BPF_PROG_TEST_RUN context,
/// so we insert the CT entry from userspace to bypass the timer creation path.
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
        pair_ip: inet4_pair {
            src_addr: inet4_addr { addr: src_addr.to_bits().to_be() },
            dst_addr: inet4_addr { addr: dst_addr.to_bits().to_be() },
            src_port: src_port.to_be(),
            dst_port: dst_port.to_be(),
        },
    };
    let value = nat_timer_value_v4 {
        server_status: 1, // CT_INIT
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

    const TC_ACT_SHOT: i32 = 2;

    // Test 1a: Static NAT to local router — TCP ingress
    // Static mapping: WAN:8080 → 0.0.0.0:80 (TCP)
    // Ingress: 10.0.0.1:9999 → 203.0.113.1:8080
    // Expected: dst_port → 80, ret = TC_ACT_UNSPEC(-1)
    #[test]
    fn tcp_ingress_local_router() {
        let mut landscape_builder = LandNatV2SkelBuilder::default();
        let pin_root = crate::tests::nat::isolated_pin_root("nat-v4-static-local");
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
                wan_port: 8080,
                lan_port: 80,
                lan_ip: Ipv4Addr::UNSPECIFIED, // 0.0.0.0 = local router
                l4_protocol: 6,                // TCP
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
        if let Some(etherparse::TransportHeader::Tcp(tcp)) = pkt_out.transport {
            assert_eq!(tcp.destination_port, 80, "dst_port should be rewritten to 80");
        } else {
            panic!("expected TCP transport header in output");
        }
    }

    // Test 1b: Static NAT to local router — TCP egress (the addr=0 fallback fix)
    // Egress: 203.0.113.1:80 → 10.0.0.1:9999
    // Expected: src_port → 8080, ret = TC_ACT_UNSPEC(-1)
    #[test]
    fn tcp_egress_local_router() {
        let mut landscape_builder = LandNatV2SkelBuilder::default();
        let pin_root = crate::tests::nat::isolated_pin_root("nat-v4-static-local");
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
                wan_port: 8080,
                lan_port: 80,
                lan_ip: Ipv4Addr::UNSPECIFIED,
                l4_protocol: 6,
            }],
        );

        // Pre-populate CT: {remote:remote_port, WAN_IP:wan_port}
        // Egress CT key uses server_nat_pair = {dst(remote):dst_port, nat_addr(WAN):nat_port(8080)}
        add_ct_entry(&landscape_skel.maps.nat4_mapping_timer, 6, REMOTE_IP, 9999, WAN_IP, 8080);

        let mut pkt = build_ipv4_tcp(WAN_IP, REMOTE_IP, 80, 9999);
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
        if let Some(etherparse::TransportHeader::Tcp(tcp)) = pkt_out.transport {
            assert_eq!(tcp.source_port, 8080, "src_port should be rewritten to 8080");
        } else {
            panic!("expected TCP transport header in output");
        }
    }

    // Test 3a: Static NAT to local router — UDP ingress
    // Static mapping: WAN:5353 → 0.0.0.0:53 (UDP)
    // Ingress: 10.0.0.1:12345 → 203.0.113.1:5353
    // Expected: dst_port → 53, ret = TC_ACT_UNSPEC(-1)
    #[test]
    fn udp_ingress_local_router() {
        let mut landscape_builder = LandNatV2SkelBuilder::default();
        let pin_root = crate::tests::nat::isolated_pin_root("nat-v4-static-local");
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
                wan_port: 5353,
                lan_port: 53,
                lan_ip: Ipv4Addr::UNSPECIFIED,
                l4_protocol: 17, // UDP
            }],
        );

        // Pre-populate CT
        add_ct_entry(&landscape_skel.maps.nat4_mapping_timer, 17, REMOTE_IP, 12345, WAN_IP, 5353);

        let mut pkt = build_ipv4_udp(REMOTE_IP, WAN_IP, 12345, 5353);
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
        if let Some(etherparse::TransportHeader::Udp(udp)) = pkt_out.transport {
            assert_eq!(udp.destination_port, 53, "dst_port should be rewritten to 53");
        } else {
            panic!("expected UDP transport header in output");
        }
    }

    // Test 3b: Static NAT to local router — UDP egress (addr=0 fallback)
    // Egress: 203.0.113.1:53 → 10.0.0.1:12345
    // Expected: src_port → 5353, ret = TC_ACT_UNSPEC(-1)
    #[test]
    fn udp_egress_local_router() {
        let mut landscape_builder = LandNatV2SkelBuilder::default();
        let pin_root = crate::tests::nat::isolated_pin_root("nat-v4-static-local");
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
                wan_port: 5353,
                lan_port: 53,
                lan_ip: Ipv4Addr::UNSPECIFIED,
                l4_protocol: 17,
            }],
        );

        // Pre-populate CT
        add_ct_entry(&landscape_skel.maps.nat4_mapping_timer, 17, REMOTE_IP, 12345, WAN_IP, 5353);

        let mut pkt = build_ipv4_udp(WAN_IP, REMOTE_IP, 53, 12345);
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
        if let Some(etherparse::TransportHeader::Udp(udp)) = pkt_out.transport {
            assert_eq!(udp.source_port, 5353, "src_port should be rewritten to 5353");
        } else {
            panic!("expected UDP transport header in output");
        }
    }

    // Test 6: No matching static mapping — Ingress DROP
    // Static mapping: WAN:8080 → 0.0.0.0:80 (TCP)
    // Ingress to unmatched port 9090: 10.0.0.1:9999 → 203.0.113.1:9090
    // Expected: ret = TC_ACT_SHOT(2)
    #[test]
    fn tcp_ingress_no_match_drop() {
        let mut landscape_builder = LandNatV2SkelBuilder::default();
        let pin_root = crate::tests::nat::isolated_pin_root("nat-v4-static-local");
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
                wan_port: 8080,
                lan_port: 80,
                lan_ip: Ipv4Addr::UNSPECIFIED,
                l4_protocol: 6,
            }],
        );

        // Send to port 9090 which has no mapping — no CT entry needed
        let mut pkt = build_ipv4_tcp(REMOTE_IP, WAN_IP, 9999, 9090);
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

        assert_eq!(
            ret, TC_ACT_SHOT,
            "ingress with no matching mapping should return TC_ACT_SHOT(2)"
        );
    }
}
