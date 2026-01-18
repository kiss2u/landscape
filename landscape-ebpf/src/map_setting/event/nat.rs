use landscape_common::metric::connect::{ConnectEventType, ConnectInfo, ConnectKey};

use crate::map_setting::{event::convert_ip, share_map::types::nat_conn_event};

impl From<&nat_conn_event> for ConnectInfo {
    fn from(ev: &nat_conn_event) -> Self {
        let key = ConnectKey {
            src_ip: convert_ip(&ev.src_addr, ev.l3_proto),
            dst_ip: convert_ip(&ev.dst_addr, ev.l3_proto),
            src_port: ev.src_port.to_be(),
            dst_port: ev.dst_port.to_be(),
            l4_proto: ev.l4_proto,
            flow_id: ev.flow_id,
            trace_id: ev.trace_id,
            l3_proto: ev.l3_proto,
            create_time: ev.create_time,
        };
        ConnectInfo {
            key,
            event_type: ConnectEventType::from(ev.event_type),
            report_time: ev.create_time,
        }
    }
}
