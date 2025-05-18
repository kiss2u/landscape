use sea_orm::prelude::Uuid;

pub mod entity;
pub mod error;
pub mod provider;
pub mod repository;

/// 定义 ID 类型
pub(crate) type DBId = Uuid;
/// 定义 JSON
pub(crate) type DBJson = serde_json::Value;
/// 定义通用时间戳存储, 用于乐观锁判断
pub(crate) type DBTimestamp = f64;
