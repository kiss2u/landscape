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
    map_setting::{add_wan_ip, nat::StaticNatMappingV6Item},
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

fn build_ipv6_udp(src: Ipv6Addr, dst: Ipv6Addr, src_port: u16, dst_port: u16) -> Vec<u8> {
    let builder = PacketBuilder::ethernet2(
        [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF],
        [0x11, 0x22, 0x33, 0x44, 0x55, 0x66],
    )
    .ipv6(src.octets(), dst.octets(), 64)
    .udp(src_port, dst_port);

    let payload = [0u8; 8];
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
    use crate::map_setting::nat::add_static_nat6_mapping;

    const TC_ACT_SHOT: i32 = 2;

    // Local router: lan_ip = :: (unspecified)
    // WAN IP byte[7] = 0x21, id_byte = 0x21 & 0x0F = 0x01
    // Client suffix = last 8 bytes of WAN IP (2409:8888:6666:4f21::) = all zeros
    const CLIENT_SUFFIX: [u8; 8] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    // Client prefix = first 8 bytes of WAN IP
    const CLIENT_PREFIX: [u8; 8] = [0x24, 0x09, 0x88, 0x88, 0x66, 0x66, 0x4f, 0x21];
    const ID_BYTE: u8 = 0x01; // 0x21 & 0x0F

    // Test: Static NAT to local router — TCP ingress
    // Remote:9999 → WAN_IP:80
    // Static mapping: port 80 TCP, LAN=::
    // Expected: dst unchanged, ret=-1
    #[test]
    fn tcp_ingress_local_router() {
        let mut landscape_builder = LandNatV2SkelBuilder::default();
        let pin_root = crate::tests::nat::isolated_pin_root("nat-v6-static-local");
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

        add_static_nat6_mapping(
            &landscape_skel.maps.nat6_static_mappings,
            vec![StaticNatMappingV6Item {
                wan_port: 80,
                lan_port: 80,
                lan_ip: Ipv6Addr::UNSPECIFIED, // :: = local router
                l4_protocol: 6,                // TCP
            }],
        );

        add_ct6_entry(
            &landscape_skel.maps.nat6_conn_timer,
            6,
            CLIENT_SUFFIX,
            80,
            ID_BYTE,
            CLIENT_PREFIX,
            remote(),
            9999,
        );

        let mut pkt = build_ipv6_tcp(remote(), wan_ip(), 9999, 80);
        let mut ctx = TestSkb::default();
        ctx.ifindex = IFINDEX;

        let mut packet_out = vec![0u8; pkt.len()];
        let input = ProgramInput {
            data_in: Some(&mut pkt),
            context_in: Some(ctx.as_mut_bytes()),
            data_out: Some(&mut packet_out),
            ..Default::default()
        };

        let result = landscape_skel.progs.nat_v6_ingress.test_run(input).expect("test_run failed");

        let ret = result.return_value as i32;
        println!("return_value = {ret}");
        crate::tests::check::analyze(&packet_out);

        assert_eq!(ret, -1, "ingress should return TC_ACT_UNSPEC(-1)");

        let pkt_out =
            etherparse::PacketHeaders::from_ethernet_slice(&packet_out).expect("parse output");
        if let Some(etherparse::NetHeaders::Ipv6(ipv6, _)) = pkt_out.net {
            let dst: Ipv6Addr = ipv6.destination.into();
            assert_eq!(dst, wan_ip(), "dst should be unchanged for local router");
        } else {
            panic!("expected IPv6 header in output");
        }
        if let Some(etherparse::TransportHeader::Tcp(tcp)) = pkt_out.transport {
            assert_eq!(tcp.destination_port, 80, "dst_port should be unchanged");
        } else {
            panic!("expected TCP transport header in output");
        }
    }

    // Test: Static NAT to local router — TCP egress
    // WAN_IP:80 → Remote:9999
    // Expected: src unchanged (NPT identity for local router), ret=-1
    #[test]
    fn tcp_egress_local_router() {
        let mut landscape_builder = LandNatV2SkelBuilder::default();
        let pin_root = crate::tests::nat::isolated_pin_root("nat-v6-static-local");
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

        add_static_nat6_mapping(
            &landscape_skel.maps.nat6_static_mappings,
            vec![StaticNatMappingV6Item {
                wan_port: 80,
                lan_port: 80,
                lan_ip: Ipv6Addr::UNSPECIFIED,
                l4_protocol: 6,
            }],
        );

        add_ct6_entry(
            &landscape_skel.maps.nat6_conn_timer,
            6,
            CLIENT_SUFFIX,
            80,
            ID_BYTE,
            CLIENT_PREFIX,
            remote(),
            9999,
        );

        let mut pkt = build_ipv6_tcp(wan_ip(), remote(), 80, 9999);
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
            assert_eq!(src, wan_ip(), "src should be unchanged (NPT identity)");
        } else {
            panic!("expected IPv6 header in output");
        }
        if let Some(etherparse::TransportHeader::Tcp(tcp)) = pkt_out.transport {
            assert_eq!(tcp.source_port, 80, "src_port should be unchanged");
        } else {
            panic!("expected TCP transport header in output");
        }
    }

    // Test: Static NAT to local router — UDP ingress
    // Remote:12345 → WAN_IP:53 (UDP)
    // Static mapping: port 53 UDP, LAN=::
    // Expected: dst unchanged, ret=-1
    #[test]
    fn udp_ingress_local_router() {
        let mut landscape_builder = LandNatV2SkelBuilder::default();
        let pin_root = crate::tests::nat::isolated_pin_root("nat-v6-static-local");
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

        add_static_nat6_mapping(
            &landscape_skel.maps.nat6_static_mappings,
            vec![StaticNatMappingV6Item {
                wan_port: 53,
                lan_port: 53,
                lan_ip: Ipv6Addr::UNSPECIFIED,
                l4_protocol: 17, // UDP
            }],
        );

        add_ct6_entry(
            &landscape_skel.maps.nat6_conn_timer,
            17,
            CLIENT_SUFFIX,
            53,
            ID_BYTE,
            CLIENT_PREFIX,
            remote(),
            12345,
        );

        let mut pkt = build_ipv6_udp(remote(), wan_ip(), 12345, 53);
        let mut ctx = TestSkb::default();
        ctx.ifindex = IFINDEX;

        let mut packet_out = vec![0u8; pkt.len()];
        let input = ProgramInput {
            data_in: Some(&mut pkt),
            context_in: Some(ctx.as_mut_bytes()),
            data_out: Some(&mut packet_out),
            ..Default::default()
        };

        let result = landscape_skel.progs.nat_v6_ingress.test_run(input).expect("test_run failed");

        let ret = result.return_value as i32;
        println!("return_value = {ret}");
        crate::tests::check::analyze(&packet_out);

        assert_eq!(ret, -1, "ingress should return TC_ACT_UNSPEC(-1)");

        let pkt_out =
            etherparse::PacketHeaders::from_ethernet_slice(&packet_out).expect("parse output");
        if let Some(etherparse::NetHeaders::Ipv6(ipv6, _)) = pkt_out.net {
            let dst: Ipv6Addr = ipv6.destination.into();
            assert_eq!(dst, wan_ip(), "dst should be unchanged for local router");
        } else {
            panic!("expected IPv6 header in output");
        }
        if let Some(etherparse::TransportHeader::Udp(udp)) = pkt_out.transport {
            assert_eq!(udp.destination_port, 53, "dst_port should be unchanged");
        } else {
            panic!("expected UDP transport header in output");
        }
    }

    // Test: Static NAT to local router — UDP egress
    // WAN_IP:53 → Remote:12345
    // Expected: src unchanged, ret=-1
    #[test]
    fn udp_egress_local_router() {
        let mut landscape_builder = LandNatV2SkelBuilder::default();
        let pin_root = crate::tests::nat::isolated_pin_root("nat-v6-static-local");
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

        add_static_nat6_mapping(
            &landscape_skel.maps.nat6_static_mappings,
            vec![StaticNatMappingV6Item {
                wan_port: 53,
                lan_port: 53,
                lan_ip: Ipv6Addr::UNSPECIFIED,
                l4_protocol: 17,
            }],
        );

        add_ct6_entry(
            &landscape_skel.maps.nat6_conn_timer,
            17,
            CLIENT_SUFFIX,
            53,
            ID_BYTE,
            CLIENT_PREFIX,
            remote(),
            12345,
        );

        let mut pkt = build_ipv6_udp(wan_ip(), remote(), 53, 12345);
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
            assert_eq!(src, wan_ip(), "src should be unchanged (NPT identity)");
        } else {
            panic!("expected IPv6 header in output");
        }
        if let Some(etherparse::TransportHeader::Udp(udp)) = pkt_out.transport {
            assert_eq!(udp.source_port, 53, "src_port should be unchanged");
        } else {
            panic!("expected UDP transport header in output");
        }
    }

    // Test: No matching static mapping — Ingress DROP
    // Static mapping for port 80 only, send to port 9090
    // Expected: ret = TC_ACT_SHOT(2)
    #[test]
    fn tcp_ingress_no_match_drop() {
        let mut landscape_builder = LandNatV2SkelBuilder::default();
        let pin_root = crate::tests::nat::isolated_pin_root("nat-v6-static-local");
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

        add_static_nat6_mapping(
            &landscape_skel.maps.nat6_static_mappings,
            vec![StaticNatMappingV6Item {
                wan_port: 80,
                lan_port: 80,
                lan_ip: Ipv6Addr::UNSPECIFIED,
                l4_protocol: 6,
            }],
        );

        // Send to port 9090 which has no mapping — no CT entry needed
        let mut pkt = build_ipv6_tcp(remote(), wan_ip(), 9999, 9090);
        let mut ctx = TestSkb::default();
        ctx.ifindex = IFINDEX;

        let mut packet_out = vec![0u8; pkt.len()];
        let input = ProgramInput {
            data_in: Some(&mut pkt),
            context_in: Some(ctx.as_mut_bytes()),
            data_out: Some(&mut packet_out),
            ..Default::default()
        };

        let result = landscape_skel.progs.nat_v6_ingress.test_run(input).expect("test_run failed");

        let ret = result.return_value as i32;
        println!("return_value = {ret}");

        assert_eq!(
            ret, TC_ACT_SHOT,
            "ingress with no matching mapping should return TC_ACT_SHOT(2)"
        );
    }
}
