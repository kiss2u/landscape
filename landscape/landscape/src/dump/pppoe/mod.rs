use std::{net::Ipv4Addr, time::SystemTime};

use pnet::util::Octets;
use serde::{Deserialize, Serialize};
use tags::PPPoETag;

use super::ipv4::split_u8_by_index;

pub mod tags;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PPPoEFrame {
    // 4 bit
    pub ver: u8,
    // 4 bit
    pub t: u8,
    pub code: u8,
    pub sid: u16,
    pub length: u16,

    pub payload: Vec<u8>,
    // pub tags: Vec<PPPoETag>,
}
impl PPPoEFrame {
    pub fn new(data: &[u8]) -> Option<PPPoEFrame> {
        if data.len() < 6 {
            return None;
        }
        let (ver, t) = split_u8_by_index(data[0], 4);
        let sid = ((data[2] as u16) << 8) | (data[3] as u16);
        let length = ((data[4] as u16) << 8) | (data[5] as u16);
        let result = PPPoEFrame {
            t,
            ver,
            code: data[1],
            sid,
            length,
            payload: data[6..].to_vec(),
        };
        Some(result)
    }

    /// Code: Active Discovery Offer (PADO) (0x07)
    pub fn is_offer(&self) -> bool {
        self.code == 0x07
    }

    /// Code: Active Discovery Terminate (PADT) (0xa7)
    pub fn is_terminate(&self) -> bool {
        self.code == 0xa7
    }

    /// Code: Active Discovery Session-confirmation (PADS) (0x65)
    pub fn is_confirm(&self) -> bool {
        self.code == 0x65
    }
    /// Code: Session Data (0x00)
    pub fn is_session_data(&self) -> bool {
        self.code == 0x00
    }

    pub fn convert_to_payload(self) -> Vec<u8> {
        let mut result = vec![(self.ver << 4) | (self.t & 0x0F), self.code];
        result.extend(self.sid.octets());
        // let mut tags = vec![];
        // for each in self.tags.into_iter() {
        //     tags.extend(each.decode_options());
        // }
        result.extend((self.payload.len() as u16).octets());
        result.extend(self.payload);
        result
    }

    pub fn get_discover(multi_modem: bool) -> (u32, PPPoEFrame) {
        let mut result = PPPoEFrame::new(&[17, 9, 0, 0, 0, 4, 1, 1, 0, 0]).unwrap();
        let mut host_uniq = 0;
        if multi_modem {
            host_uniq =
                SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as u32;
            result.payload.extend(PPPoETag::HostUniq(host_uniq).decode_options());
        }
        (host_uniq, result)
    }

    pub fn get_discover_with_host_uniq(host_uniq: u32) -> PPPoEFrame {
        let mut result = PPPoEFrame::new(&[17, 9, 0, 0, 0, 4, 1, 1, 0, 0]).unwrap();

        result.payload.extend(PPPoETag::HostUniq(host_uniq).decode_options());

        result
    }

    pub fn get_request(host_uniq_id: u32, ac_cookie: Option<Vec<u8>>) -> PPPoEFrame {
        let mut result = PPPoEFrame::new(&[17, 25, 0, 0, 0, 12, 1, 1, 0, 0]).unwrap();
        if host_uniq_id != 0 {
            result.payload.extend(PPPoETag::HostUniq(host_uniq_id).decode_options());
            if let Some(ac_cookie) = ac_cookie {
                result.payload.extend(PPPoETag::AcCookie(ac_cookie).decode_options());
            }
        }
        result.length = result.payload.len() as u16;
        result
    }

    pub fn conversion_payload_to_ppp(&self) -> Option<PointToPoint> {
        PointToPoint::new(&self.payload)
    }

    pub fn get_ppp_mru_config_request(sid: u16, request_id: u8, magic_number: u32) -> PPPoEFrame {
        let data = PointToPoint::request_mru(request_id, 1492, magic_number);
        PPPoEFrame {
            ver: 1,
            t: 1,
            code: 0,
            sid,
            length: data.len() as u16,
            payload: data,
        }
    }

    pub fn get_ppp_lcp_pap(sid: u16, peer_id: &str, password: &str) -> PPPoEFrame {
        let lcp_pap = PointToPoint::gen_pap(peer_id, password).convert_to_payload();
        PPPoEFrame {
            ver: 1,
            t: 1,
            code: 0,
            sid,
            length: lcp_pap.len() as u16,
            payload: lcp_pap,
        }
    }

    pub fn gen_echo_request_with_magic(sid: u16, req_id: u8, magic_number: u32) -> PPPoEFrame {
        let lcp_pap = PointToPoint::gen_echo_request_with_magic(req_id, magic_number);
        PPPoEFrame {
            ver: 1,
            t: 1,
            code: 0,
            sid,
            length: lcp_pap.len() as u16,
            payload: lcp_pap,
        }
    }

    /// gen IPCP config
    pub fn get_ipcp_request(sid: u16, req_id: u8) -> PPPoEFrame {
        let lcp_pap = PointToPoint::get_ipcp_request(
            req_id,
            Ipv4Addr::UNSPECIFIED,
            Ipv4Addr::UNSPECIFIED,
            Ipv4Addr::UNSPECIFIED,
        )
        .convert_to_payload();
        PPPoEFrame {
            ver: 1,
            t: 1,
            code: 0,
            sid,
            length: lcp_pap.len() as u16,
            payload: lcp_pap,
        }
    }

    pub fn get_ipcp_request_only_client_ip(sid: u16, req_id: u8, ip: Ipv4Addr) -> PPPoEFrame {
        let lcp_pap =
            PointToPoint::get_ipcp_request_only_client_ip(req_id, ip).convert_to_payload();
        PPPoEFrame {
            ver: 1,
            t: 1,
            code: 0,
            sid,
            length: lcp_pap.len() as u16,
            payload: lcp_pap,
        }
    }

    pub fn get_ipcp_request_with_ip(
        sid: u16,
        req_id: u8,
        ip: Ipv4Addr,
        dns1: Ipv4Addr,
        dns2: Ipv4Addr,
    ) -> PPPoEFrame {
        let lcp_pap = PointToPoint::get_ipcp_request(req_id, ip, dns1, dns2).convert_to_payload();
        PPPoEFrame {
            ver: 1,
            t: 1,
            code: 0,
            sid,
            length: lcp_pap.len() as u16,
            payload: lcp_pap,
        }
    }
    /// gen IPV6CP config
    pub fn get_ipv6cp_request(sid: u16, ipv6_interface_id: Vec<u8>, req_id: u8) -> PPPoEFrame {
        let lcp_pap =
            PointToPoint::get_ipv6cp_request(ipv6_interface_id, req_id).convert_to_payload();
        PPPoEFrame {
            ver: 1,
            t: 1,
            code: 0,
            sid,
            length: lcp_pap.len() as u16,
            payload: lcp_pap,
        }
    }

    pub fn get_termination_request(sid: u16, req_id: u8) -> PPPoEFrame {
        let lcp_pap = PointToPoint::get_termination_request(req_id).convert_to_payload();
        PPPoEFrame {
            ver: 1,
            t: 1,
            code: 0,
            sid,
            length: lcp_pap.len() as u16,
            payload: lcp_pap,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PointToPoint {
    /// 配置类型
    ///
    /// 0xc021 表示配置 LCP 本身
    /// 0xc023 进行 PAP 认证
    /// 0x8021 进行 IPCP
    /// 0x8057 进行 IPV6CP
    pub protocol: u16,
    /// LCP 的消息类型
    ///
    /// 1. 表示请求
    /// 2. 表示 ACK
    /// 3. 表示 NAK
    /// 5. 表示终止
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
        let protocol = ((data[0] as u16) << 8) | (data[1] as u16);
        let code = data[2];
        let id = data[3];
        let length = ((data[4] as u16) << 8) | (data[5] as u16);

        let data_end = length as usize + 2;
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
        let len = 14 as u16;
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
        result.push(4); // ack is 2
        result.push(self.id);
        result.extend((reject_option.len() as u16 + 4_u16).to_be_bytes());
        result.extend(reject_option);
        result
    }

    pub fn gen_ack(&self) -> Vec<u8> {
        let mut result = vec![];
        result.extend(self.protocol.to_be_bytes());
        result.push(2); // ack is 2
        result.push(self.id);
        result.extend(self.length.to_be_bytes());
        result.extend(self.payload.clone());
        result
    }

    pub fn gen_reply(&self) -> Vec<u8> {
        let mut result = vec![];
        result.extend(self.protocol.to_be_bytes());
        result.push(10); // reply is 10
        result.push(self.id);
        result.extend(self.length.to_be_bytes());
        result.extend(self.payload.clone());
        result
    }

    pub fn gen_reply_with_magic(&self, magic_number: u32) -> Vec<u8> {
        let mut result = vec![];
        result.extend(self.protocol.to_be_bytes());
        result.push(10); // reply is 10
        result.push(self.id);
        result.extend(8_u16.to_be_bytes());
        result.extend(magic_number.to_be_bytes());
        result
    }

    pub fn gen_echo_request_with_magic(id: u8, magic_number: u32) -> Vec<u8> {
        let mut result = vec![0xc0, 0x21];
        result.push(9); // echo request is 9
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
            length: (payload.len() as u16) + 4,
            payload,
        }
    }

    pub fn get_ipcp_request_only_client_ip(id: u8, ip: Ipv4Addr) -> PointToPoint {
        let ip = ip.octets();
        let options: Vec<u8> = [
            03, // ip add
            06, // len
            ip[0], ip[1], ip[2], ip[3], // ip
        ]
        .to_vec();

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
            03, // ip add
            06, // len
            ip[0], ip[1], ip[2], ip[3], // ip
            0x81,  // primary dns
            06,    // len
            dns1[0], dns1[1], dns1[2], dns1[3], // ip
            0x83,    // secondary dns
            06,      // len
            dns2[0], dns2[1], dns2[2], dns2[3], // ip
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
        let mut options: Vec<u8> = [
            01,   // ip add
            0x0a, // len
        ]
        .to_vec();
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
        // user request
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
        result.push(6); // termination ack
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

#[derive(Debug, Serialize, Deserialize)]
pub struct PPPOption {
    pub t: u8,
    pub length: u8,
    pub data: Vec<u8>,
}

impl PPPOption {
    pub fn from_bytes(data: &[u8]) -> Vec<PPPOption> {
        let mut result = vec![];
        let mut index = 0;
        loop {
            if index + 2 > data.len() {
                break;
            }
            let t = data[index];
            if t == 0 {
                break;
            }
            let length = data[index + 1];
            let data_end = index + length as usize;
            if data_end > data.len() {
                break;
            }
            result.push(PPPOption {
                t,
                length,
                data: data[(index + 2)..data_end].to_vec(),
            });
            index = data_end;
        }
        result
    }

    pub fn is_mru(&self) -> bool {
        self.t == 0x01
    }

    pub fn is_auth_type(&self) -> bool {
        self.t == 0x03
    }

    pub fn is_magic_number(&self) -> bool {
        self.t == 0x05
    }

    pub fn convert_to_payload(&self) -> Vec<u8> {
        let mut result = vec![self.t, self.length];
        result.extend(&self.data);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::{PPPOption, PPPoEFrame};

    #[test]
    fn discover() {
        let data = [17, 9, 0, 0, 0, 4, 1, 1, 0, 0];
        let p1 = PPPoEFrame::new(&data);
        println!("{:?}", p1);
        println!("{:?}", p1.unwrap().convert_to_payload());
        let data2 = [17, 9, 0, 0, 0, 12, 1, 1, 0, 0, 1, 3, 0, 4, 34, 30, 2, 0];
        let p2 = PPPoEFrame::new(&data2);
        println!("{:?}", p2);
        println!("{:?}", p2.unwrap().convert_to_payload());
    }

    #[test]
    fn test_option() {
        let data: Vec<u8> = [
            0x01, 0x04, 0x05, 0xd4, 0x03, 0x04, 0xc0, 0x23, 0x05, 0x06, 0xe1, 0xe3, 0xfb, 0x26,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0xfb, 0x26, 0x00, 0x00, 0x00, 0x00,
        ]
        .to_vec();
        let data = PPPOption::from_bytes(&data);
        println!("{:?}", data);
    }
}
