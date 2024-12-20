use std::net::Ipv4Addr;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MarkRule {
    prefixlen: u32,
    addr: Ipv4Addr,
    mark: PacketMark,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(tag = "t")]
#[serde(rename_all = "lowercase")]
pub enum PacketMark {
    /// 默认情况
    /// 后续将会有 IP 的匹配, 再进行打标过滤
    #[default]
    NoMark,
    /// 直连
    Direct,
    /// 丢弃数据包
    Drop,
    /// 转发到指定 index 的网卡中
    Redirect { index: u8 },
    /// 进行 IP 校验 ( 阻止进行打洞 )
    SymmetricNat,
}

impl PacketMark {
    pub fn need_add_mark_config(&self) -> bool {
        match self {
            PacketMark::NoMark => false,
            _ => true,
        }
    }
}

const OK_MARK: u8 = 0;
const DIRECT_MARK: u8 = 1;
const DROP_MARK: u8 = 2;
const REDIRECT_MARK: u8 = 3;
const SYMMETRIC_NAT: u8 = 4;

const ACTION_MASK: u32 = 0x00FF;
const INDEX_MASK: u32 = 0xFF00;

impl From<u32> for PacketMark {
    fn from(value: u32) -> Self {
        let action = (value & ACTION_MASK) as u8;
        let index = ((value & INDEX_MASK) >> 8) as u8;

        match action {
            OK_MARK => PacketMark::NoMark,
            DIRECT_MARK => PacketMark::Direct,
            DROP_MARK => PacketMark::Drop,
            REDIRECT_MARK => PacketMark::Redirect { index },
            SYMMETRIC_NAT => PacketMark::SymmetricNat,
            _ => PacketMark::NoMark,
        }
    }
}

impl Into<u32> for PacketMark {
    fn into(self) -> u32 {
        match self {
            PacketMark::NoMark => OK_MARK as u32,
            PacketMark::Direct => DIRECT_MARK as u32,
            PacketMark::Drop => DROP_MARK as u32,
            PacketMark::Redirect { index } => REDIRECT_MARK as u32 | ((index as u32) << 8),
            PacketMark::SymmetricNat => SYMMETRIC_NAT as u32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packetmark_from_u32() {
        // action = 1 (Direct)
        assert_eq!(PacketMark::from(0x0001), PacketMark::Direct);
        // action = 3 (Redirect), index = 5
        assert_eq!(PacketMark::from(0x0503), PacketMark::Redirect { index: 5 });
        // action = 4 (SymmetricNat)
        assert_eq!(PacketMark::from(0x0004), PacketMark::SymmetricNat);
    }

    #[test]
    fn test_packetmark_into_u32() {
        // Direct -> action = 1
        let mark: u32 = PacketMark::Direct.into();
        assert_eq!(mark, 0x0001);

        // Redirect { index: 5 } -> action = 3, index = 5
        let mark: u32 = PacketMark::Redirect { index: 5 }.into();
        assert_eq!(mark, 0x0503);

        // SymmetricNat -> action = 4
        let mark: u32 = PacketMark::SymmetricNat.into();
        assert_eq!(mark, 0x0004);
    }
}
