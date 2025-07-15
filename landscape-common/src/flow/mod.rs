use std::net::IpAddr;

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

use crate::database::repository::LandscapeDBStore;
use crate::flow::mark::FlowDnsMark;
use crate::store::storev2::LandscapeStore;
use crate::utils::time::get_f64_timestamp;

pub mod mark;
pub mod target;

/// 流控配置结构体
#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(export, export_to = "common/flow.d.ts")]
pub struct FlowConfig {
    pub id: Option<Uuid>,
    /// 是否启用
    pub enable: bool,
    /// 流 ID
    pub flow_id: u32,
    /// 匹配规则
    pub flow_match_rules: Vec<PacketMatchMark>,
    /// 处理流量目标网卡, 目前只取第一个
    /// 暂定, 可能会移动到具体的网卡上进行设置
    pub flow_targets: Vec<FlowTarget>,
    /// 备注
    pub remark: String,
    #[serde(default = "get_f64_timestamp")]
    pub update_at: f64,
}

impl LandscapeStore for FlowConfig {
    fn get_store_key(&self) -> String {
        self.flow_id.to_string()
    }
}

impl LandscapeDBStore<Uuid> for FlowConfig {
    fn get_id(&self) -> Uuid {
        self.id.unwrap_or(Uuid::new_v4())
    }
}

/// 数据包匹配该流控标志
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash, TS)]
#[ts(export, export_to = "common/flow.d.ts")]
pub struct PacketMatchMark {
    pub ip: IpAddr,
    pub vlan_id: Option<u32>,
    pub qos: Option<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(export, export_to = "common/flow.d.ts")]
#[serde(tag = "t")]
#[serde(rename_all = "snake_case")]
pub enum FlowTarget {
    Interface { name: String },
    Netns { container_name: String },
}

/// 用于 Flow ebpf 匹配记录操作
pub struct FlowMathPair {
    pub match_rule: PacketMatchMark,
    pub flow_id: u32,
}

/// 用于 Flow ebpf DNS Map 记录操作
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FlowDnsMarkInfo {
    pub ip: IpAddr,
    pub mark: u32,
    pub priority: u16,
}

#[derive(Debug, Clone)]
pub struct DnsRuntimeMarkInfo {
    pub mark: FlowDnsMark,
    pub priority: u16,
}
