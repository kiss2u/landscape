use serde::{Deserialize, Serialize};

use crate::{mark::PacketMark, store::storev2::LandScapeStore};

#[derive(Serialize, Deserialize, Debug, Clone)]
/// DNS 配置
pub struct DNSRuleConfig {
    pub name: String,
    // 优先级
    pub index: u32,
    // 是否启用
    pub enable: bool,
    /// 是否是重定向域名
    pub redirection: bool,
    // 配置使用的 DNS 解析服务器
    #[serde(default = "default_dns")]
    pub dns_resolve_ip: String,
    /// 流量标记
    #[serde(default)]
    pub mark: PacketMark,
    /// 匹配规则列表
    #[serde(default)]
    pub source: Vec<RuleSource>,
}

fn default_dns() -> String {
    "1.1.1.1".to_string()
}

impl LandScapeStore for DNSRuleConfig {
    fn get_store_key(&self) -> String {
        self.index.to_string()
    }
}

impl Default for DNSRuleConfig {
    fn default() -> Self {
        Self {
            name: "default rule".into(),
            index: 10000,
            enable: true,
            redirection: false,
            dns_resolve_ip: Default::default(),
            mark: Default::default(),
            source: vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "t")]
#[serde(rename_all = "lowercase")]
pub enum RuleSource {
    GeoKey { key: String },
    Config(DomainConfig),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DomainConfig {
    pub match_type: DomainMatchType,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum DomainMatchType {
    /// The value is used as is.
    Plain = 0,
    /// The value is used as a regular expression.
    Regex = 1,
    /// 域名匹配， 前缀匹配
    Domain = 2,
    /// The value is a domain.
    Full = 3,
}
