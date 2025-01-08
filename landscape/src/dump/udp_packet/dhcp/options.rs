use std::net::Ipv4Addr;

use pnet::util::Octets;
use serde::{Deserialize, Serialize};

use hickory_proto::rr::Name;

/// 定义 OptionDefined trait
pub trait OptionDefined {
    /// 编码为字节数组
    fn encode(&self) -> Vec<u8>;

    /// 解码成对应的类型
    fn decode(data: &[u8]) -> Option<Self>
    where
        Self: Sized;
}

macro_rules! dhcp_options {
    ( $( { $num:expr, $name:ident, $comment:expr, $type:ty } ),* $(,)? ) => {
        #[derive(Debug, Clone)]
        #[derive(serde::Serialize, serde::Deserialize)]
        #[repr(u8)]
        pub enum DhcpOptions {

            #[doc = "0: Padding"]
            Pad = 0,

            $(
                #[doc = $comment]
                $name($type) = $num,
            )*

            #[doc = "end-of-list marker"]
            End(Vec<u8>) = 255,
        }

        impl DhcpOptions {
            pub fn from_data(data: &[u8]) -> Vec<DhcpOptions> {
                let mut result = vec![];
                let mut index = 0;

                while index < data.len() {
                    let code = data[index];
                    index += 1;
                    match code {
                        0 => {
                            result.push(DhcpOptions::Pad);
                        },
                        $(
                            $num => {
                                let length = data[index]as usize;
                                index += 1;

                                let start_index = index;
                                index += length ;

                                let option = <$type as OptionDefined>::decode(&data[start_index..index]);
                                if let Some(option) = option {
                                    result.push(DhcpOptions::$name(option));
                                }
                            }
                        )*
                        255 | _ => {
                            result.push(DhcpOptions::End(data[(index - 1)..].to_vec()));
                            break;
                        },
                    }
                }

                result
            }

            pub fn get_index(&self) -> u8 {
                match self {
                    DhcpOptions::Pad => 0,
                    $(
                        DhcpOptions::$name(_) => $num,
                    )*
                    DhcpOptions::End(_) => 255
                }
            }

            pub fn decode_option(&self) -> Vec<u8> {
                match self {
                    DhcpOptions::Pad => vec![0],
                    $(
                        DhcpOptions::$name(value) => {
                            let data = value.encode();
                            [vec![$num, data.len() as u8], data].concat()
                        }
                    )*
                    DhcpOptions::End(data) => data.clone()
                }
            }

        }
    };
}

dhcp_options! {
    {1,   SubnetMask, "1: Subnet Mask", Ipv4Addr},
    {2,   TimeOffset, "2: Time Offset", i32},
    {3,   Router, "3: Router", Ipv4Addr},
    {6,   DomainNameServer, "6: Name Server", Vec<Ipv4Addr>},
    {12,  Hostname, "12: Host name", String},
    {15,  DomainName, "15: Domain Name", String},
    {26,  InterfaceMtu, "26: Interface MTU", u16},
    {28,  BroadcastAddr, "28: Broadcast address", Ipv4Addr},
    {42,  NtpServers, "42: NTP servers", Vec<Ipv4Addr>},
    {44,  NetBiosNameServers, "44: NetBIOS over TCP/IP name server", Vec<Ipv4Addr>},
    {47,  NetBiosScope, "47: NetBIOS over TCP/IP Scope", String},
    {50,  RequestedIpAddress, "50: Requested IP Address", Ipv4Addr},
    {51,  AddressLeaseTime, "51: IP Address Lease Time, second", u32},
    {53,  MessageType, "53: Message Type", DhcpOptionMessageType},
    {54,  ServerIdentifier, "54: Server Identifier", Ipv4Addr},
    {55,  ParameterRequestList, "55: Parameter Request List", Vec<u8>},
    {56,  Message, "56: Message", String},
    {57,  MaxMessageSize, "57: Maximum DHCP Message Size", u16},
    {58,  Renewal, "58: Renewal (T1) Time Value", u32},
    {59,  Rebinding, "59: Rebinding (T2) Time Value", u32},
    {60,  ClassIdentifier, "60: Class-identifier", Vec<u8>},
    {61,  ClientIdentifier, "61: Client Identifier", Vec<u8>},
    {81,  ClientFQDN, "81: FQDN - <https://datatracker.ietf.org/doc/html/rfc4702>", Vec<u8>},
    {116, DisableSLAAC, "116: Disable Stateless Autoconfig for Ipv4 - <https://datatracker.ietf.org/doc/html/rfc2563>", AutoConfig},
    {119, DomainSearch, "119: Domain Search - <https://www.rfc-editor.org/rfc/rfc3397.html>", Vec<Name>},
}

// 定义 EmptyOption 结构体和实现 OptionDefined trait
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct EmptyOption;

// 为 Ipv4Addr 实现 OptionDefined trait
impl OptionDefined for Ipv4Addr {
    fn encode(&self) -> Vec<u8> {
        self.octets().to_vec()
    }

    fn decode(data: &[u8]) -> Option<Self> {
        if data.len() == 4 {
            Some(Ipv4Addr::new(data[0], data[1], data[2], data[3]))
        } else {
            None
        }
    }
}

impl OptionDefined for i32 {
    fn encode(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn decode(data: &[u8]) -> Option<Self> {
        if data.len() == 4 {
            Some(i32::from_be_bytes([data[0], data[1], data[2], data[3]]))
        } else {
            None
        }
    }
}

impl OptionDefined for u32 {
    fn encode(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn decode(data: &[u8]) -> Option<Self> {
        if data.len() == 4 {
            Some(u32::from_be_bytes([data[0], data[1], data[2], data[3]]))
        } else {
            None
        }
    }
}

/// AutoConfigure option values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum AutoConfig {
    /// Do not autoconfig
    DoNotAutoConfigure = 0,
    /// autoconfig
    AutoConfigure = 1,
}

impl AutoConfig {
    fn from(value: u8) -> Option<Self> {
        match value {
            0 => Some(AutoConfig::DoNotAutoConfigure),
            1 => Some(AutoConfig::AutoConfigure),
            _ => None,
        }
    }
}

impl OptionDefined for AutoConfig {
    fn encode(&self) -> Vec<u8> {
        vec![self.clone() as u8]
    }

    fn decode(data: &[u8]) -> Option<Self> {
        AutoConfig::from(data[0])
    }
}

impl OptionDefined for String {
    fn encode(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }

    fn decode(data: &[u8]) -> Option<Self> {
        Some(String::from_utf8_lossy(data).to_string())
    }
}

impl OptionDefined for u16 {
    fn encode(&self) -> Vec<u8> {
        self.octets().to_vec()
    }

    fn decode(data: &[u8]) -> Option<Self> {
        Some(u16::from_be_bytes([data[0], data[1]]))
    }
}

impl OptionDefined for Vec<u8> {
    fn encode(&self) -> Vec<u8> {
        self.clone()
    }

    fn decode(data: &[u8]) -> Option<Self> {
        Some(data.to_vec())
    }
}

impl OptionDefined for Vec<Ipv4Addr> {
    fn encode(&self) -> Vec<u8> {
        // let length = (self.len() as u8) * 4;
        let mut data = vec![];
        for each in self.iter() {
            data.extend_from_slice(&each.encode());
        }
        data
    }

    fn decode(data: &[u8]) -> Option<Self> {
        let mut result = vec![];
        for data in data.chunks(4) {
            if let Some(ip) = Ipv4Addr::decode(data) {
                result.push(ip);
            } else {
                return None;
            }
        }
        Some(result)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[repr(u8)]
pub enum DhcpOptionMessageType {
    Discover = 1,
    Offer,
    Request,
    Decline,
    Ack,
    Nak,
    Release,
    Inform,
    ForceRenew,
    LeaseQuery,
    LeaseUnassigned,
    LeaseUnknown,
    LeaseActive,
    BulkLeaseQuery,
    LeaseQueryDone,
    ActiveLeaseQuery,
    LeaseQueryStatus,
    Tls,
}
impl DhcpOptionMessageType {
    fn from(value: u8) -> Option<Self> {
        match value {
            1 => Some(DhcpOptionMessageType::Discover),
            2 => Some(DhcpOptionMessageType::Offer),
            3 => Some(DhcpOptionMessageType::Request),
            4 => Some(DhcpOptionMessageType::Decline),
            5 => Some(DhcpOptionMessageType::Ack),
            6 => Some(DhcpOptionMessageType::Nak),
            7 => Some(DhcpOptionMessageType::Release),
            8 => Some(DhcpOptionMessageType::Inform),
            9 => Some(DhcpOptionMessageType::ForceRenew),
            10 => Some(DhcpOptionMessageType::LeaseQuery),
            11 => Some(DhcpOptionMessageType::LeaseUnassigned),
            12 => Some(DhcpOptionMessageType::LeaseUnknown),
            13 => Some(DhcpOptionMessageType::LeaseActive),
            14 => Some(DhcpOptionMessageType::BulkLeaseQuery),
            15 => Some(DhcpOptionMessageType::LeaseQueryDone),
            16 => Some(DhcpOptionMessageType::ActiveLeaseQuery),
            17 => Some(DhcpOptionMessageType::LeaseQueryStatus),
            18 => Some(DhcpOptionMessageType::Tls),
            _ => None,
        }
    }
}

impl OptionDefined for DhcpOptionMessageType {
    fn encode(&self) -> Vec<u8> {
        let value = self.clone() as u8;
        vec![value]
    }

    fn decode(data: &[u8]) -> Option<Self> {
        DhcpOptionMessageType::from(data[0])
    }
}

impl OptionDefined for Vec<Name> {
    fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        let mut name_encoder = hickory_proto::serialize::binary::BinEncoder::new(&mut buf);
        for name in self.iter() {
            hickory_proto::serialize::binary::BinEncodable::emit(name, &mut name_encoder).unwrap();
        }
        buf
    }

    fn decode(data: &[u8]) -> Option<Self> {
        let mut name_decoder = hickory_proto::serialize::binary::BinDecoder::new(data);
        let mut names = Vec::new();
        while let Ok(name) =
            <Name as hickory_proto::serialize::binary::BinDecodable>::read(&mut name_decoder)
        {
            names.push(name);
        }
        Some(names)
    }
}

#[cfg(test)]
mod tests {}
