use std::net::IpAddr;

use crate::{mark::PacketMark, store::storev2::LandScapeStore};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
/// 对于外部 IP 规则
pub struct WanIPRuleConfig {
    // 优先级 用作存储主键
    pub index: u32,
    // 是否启用
    pub enable: bool,
    /// 流量标记
    #[serde(default)]
    pub mark: PacketMark,
    /// 匹配规则列表
    #[serde(default)]
    pub source: Vec<WanIPRuleSource>,
    // 备注
    pub remark: String,
}

impl LandScapeStore for WanIPRuleConfig {
    fn get_store_key(&self) -> String {
        self.index.to_string()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// 对于外部 IP 规则
pub struct LanIPRuleConfig {
    // 优先级 用作存储主键
    pub index: u32,
    // 是否启用
    pub enable: bool,
    /// 流量标记
    #[serde(default)]
    pub mark: PacketMark,
    /// 匹配规则列表
    #[serde(default)]
    pub source: Vec<IpConfig>,
    // 备注
    pub remark: String,
}

impl LandScapeStore for LanIPRuleConfig {
    fn get_store_key(&self) -> String {
        self.index.to_string()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "t")]
#[serde(rename_all = "lowercase")]
pub enum WanIPRuleSource {
    GeoKey { country_code: String },
    Config(IpConfig),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct IpConfig {
    pub ip: IpAddr,
    pub prefix: u32,
    // pub reverse_match: String,
}

/// IP 标记最小单元
pub struct IpMarkInfo {
    pub mark: PacketMark,
    pub cidr: IpConfig,
}
