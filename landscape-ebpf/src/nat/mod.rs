use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use crate::{LANDSCAPE_IPV4_TYPE, LANDSCAPE_IPV6_TYPE};

use crate::map_setting::share_map::types::{nat_conn_event, u_inet_addr};

use landscape_common::event::nat::{NatEvent, NatEventType};

pub mod test;
pub mod v2;

impl From<&nat_conn_event> for NatEvent {
    fn from(ev: &nat_conn_event) -> Self {
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

        NatEvent {
            event_type: NatEventType::from(ev.event_type),
            src_ip: convert_ip(&ev.src_addr, ev.l3_proto),
            dst_ip: convert_ip(&ev.dst_addr, ev.l3_proto),
            src_port: ev.src_port.to_be(),
            dst_port: ev.dst_port.to_be(),
            l4_proto: ev.l4_proto,
            flow_id: ev.flow_id,
            trace_id: ev.trace_id,
            l3_proto: ev.l3_proto,
            time: ev.create_time,
        }
    }
}
