use sea_orm::prelude::Uuid;

pub mod dhcp_v4_server;
pub mod dhcp_v6_client;
pub mod dns;
pub mod error;
pub mod firewall;
pub mod flow_wan;
pub mod iface;
pub mod mss_clamp;
pub mod provider;

/// 定义 ID 类型
pub(crate) type DBId = Uuid;
/// 定义 JSON
pub(crate) type DBJson = serde_json::Value;
/// 定义通用时间戳存储, 用于乐观锁判断
pub(crate) type DBTimestamp = f64;
