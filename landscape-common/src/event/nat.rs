use std::net::IpAddr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct NatEvent {
    pub event_type: NatEventType,
    pub src_ip: IpAddr,
    pub dst_ip: IpAddr,
    pub src_port: u16,
    pub dst_port: u16,
    /// TCP / UDP / ICMP
    pub l4_proto: u8,
    pub flow_id: u8,
    pub trace_id: u8,
    pub l3_proto: u8,
    pub time: u64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub enum NatEventType {
    #[default]
    Unknow,
    CreateConnect,
    DisConnct,
}

impl From<u8> for NatEventType {
    fn from(value: u8) -> Self {
        match value {
            1 => NatEventType::CreateConnect,
            2 => NatEventType::DisConnct,
            _ => NatEventType::Unknow,
        }
    }
}
