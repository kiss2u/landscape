use std::net::IpAddr;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize)]
pub enum FirewallMessage {
    Event(FirewallEvent),
    Metric(FirewallMetric),
}

#[derive(Debug, Serialize, Deserialize, Eq, Hash, PartialEq, Clone, TS)]
#[ts(export, export_to = "common/metric.d.ts")]
pub struct FirewallKey {
    pub src_ip: IpAddr,
    pub dst_ip: IpAddr,
    pub src_port: u16,
    pub dst_port: u16,
    pub l4_proto: u8,
    pub l3_proto: u8,
    pub flow_id: u8,
    pub trace_id: u8,
    #[ts(type = "number")]
    pub create_time: u64,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "common/metric.d.ts")]
pub struct FirewallEvent {
    pub event_type: FirewallEventType,
    pub src_ip: IpAddr,
    pub dst_ip: IpAddr,
    pub src_port: u16,
    pub dst_port: u16,
    /// TCP / UDP / ICMP
    pub l4_proto: u8,
    pub l3_proto: u8,
    pub flow_id: u8,
    pub trace_id: u8,
    #[ts(type = "number")]
    pub create_time: u64,
}

impl FirewallEvent {
    pub fn convert_to_key(self) -> (FirewallKey, FirewallEventType) {
        (
            FirewallKey {
                src_ip: self.src_ip,
                dst_ip: self.dst_ip,
                src_port: self.src_port,
                dst_port: self.dst_port,
                l4_proto: self.l4_proto,
                l3_proto: self.l3_proto,
                flow_id: self.flow_id,
                trace_id: self.trace_id,
                create_time: self.create_time,
            },
            self.event_type,
        )
    }
}

#[derive(Debug, Serialize, Deserialize, Default, TS)]
#[ts(export, export_to = "common/metric.d.ts")]
#[serde(rename_all = "snake_case")]
pub enum FirewallEventType {
    #[default]
    Unknow,
    CreateConnect,
    DisConnct,
}

impl From<u8> for FirewallEventType {
    fn from(value: u8) -> Self {
        match value {
            1 => FirewallEventType::CreateConnect,
            2 => FirewallEventType::DisConnct,
            _ => FirewallEventType::Unknow,
        }
    }
}

/// 防火墙数据上报 metric
#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(export, export_to = "common/metric.d.ts")]
pub struct FirewallMetric {
    pub src_ip: IpAddr,
    pub dst_ip: IpAddr,
    pub src_port: u16,
    pub dst_port: u16,
    #[ts(type = "number")]
    pub create_time: u64,
    #[ts(type = "number")]
    pub time: u64,
    /// TCP / UDP / ICMP
    pub l4_proto: u8,
    pub l3_proto: u8,
    pub flow_id: u8,
    pub trace_id: u8,
    #[ts(type = "number")]
    pub ingress_bytes: u64,
    #[ts(type = "number")]
    pub ingress_packets: u64,
    #[ts(type = "number")]
    pub egress_bytes: u64,
    #[ts(type = "number")]
    pub egress_packets: u64,
}

impl FirewallMetric {
    pub fn convert_to_key(&self) -> (FirewallKey, ConnectMetric) {
        let key = FirewallKey {
            src_ip: self.src_ip,
            dst_ip: self.dst_ip,
            src_port: self.src_port,
            dst_port: self.dst_port,
            l4_proto: self.l4_proto,
            l3_proto: self.l3_proto,
            flow_id: self.flow_id,
            trace_id: self.trace_id,
            create_time: self.create_time,
        };
        let metric = ConnectMetric {
            time: self.time,
            ingress_bytes: self.ingress_bytes,
            ingress_packets: self.ingress_packets,
            egress_bytes: self.egress_bytes,
            egress_packets: self.egress_packets,
        };
        (key, metric)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(export, export_to = "common/metric.d.ts")]
pub struct ConnectMetric {
    #[ts(type = "number")]
    pub time: u64,
    #[ts(type = "number")]
    pub ingress_bytes: u64,
    #[ts(type = "number")]
    pub ingress_packets: u64,
    #[ts(type = "number")]
    pub egress_bytes: u64,
    #[ts(type = "number")]
    pub egress_packets: u64,
}

impl ConnectMetric {
    pub fn append_other(&mut self, other: &Self) {
        self.time = other.time;
        self.ingress_bytes += other.ingress_bytes;
        self.ingress_packets += other.ingress_packets;
        self.egress_bytes += other.egress_bytes;
        self.egress_packets += other.egress_packets;
    }
}
