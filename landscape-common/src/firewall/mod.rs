use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use crate::{
    args::LAND_ARGS, mark::PacketMark, network::LandscapeIpProtocolCode,
    store::storev2::LandscapeStore, LANDSCAPE_DEFAULE_DHCP_V6_CLIENT_PORT,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FirewallRuleConfig {
    // 优先级 用作存储主键
    pub index: u32,
    pub enable: bool,

    pub remark: String,
    pub items: Vec<FirewallRuleConfigItem>,
    /// 流量标记
    #[serde(default)]
    pub mark: PacketMark,
}

impl LandscapeStore for FirewallRuleConfig {
    fn get_store_key(&self) -> String {
        self.index.to_string()
    }
}

/// 配置的小项
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub struct FirewallRuleConfigItem {
    // IP 承载的协议
    pub ip_protocol: Option<LandscapeIpProtocolCode>,
    pub local_port: Option<String>,
    pub address: IpAddr,
    pub ip_prefixlen: u8,
}

/// 存入 bpf map 中的遍历项
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub struct FirewallRuleItem {
    pub ip_protocol: Option<LandscapeIpProtocolCode>,
    pub local_port: Option<u16>,
    pub address: IpAddr,
    pub ip_prefixlen: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum LandscapeIpType {
    Ipv4 = 0,
    Ipv6 = 1,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct FirewallRuleMark {
    pub item: FirewallRuleItem,
    pub mark: PacketMark,
}

pub fn insert_default_firewall_rule() -> Option<FirewallRuleConfig> {
    let mut items = vec![];
    #[cfg(debug_assertions)]
    {
        items.push(FirewallRuleConfigItem {
            ip_protocol: Some(LandscapeIpProtocolCode::TCP),
            local_port: Some("22".to_string()),
            address: IpAddr::V6(Ipv6Addr::UNSPECIFIED),
            ip_prefixlen: 0,
        });
        items.push(FirewallRuleConfigItem {
            ip_protocol: Some(LandscapeIpProtocolCode::TCP),
            local_port: Some("22".to_string()),
            address: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            ip_prefixlen: 0,
        });

        items.push(FirewallRuleConfigItem {
            ip_protocol: Some(LandscapeIpProtocolCode::TCP),
            local_port: Some("5173".to_string()),
            address: IpAddr::V6(Ipv6Addr::UNSPECIFIED),
            ip_prefixlen: 0,
        });
        items.push(FirewallRuleConfigItem {
            ip_protocol: Some(LandscapeIpProtocolCode::TCP),
            local_port: Some("5173".to_string()),
            address: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            ip_prefixlen: 0,
        });

        items.push(FirewallRuleConfigItem {
            ip_protocol: Some(LandscapeIpProtocolCode::TCP),
            local_port: Some("5800".to_string()),
            address: IpAddr::V6(Ipv6Addr::UNSPECIFIED),
            ip_prefixlen: 0,
        });
        items.push(FirewallRuleConfigItem {
            ip_protocol: Some(LandscapeIpProtocolCode::TCP),
            local_port: Some("5800".to_string()),
            address: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            ip_prefixlen: 0,
        });
    }
    #[cfg(not(debug_assertions))]
    {}

    // DHCPv4 Client
    items.push(FirewallRuleConfigItem {
        ip_protocol: Some(LandscapeIpProtocolCode::UDP),
        local_port: Some("68".to_string()),
        address: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
        ip_prefixlen: 0,
    });

    // DHCPv6 PD Client
    items.push(FirewallRuleConfigItem {
        ip_protocol: Some(LandscapeIpProtocolCode::UDP),
        local_port: Some(format!("{}", LANDSCAPE_DEFAULE_DHCP_V6_CLIENT_PORT)),
        address: IpAddr::V6(Ipv6Addr::UNSPECIFIED),
        ip_prefixlen: 0,
    });

    if LAND_ARGS.export_manager {
        items.push(FirewallRuleConfigItem {
            ip_protocol: Some(LandscapeIpProtocolCode::TCP),
            local_port: Some(format!("{}", LAND_ARGS.port)),
            address: IpAddr::V6(Ipv6Addr::UNSPECIFIED),
            ip_prefixlen: 0,
        });
        items.push(FirewallRuleConfigItem {
            ip_protocol: Some(LandscapeIpProtocolCode::TCP),
            local_port: Some(format!("{}", LAND_ARGS.port)),
            address: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            ip_prefixlen: 0,
        });
    }

    if items.is_empty() {
        None
    } else {
        Some(FirewallRuleConfig {
            index: 1,
            enable: true,
            remark: "Landscape Router Default Firewall Rule".to_string(),
            items,
            mark: PacketMark::default(),
        })
    }
}
