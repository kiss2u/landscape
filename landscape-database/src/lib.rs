use sea_orm::prelude::Uuid;

pub mod entity;
pub mod provider;
pub mod repository;

/// 定义 ID 类型
pub(crate) type DBId = Uuid;
/// 定义 JSON
pub(crate) type DBJson = serde_json::Value;
/// 定义通用时间存储
#[allow(dead_code)]
pub(crate) type DBTime = f64;
