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
        nat::NatMappingKeyV4,
        nat::{add_static_nat4_mapping, StaticNatMappingV4Item},
        share_map::types::{inet4_addr, inet4_pair, nat_mapping_value_v4, nat_timer_key_v4},
    },
    nat::v3::land_nat_v3::{types, LandNatV3SkelBuilder},
    tests::TestSkb,
    NAT_MAPPING_EGRESS, NAT_MAPPING_INGRESS,
};

const WAN_IP: Ipv4Addr = Ipv4Addr::new(203, 0, 113, 1);
const LAN_HOST: Ipv4Addr = Ipv4Addr::new(192, 168, 1, 100);
const REMOTE_IP: Ipv4Addr = Ipv4Addr::new(50, 18, 88, 205);
const IFINDEX: u32 = 6;
const LAN_PORT: u16 = 56186;
const NAT_PORT: u16 = 40000;
const GENERATION: u16 = 7;

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

fn build_ipv4_tcp_syn(src: Ipv4Addr, dst: Ipv4Addr, src_port: u16, dst_port: u16) -> Vec<u8> {
    let builder = PacketBuilder::ethernet2(
        [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF],
        [0x11, 0x22, 0x33, 0x44, 0x55, 0x66],
    )
    .ipv4(src.octets(), dst.octets(), 64)
    .tcp(src_port, dst_port, 0x12345678, 65535)
    .syn();

    let payload = [0u8; 0];
    let mut buf = Vec::with_capacity(builder.size(payload.len()));
    builder.write(&mut buf, &payload).unwrap();
    buf
}

fn put_v3_state<T: MapCore>(
    state_map: &T,
    l4proto: u8,
    nat_addr: Ipv4Addr,
    nat_port: u16,
    state_ref: u64,
) {
    let ingress_key = NatMappingKeyV4 {
        gress: NAT_MAPPING_INGRESS,
        l4proto,
        from_port: nat_port.to_be(),
        from_addr: nat_addr.to_bits().to_be(),
    };
    let value = types::nat4_mapping_state_v3 {
        state_ref,
        generation: GENERATION,
        _pad0: 0,
        _pad1: 0,
    };

    state_map
        .update(
            unsafe { plain::as_bytes(&ingress_key) },
            unsafe { plain::as_bytes(&value) },
            MapFlags::ANY,
        )
        .expect("insert v3 state");
}

fn add_v3_state<T: MapCore>(state_map: &T, l4proto: u8, nat_addr: Ipv4Addr, nat_port: u16) {
    put_v3_state(state_map, l4proto, nat_addr, nat_port, ((1u64) << 56) | 1);
}

fn delete_v3_state<T: MapCore>(state_map: &T, l4proto: u8, nat_addr: Ipv4Addr, nat_port: u16) {
    let ingress_key = NatMappingKeyV4 {
        gress: NAT_MAPPING_INGRESS,
        l4proto,
        from_port: nat_port.to_be(),
        from_addr: nat_addr.to_bits().to_be(),
    };

    let _ = state_map.delete(unsafe { plain::as_bytes(&ingress_key) });
}

fn add_v3_ct<T: MapCore>(
    timer_map: &T,
    l4proto: u8,
    src_addr: Ipv4Addr,
    src_port: u16,
    nat_addr: Ipv4Addr,
    nat_port: u16,
    client_addr: Ipv4Addr,
    client_port: u16,
    gress: u8,
) {
    let key = nat_timer_key_v4 {
        l4proto,
        _pad: [0; 3],
        pair_ip: inet4_pair {
            src_addr: inet4_addr { addr: src_addr.to_bits().to_be() },
            dst_addr: inet4_addr { addr: nat_addr.to_bits().to_be() },
            src_port: src_port.to_be(),
            dst_port: nat_port.to_be(),
        },
    };

    let mut value = types::nat_timer_value_v4_v3::default();
    value.client_addr = types::inet4_addr { addr: client_addr.to_bits().to_be() };
    value.client_port = client_port.to_be();
    value.client_status = 1;
    value.server_status = 1;
    value.gress = gress;
    value.generation_snapshot = GENERATION;

    timer_map
        .update(unsafe { plain::as_bytes(&key) }, unsafe { plain::as_bytes(&value) }, MapFlags::ANY)
        .expect("insert v3 ct");
}

fn delete_v3_ct<T: MapCore>(
    timer_map: &T,
    l4proto: u8,
    src_addr: Ipv4Addr,
    src_port: u16,
    nat_addr: Ipv4Addr,
    nat_port: u16,
) {
    let key = nat_timer_key_v4 {
        l4proto,
        _pad: [0; 3],
        pair_ip: inet4_pair {
            src_addr: inet4_addr { addr: src_addr.to_bits().to_be() },
            dst_addr: inet4_addr { addr: nat_addr.to_bits().to_be() },
            src_port: src_port.to_be(),
            dst_port: nat_port.to_be(),
        },
    };

    let _ = timer_map.delete(unsafe { plain::as_bytes(&key) });
}

fn add_dynamic_mapping_pair<T: MapCore>(
    map: &T,
    l4proto: u8,
    lan_addr: Ipv4Addr,
    lan_port: u16,
    nat_addr: Ipv4Addr,
    nat_port: u16,
    remote_addr: Ipv4Addr,
    remote_port: u16,
) {
    let egress_key = NatMappingKeyV4 {
        gress: NAT_MAPPING_EGRESS,
        l4proto,
        from_port: lan_port.to_be(),
        from_addr: lan_addr.to_bits().to_be(),
    };
    let egress_val = nat_mapping_value_v4 {
        addr: nat_addr.to_bits().to_be(),
        port: nat_port.to_be(),
        trigger_addr: remote_addr.to_bits().to_be(),
        trigger_port: remote_port.to_be(),
        is_static: 0,
        is_allow_reuse: 1,
        ..Default::default()
    };

    let ingress_key = NatMappingKeyV4 {
        gress: NAT_MAPPING_INGRESS,
        l4proto,
        from_port: nat_port.to_be(),
        from_addr: nat_addr.to_bits().to_be(),
    };
    let ingress_val = nat_mapping_value_v4 {
        addr: lan_addr.to_bits().to_be(),
        port: lan_port.to_be(),
        trigger_addr: remote_addr.to_bits().to_be(),
        trigger_port: remote_port.to_be(),
        is_static: 0,
        is_allow_reuse: 1,
        ..Default::default()
    };

    map.update(
        unsafe { plain::as_bytes(&egress_key) },
        unsafe { plain::as_bytes(&egress_val) },
        MapFlags::ANY,
    )
    .expect("insert egress mapping");
    map.update(
        unsafe { plain::as_bytes(&ingress_key) },
        unsafe { plain::as_bytes(&ingress_val) },
        MapFlags::ANY,
    )
    .expect("insert ingress mapping");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::nat::NAT_V3_TEST_LOCK;

    #[test]
    fn tcp_egress_dynamic_v3_existing_state_and_ct() {
        let _guard = NAT_V3_TEST_LOCK.lock().unwrap();
        let landscape_builder = LandNatV3SkelBuilder::default();
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

        add_dynamic_mapping_pair(
            &landscape_skel.maps.nat4_mappings,
            6,
            LAN_HOST,
            LAN_PORT,
            WAN_IP,
            NAT_PORT,
            REMOTE_IP,
            443,
        );
        add_v3_state(&landscape_skel.maps.nat4_dynamic_state_v3, 6, WAN_IP, NAT_PORT);
        add_v3_ct(
            &landscape_skel.maps.nat4_mapping_timer_v3,
            6,
            REMOTE_IP,
            443,
            WAN_IP,
            NAT_PORT,
            LAN_HOST,
            LAN_PORT,
            NAT_MAPPING_EGRESS,
        );

        let mut pkt = build_ipv4_tcp(LAN_HOST, REMOTE_IP, LAN_PORT, 443);
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

        assert_eq!(result.return_value as i32, -1);

        let pkt_out = etherparse::PacketHeaders::from_ethernet_slice(&packet_out)
            .expect("parse output packet");
        if let Some(etherparse::NetHeaders::Ipv4(ipv4, _)) = pkt_out.net {
            let src: Ipv4Addr = ipv4.source.into();
            assert_eq!(src, WAN_IP);
        } else {
            panic!("expected IPv4 header in output");
        }
        if let Some(etherparse::TransportHeader::Tcp(tcp)) = pkt_out.transport {
            assert_eq!(tcp.source_port, NAT_PORT);
        } else {
            panic!("expected TCP transport header in output");
        }
    }

    #[test]
    fn tcp_egress_dynamic_v3_missing_state_drops() {
        let _guard = NAT_V3_TEST_LOCK.lock().unwrap();
        let landscape_builder = LandNatV3SkelBuilder::default();
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

        add_dynamic_mapping_pair(
            &landscape_skel.maps.nat4_mappings,
            6,
            LAN_HOST,
            LAN_PORT,
            WAN_IP,
            NAT_PORT,
            REMOTE_IP,
            443,
        );
        delete_v3_state(&landscape_skel.maps.nat4_dynamic_state_v3, 6, WAN_IP, NAT_PORT);

        let mut pkt = build_ipv4_tcp(LAN_HOST, REMOTE_IP, LAN_PORT, 443);
        let mut ctx = TestSkb::default();
        ctx.ifindex = IFINDEX;

        let input = ProgramInput {
            data_in: Some(&mut pkt),
            context_in: Some(ctx.as_mut_bytes()),
            context_out: None,
            data_out: None,
            ..Default::default()
        };

        let result = landscape_skel.progs.nat_v4_egress.test_run(input).expect("test_run failed");

        assert_eq!(result.return_value as i32, 2, "missing v3 state should drop packet");
    }

    #[test]
    fn tcp_ingress_dynamic_v3_reuse_creates_ct() {
        let _guard = NAT_V3_TEST_LOCK.lock().unwrap();
        let landscape_builder = LandNatV3SkelBuilder::default();
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

        add_dynamic_mapping_pair(
            &landscape_skel.maps.nat4_mappings,
            6,
            LAN_HOST,
            LAN_PORT,
            WAN_IP,
            NAT_PORT,
            REMOTE_IP,
            443,
        );
        add_v3_state(&landscape_skel.maps.nat4_dynamic_state_v3, 6, WAN_IP, NAT_PORT);
        delete_v3_ct(
            &landscape_skel.maps.nat4_mapping_timer_v3,
            6,
            REMOTE_IP,
            443,
            WAN_IP,
            NAT_PORT,
        );

        let mut pkt = build_ipv4_tcp_syn(REMOTE_IP, WAN_IP, 443, NAT_PORT);
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

        assert_eq!(result.return_value as i32, -1);

        let pkt_out = etherparse::PacketHeaders::from_ethernet_slice(&packet_out)
            .expect("parse output packet");
        if let Some(etherparse::NetHeaders::Ipv4(ipv4, _)) = pkt_out.net {
            let dst: Ipv4Addr = ipv4.destination.into();
            assert_eq!(dst, LAN_HOST);
        } else {
            panic!("expected IPv4 header in output");
        }
        if let Some(etherparse::TransportHeader::Tcp(tcp)) = pkt_out.transport {
            assert_eq!(tcp.destination_port, LAN_PORT);
        } else {
            panic!("expected TCP transport header in output");
        }

        let ingress_key = NatMappingKeyV4 {
            gress: NAT_MAPPING_INGRESS,
            l4proto: 6,
            from_port: NAT_PORT.to_be(),
            from_addr: WAN_IP.to_bits().to_be(),
        };
        let state_bytes = landscape_skel
            .maps
            .nat4_dynamic_state_v3
            .lookup(unsafe { plain::as_bytes(&ingress_key) }, MapFlags::ANY)
            .expect("lookup v3 state")
            .expect("state should exist");
        let state = unsafe {
            std::ptr::read_unaligned(state_bytes.as_ptr().cast::<types::nat4_mapping_state_v3>())
        };
        assert_eq!(state.generation, GENERATION);
        assert_eq!(state.state_ref, ((1u64) << 56) | 2, "reuse ingress should incref state_ref");

        let timer_key = nat_timer_key_v4 {
            l4proto: 6,
            _pad: [0; 3],
            pair_ip: inet4_pair {
                src_addr: inet4_addr { addr: REMOTE_IP.to_bits().to_be() },
                dst_addr: inet4_addr { addr: WAN_IP.to_bits().to_be() },
                src_port: 443u16.to_be(),
                dst_port: NAT_PORT.to_be(),
            },
        };
        let timer_bytes = landscape_skel
            .maps
            .nat4_mapping_timer_v3
            .lookup(unsafe { plain::as_bytes(&timer_key) }, MapFlags::ANY)
            .expect("lookup v3 ct");
        let timer_bytes = timer_bytes.expect("ingress reuse should create ct");
        let timer = unsafe {
            std::ptr::read_unaligned(timer_bytes.as_ptr().cast::<types::nat_timer_value_v4_v3>())
        };
        assert_eq!(timer.generation_snapshot, GENERATION);
        assert_eq!(timer.client_port, LAN_PORT.to_be());
    }

    #[test]
    fn tcp_ingress_dynamic_v3_closed_blocks_new_ct() {
        let _guard = NAT_V3_TEST_LOCK.lock().unwrap();
        let landscape_builder = LandNatV3SkelBuilder::default();
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

        add_dynamic_mapping_pair(
            &landscape_skel.maps.nat4_mappings,
            6,
            LAN_HOST,
            LAN_PORT,
            WAN_IP,
            NAT_PORT,
            REMOTE_IP,
            443,
        );
        put_v3_state(
            &landscape_skel.maps.nat4_dynamic_state_v3,
            6,
            WAN_IP,
            NAT_PORT,
            ((2u64) << 56) | 1,
        );
        delete_v3_ct(
            &landscape_skel.maps.nat4_mapping_timer_v3,
            6,
            REMOTE_IP,
            443,
            WAN_IP,
            NAT_PORT,
        );

        let mut pkt = build_ipv4_tcp_syn(REMOTE_IP, WAN_IP, 443, NAT_PORT);
        let mut ctx = TestSkb::default();
        ctx.ifindex = IFINDEX;

        let input = ProgramInput {
            data_in: Some(&mut pkt),
            context_in: Some(ctx.as_mut_bytes()),
            context_out: None,
            data_out: None,
            ..Default::default()
        };

        let result = landscape_skel.progs.nat_v4_ingress.test_run(input).expect("test_run failed");

        assert_eq!(result.return_value as i32, 2, "closed mapping should reject new ingress CT");

        let ingress_key = NatMappingKeyV4 {
            gress: NAT_MAPPING_INGRESS,
            l4proto: 6,
            from_port: NAT_PORT.to_be(),
            from_addr: WAN_IP.to_bits().to_be(),
        };
        let state_bytes = landscape_skel
            .maps
            .nat4_dynamic_state_v3
            .lookup(unsafe { plain::as_bytes(&ingress_key) }, MapFlags::ANY)
            .expect("lookup v3 state")
            .expect("state should exist");
        let state = unsafe {
            std::ptr::read_unaligned(state_bytes.as_ptr().cast::<types::nat4_mapping_state_v3>())
        };
        assert_eq!(state.state_ref, ((2u64) << 56) | 1);
    }

    #[test]
    fn tcp_ingress_dynamic_v3_closed_allows_existing_ct() {
        let _guard = NAT_V3_TEST_LOCK.lock().unwrap();
        let landscape_builder = LandNatV3SkelBuilder::default();
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

        add_dynamic_mapping_pair(
            &landscape_skel.maps.nat4_mappings,
            6,
            LAN_HOST,
            LAN_PORT,
            WAN_IP,
            NAT_PORT,
            REMOTE_IP,
            443,
        );
        put_v3_state(
            &landscape_skel.maps.nat4_dynamic_state_v3,
            6,
            WAN_IP,
            NAT_PORT,
            ((2u64) << 56) | 1,
        );
        add_v3_ct(
            &landscape_skel.maps.nat4_mapping_timer_v3,
            6,
            REMOTE_IP,
            443,
            WAN_IP,
            NAT_PORT,
            LAN_HOST,
            LAN_PORT,
            NAT_MAPPING_INGRESS,
        );

        let mut pkt = build_ipv4_tcp_syn(REMOTE_IP, WAN_IP, 443, NAT_PORT);
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

        assert_eq!(result.return_value as i32, -1);

        let pkt_out = etherparse::PacketHeaders::from_ethernet_slice(&packet_out)
            .expect("parse output packet");
        if let Some(etherparse::NetHeaders::Ipv4(ipv4, _)) = pkt_out.net {
            let dst: Ipv4Addr = ipv4.destination.into();
            assert_eq!(dst, LAN_HOST);
        } else {
            panic!("expected IPv4 header in output");
        }

        let ingress_key = NatMappingKeyV4 {
            gress: NAT_MAPPING_INGRESS,
            l4proto: 6,
            from_port: NAT_PORT.to_be(),
            from_addr: WAN_IP.to_bits().to_be(),
        };
        let state_bytes = landscape_skel
            .maps
            .nat4_dynamic_state_v3
            .lookup(unsafe { plain::as_bytes(&ingress_key) }, MapFlags::ANY)
            .expect("lookup v3 state")
            .expect("state should exist");
        let state = unsafe {
            std::ptr::read_unaligned(state_bytes.as_ptr().cast::<types::nat4_mapping_state_v3>())
        };
        assert_eq!(
            state.state_ref,
            ((2u64) << 56) | 1,
            "existing ct should not incref closed mapping"
        );
    }

    #[test]
    fn tcp_egress_static_v3_creates_ct_without_dynamic_state() {
        let _guard = NAT_V3_TEST_LOCK.lock().unwrap();
        let landscape_builder = LandNatV3SkelBuilder::default();
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
                lan_ip: LAN_HOST,
                l4_protocol: 6,
            }],
        );

        let mut pkt = build_ipv4_tcp_syn(LAN_HOST, REMOTE_IP, 80, 443);
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

        assert_eq!(result.return_value as i32, -1);

        let pkt_out = etherparse::PacketHeaders::from_ethernet_slice(&packet_out)
            .expect("parse output packet");
        if let Some(etherparse::NetHeaders::Ipv4(ipv4, _)) = pkt_out.net {
            let src: Ipv4Addr = ipv4.source.into();
            assert_eq!(src, WAN_IP);
        } else {
            panic!("expected IPv4 header in output");
        }
        if let Some(etherparse::TransportHeader::Tcp(tcp)) = pkt_out.transport {
            assert_eq!(tcp.source_port, 8080);
        } else {
            panic!("expected TCP transport header in output");
        }

        let timer_key = nat_timer_key_v4 {
            l4proto: 6,
            _pad: [0; 3],
            pair_ip: inet4_pair {
                src_addr: inet4_addr { addr: REMOTE_IP.to_bits().to_be() },
                dst_addr: inet4_addr { addr: WAN_IP.to_bits().to_be() },
                src_port: 443u16.to_be(),
                dst_port: 8080u16.to_be(),
            },
        };
        let timer_bytes = landscape_skel
            .maps
            .nat4_mapping_timer_v3
            .lookup(unsafe { plain::as_bytes(&timer_key) }, MapFlags::ANY)
            .expect("lookup static v3 ct");
        let timer_bytes = timer_bytes.expect("static egress should create ct");
        let timer = unsafe {
            std::ptr::read_unaligned(timer_bytes.as_ptr().cast::<types::nat_timer_value_v4_v3>())
        };
        assert_eq!(timer.client_port, 80u16.to_be());
    }
}
