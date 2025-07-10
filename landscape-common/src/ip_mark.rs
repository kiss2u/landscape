use std::net::IpAddr;

use crate::config::geo::GeoConfigKey;
use crate::utils::time::get_f64_timestamp;
use crate::{database::repository::LandscapeDBStore, flow::mark::FlowDnsMark};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export, export_to = "common/flow.d.ts")]
/// 对于外部 IP 规则
pub struct WanIpRuleConfig {
    pub id: Option<Uuid>,
    // 优先级 用作存储主键
    pub index: u32,
    // 是否启用
    pub enable: bool,
    /// 流量标记
    #[serde(default)]
    pub mark: FlowDnsMark,
    /// 匹配规则列表
    #[serde(default)]
    pub source: Vec<WanIPRuleSource>,
    // 备注
    pub remark: String,

    #[serde(default = "default_flow_id")]
    pub flow_id: u32,

    #[serde(default)]
    pub override_dns: bool,

    #[serde(default = "get_f64_timestamp")]
    pub update_at: f64,
}

fn default_flow_id() -> u32 {
    0_u32
}

impl LandscapeDBStore<Uuid> for WanIpRuleConfig {
    fn get_id(&self) -> Uuid {
        self.id.unwrap_or(Uuid::new_v4())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export, export_to = "flow.ts")]
#[serde(tag = "t")]
#[serde(rename_all = "snake_case")]
pub enum WanIPRuleSource {
    GeoKey(GeoConfigKey),
    Config(IpConfig),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, TS)]
#[ts(export, export_to = "flow.ts")]
pub struct IpConfig {
    pub ip: IpAddr,
    pub prefix: u32,
    // pub reverse_match: String,
}

/// IP 标记最小单元
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct IpMarkInfo {
    pub mark: FlowDnsMark,
    pub cidr: IpConfig,
    pub override_dns: bool,
}
