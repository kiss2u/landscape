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
        share_map::types::{inet4_addr, inet4_pair, nat_timer_key_v4, nat_timer_value_v4},
    },
    nat::v2::land_nat_v2::LandNatV2SkelBuilder,
    tests::TestSkb,
};

const WAN_IP: Ipv4Addr = Ipv4Addr::new(203, 0, 113, 1);
const LAN_HOST: Ipv4Addr = Ipv4Addr::new(192, 168, 1, 100);
const REMOTE_IP: Ipv4Addr = Ipv4Addr::new(50, 18, 88, 205);
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
    use crate::map_setting::nat::NatMappingKeyV4;
    use crate::{NAT_MAPPING_EGRESS, NAT_MAPPING_INGRESS};

    // Test 4: Dynamic NAT — TCP egress (no static mapping)
    // No static mapping. WAN IP bound.
    // Egress: 192.168.1.100:56186 → 50.18.88.205:443
    // Expected: src_addr → 203.0.113.1, src_port allocated from range, ret = TC_ACT_UNSPEC(-1)
    //
    // Dynamic NAT creates both mapping and CT internally. Since bpf_timer_init
    // fails in BPF_PROG_TEST_RUN, we pre-populate a dynamic mapping and CT entry
    // to exercise the existing-mapping egress path instead.
    #[test]
    fn tcp_egress_dynamic() {
        let landscape_builder = LandNatV2SkelBuilder::default();
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

        // Pre-populate a dynamic NAT mapping pair to simulate previous connection.
        // Egress key: {EGRESS, TCP, src_port=56186, src_addr=LAN_HOST}
        // Egress value: {addr=WAN_IP, port=56186, trigger_addr=REMOTE, trigger_port=443}
        {
            use crate::map_setting::share_map::types::nat_mapping_value_v4;

            let egress_key = NatMappingKeyV4 {
                gress: NAT_MAPPING_EGRESS,
                l4proto: 6,
                from_port: 56186u16.to_be(),
                from_addr: LAN_HOST.to_bits().to_be(),
            };
            let egress_val = nat_mapping_value_v4 {
                addr: WAN_IP.to_bits().to_be(),
                port: 56186u16.to_be(),
                trigger_addr: REMOTE_IP.to_bits().to_be(),
                trigger_port: 443u16.to_be(),
                is_static: 0,
                is_allow_reuse: 1,
                ..Default::default()
            };

            let ingress_key = NatMappingKeyV4 {
                gress: NAT_MAPPING_INGRESS,
                l4proto: 6,
                from_port: 56186u16.to_be(),
                from_addr: WAN_IP.to_bits().to_be(),
            };
            let ingress_val = nat_mapping_value_v4 {
                addr: LAN_HOST.to_bits().to_be(),
                port: 56186u16.to_be(),
                trigger_addr: REMOTE_IP.to_bits().to_be(),
                trigger_port: 443u16.to_be(),
                is_static: 0,
                is_allow_reuse: 1,
                ..Default::default()
            };

            let ek = unsafe { plain::as_bytes(&egress_key) };
            let ev = unsafe { plain::as_bytes(&egress_val) };
            let ik = unsafe { plain::as_bytes(&ingress_key) };
            let iv = unsafe { plain::as_bytes(&ingress_val) };
            landscape_skel
                .maps
                .nat4_mappings
                .update(ek, ev, MapFlags::ANY)
                .expect("insert egress mapping");
            landscape_skel
                .maps
                .nat4_mappings
                .update(ik, iv, MapFlags::ANY)
                .expect("insert ingress mapping");
        }

        // Pre-populate CT: {remote:443, WAN_IP:56186}
        add_ct_entry(&landscape_skel.maps.nat4_mapping_timer, 6, REMOTE_IP, 443, WAN_IP, 56186);

        let mut pkt = build_ipv4_tcp(LAN_HOST, REMOTE_IP, 56186, 443);
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
            assert_eq!(tcp.source_port, 56186, "src_port should be the allocated WAN port");
        } else {
            panic!("expected TCP transport header in output");
        }
    }
}
