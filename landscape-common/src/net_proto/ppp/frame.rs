use std::net::Ipv4Addr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PointToPoint {
    /// 0xc021 LCP, 0xc023 PAP, 0x8021 IPCP, 0x8057 IPV6CP
    pub protocol: u16,
    pub code: u8,
    pub id: u8,
    pub length: u16,
    pub payload: Vec<u8>,
}

impl PointToPoint {
    pub fn new(data: &[u8]) -> Option<PointToPoint> {
        if data.len() < 6 {
            return None;
        }
        let protocol = u16::from_be_bytes([data[0], data[1]]);
        let code = data[2];
        let id = data[3];
        let length = u16::from_be_bytes([data[4], data[5]]);
        if length < 4 {
            return None;
        }

        let data_end = length as usize + 2;
        if data_end > data.len() {
            return None;
        }

        Some(PointToPoint {
            protocol,
            code,
            id,
            length,
            payload: data[6..data_end].to_vec(),
        })
    }

    pub fn is_lcp_config(&self) -> bool {
        self.protocol == 0xc021
    }

    pub fn is_pap_auth(&self) -> bool {
        self.protocol == 0xc023
    }

    pub fn is_ipcp(&self) -> bool {
        self.protocol == 0x8021
    }

    pub fn is_ipv6cp(&self) -> bool {
        self.protocol == 0x8057
    }

    pub fn is_request(&self) -> bool {
        self.code == 1
    }

    pub fn is_ack(&self) -> bool {
        self.code == 2
    }

    pub fn is_nak(&self) -> bool {
        self.code == 3
    }

    pub fn is_reject(&self) -> bool {
        self.code == 4
    }

    pub fn is_termination(&self) -> bool {
        self.code == 5
    }

    pub fn is_termination_ack(&self) -> bool {
        self.code == 6
    }

    pub fn is_proto_reject(&self) -> bool {
        self.code == 8
    }

    pub fn is_echo_request(&self) -> bool {
        self.code == 9
    }

    pub fn is_echo_reply(&self) -> bool {
        self.code == 10
    }

    pub fn request_mru(request_id: u8, mru: u16, magic_number: u32) -> Vec<u8> {
        let len = 14_u16;
        [
            [0xc0, 0x21, 1, request_id].to_vec(),
            len.to_be_bytes().to_vec(),
            [1, 4].to_vec(),
            mru.to_be_bytes().to_vec(),
            [5, 6].to_vec(),
            magic_number.to_be_bytes().to_vec(),
        ]
        .concat()
    }

    pub fn gen_reject(&self, reject_option: Vec<u8>) -> Vec<u8> {
        let mut result = vec![];
        result.extend(self.protocol.to_be_bytes());
        result.push(4);
        result.push(self.id);
        result.extend((reject_option.len() as u16 + 4_u16).to_be_bytes());
        result.extend(reject_option);
        result
    }

    pub fn gen_ack(&self) -> Vec<u8> {
        let mut result = vec![];
        result.extend(self.protocol.to_be_bytes());
        result.push(2);
        result.push(self.id);
        result.extend(self.length.to_be_bytes());
        result.extend(self.payload.clone());
        result
    }

    pub fn gen_reply(&self) -> Vec<u8> {
        let mut result = vec![];
        result.extend(self.protocol.to_be_bytes());
        result.push(10);
        result.push(self.id);
        result.extend(self.length.to_be_bytes());
        result.extend(self.payload.clone());
        result
    }

    pub fn gen_reply_with_magic(&self, magic_number: u32) -> Vec<u8> {
        let mut result = vec![];
        result.extend(self.protocol.to_be_bytes());
        result.push(10);
        result.push(self.id);
        result.extend(8_u16.to_be_bytes());
        result.extend(magic_number.to_be_bytes());
        result
    }

    pub fn gen_echo_request_with_magic(id: u8, magic_number: u32) -> Vec<u8> {
        let mut result = vec![0xc0, 0x21];
        result.push(9);
        result.push(id);
        result.extend(8_u16.to_be_bytes());
        result.extend(magic_number.to_be_bytes());
        result
    }

    pub fn gen_pap(peer_id: &str, password: &str) -> PointToPoint {
        let mut payload = vec![peer_id.len() as u8];
        payload.extend(peer_id.as_bytes());
        payload.push(password.len() as u8);
        payload.extend(password.as_bytes());
        PointToPoint {
            protocol: 0xc023,
            code: 1,
            id: 1,
            length: payload.len() as u16 + 4,
            payload,
        }
    }

    pub fn get_ipcp_request_only_client_ip(id: u8, ip: Ipv4Addr) -> PointToPoint {
        let ip = ip.octets();
        let options: Vec<u8> = [3, 6, ip[0], ip[1], ip[2], ip[3]].to_vec();

        PointToPoint {
            protocol: 0x8021,
            code: 1,
            id,
            length: 10,
            payload: options,
        }
    }

    pub fn get_ipcp_request(id: u8, ip: Ipv4Addr, dns1: Ipv4Addr, dns2: Ipv4Addr) -> PointToPoint {
        let ip = ip.octets();
        let dns1 = dns1.octets();
        let dns2 = dns2.octets();
        let options: Vec<u8> = [
            3, 6, ip[0], ip[1], ip[2], ip[3], 0x81, 6, dns1[0], dns1[1], dns1[2], dns1[3], 0x83, 6,
            dns2[0], dns2[1], dns2[2], dns2[3],
        ]
        .to_vec();

        PointToPoint {
            protocol: 0x8021,
            code: 1,
            id,
            length: 22,
            payload: options,
        }
    }

    pub fn get_ipv6cp_request(ipv6_interface_id: Vec<u8>, id: u8) -> PointToPoint {
        let mut options: Vec<u8> = [1, 0x0a].to_vec();
        options.extend(ipv6_interface_id);
        let length = options.len() as u16 + 4;
        PointToPoint {
            protocol: 0x8057,
            code: 1,
            id,
            length,
            payload: options,
        }
    }

    pub fn get_termination_request(id: u8) -> PointToPoint {
        let options: Vec<u8> =
            [0x55, 0x73, 0x65, 0x72, 0x20, 0x72, 0x65, 0x71, 0x75, 0x65, 0x73, 0x74].to_vec();
        let length = options.len() as u16 + 4;
        PointToPoint {
            protocol: 0xc021,
            code: 5,
            id,
            length,
            payload: options,
        }
    }

    pub fn get_termination_ack(&self) -> Vec<u8> {
        let mut result = vec![];
        result.extend(self.protocol.to_be_bytes());
        result.push(6);
        result.push(self.id);
        result.extend(self.length.to_be_bytes());
        result.extend(self.payload.clone());
        result
    }

    pub fn convert_to_payload(&self) -> Vec<u8> {
        let mut result = vec![];
        result.extend(self.protocol.to_be_bytes());
        result.push(self.code);
        result.push(self.id);
        result.extend(self.length.to_be_bytes());
        result.extend(self.payload.clone());
        result
    }
}
