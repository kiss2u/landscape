use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use landscape_common::event::firewall::{FirewallEvent, FirewallEventType, FirewallMetric};

use crate::{LANDSCAPE_IPV4_TYPE, LANDSCAPE_IPV6_TYPE};

use super::firewall_bpf::types::{firewall_conn_event, firewall_conn_metric_event, u_inet_addr};

unsafe impl plain::Plain for firewall_conn_event {}
unsafe impl plain::Plain for u_inet_addr {}
unsafe impl plain::Plain for firewall_conn_metric_event {}

impl From<&firewall_conn_event> for FirewallEvent {
    fn from(ev: &firewall_conn_event) -> Self {
        let time = landscape_common::utils::time::get_relative_time_ns().unwrap_or_default();
        FirewallEvent {
            event_type: FirewallEventType::from(ev.event_type),
            src_ip: convert_ip(&ev.src_addr, ev.l3_proto),
            dst_ip: convert_ip(&ev.dst_addr, ev.l3_proto),
            src_port: ev.src_port.to_be(),
            dst_port: ev.dst_port.to_be(),
            l4_proto: ev.l4_proto,
            flow_id: ev.flow_id,
            trace_id: ev.trace_id,
            l3_proto: ev.l3_proto,
            create_time: ev.create_time + time,
        }
    }
}

impl From<&firewall_conn_metric_event> for FirewallMetric {
    fn from(ev: &firewall_conn_metric_event) -> Self {
        let time = landscape_common::utils::time::get_relative_time_ns().unwrap_or_default();
        FirewallMetric {
            src_ip: convert_ip(&ev.src_addr, ev.l3_proto),
            dst_ip: convert_ip(&ev.dst_addr, ev.l3_proto),
            src_port: ev.src_port.to_be(),
            dst_port: ev.dst_port.to_be(),
            l4_proto: ev.l4_proto,
            flow_id: ev.flow_id,
            trace_id: ev.trace_id,
            l3_proto: ev.l3_proto,
            create_time: ev.create_time + time,
            time: ev.time + time,
            ingress_bytes: ev.ingress_bytes,
            ingress_packets: ev.ingress_packets,
            egress_bytes: ev.egress_bytes,
            egress_packets: ev.egress_packets,
        }
    }
}

fn convert_ip(raw: &u_inet_addr, proto: u8) -> IpAddr {
    match proto {
        LANDSCAPE_IPV4_TYPE => {
            let ip = unsafe { raw.ip.clone().to_be() };
            IpAddr::V4(Ipv4Addr::from_bits(ip))
        }
        LANDSCAPE_IPV6_TYPE => {
            let bits = unsafe { raw.bits };
            IpAddr::V6(Ipv6Addr::from(bits))
        }
        _ => IpAddr::V4(Ipv4Addr::UNSPECIFIED), // fallback
    }
}
