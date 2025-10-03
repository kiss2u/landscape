use std::net::IpAddr;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{flow::mark::FlowMark, net::MacAddr};

pub mod config;
pub mod mark;
pub mod target;

/// Flow 入口匹配规则
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash, TS)]
#[ts(export, export_to = "common/flow.d.ts")]
pub struct FlowEntryRule {
    // pub vlan_id: Option<u32>,
    #[serde(default)]
    pub qos: Option<u32>,
    pub mode: FlowEntryMatchMode,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash, TS)]
#[ts(export, export_to = "common/flow.d.ts")]
#[serde(tag = "t")]
#[serde(rename_all = "snake_case")]
pub enum FlowEntryMatchMode {
    Mac {
        mac_addr: MacAddr,
    },
    Ip {
        ip: IpAddr,
        #[serde(default = "default_prefix_len")]
        prefix_len: u8,
    },
}

/// 用于 Flow ebpf 匹配记录操作
pub struct FlowEbpfMatchPair {
    pub entry_rule: FlowEntryRule,
    pub flow_id: u32,
}

impl FlowEbpfMatchPair {
    pub fn new(entry_rule: FlowEntryRule, flow_id: u32) -> Self {
        Self { entry_rule, flow_id }
    }
}

fn default_prefix_len() -> u8 {
    32
}

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(export, export_to = "common/flow.d.ts")]
#[serde(tag = "t")]
#[serde(rename_all = "snake_case")]
pub enum FlowTarget {
    Interface { name: String },
    Netns { container_name: String },
}

/// 用于 Flow ebpf DNS Map 记录操作
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FlowMarkInfo {
    pub ip: IpAddr,
    pub mark: u32,
    pub priority: u16,
}

#[derive(Debug, Clone)]
pub struct DnsRuntimeMarkInfo {
    pub mark: FlowMark,
    pub priority: u16,
}
