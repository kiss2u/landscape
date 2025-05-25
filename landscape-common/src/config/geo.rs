use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

use crate::{database::repository::LandscapeDBStore, store::storev2::LandscapeStore};

use super::dns::DomainConfig;

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(export, export_to = "common/geo_site.ts")]
pub struct GeoSiteConfig {
    /// 用这个 ID 作为文件名称
    pub id: Option<Uuid>,
    /// 记录更新时间
    pub update_at: f64,
    /// 文件 URL 地址
    pub url: String,
    /// 展示名称
    pub name: String,
    /// 启用状态
    pub enable: bool,
    /// 下次更新时间
    pub next_update_at: f64,
    /// 提取文件中的 key
    pub geo_keys: Vec<String>,
}

impl LandscapeDBStore<Uuid> for GeoSiteConfig {
    fn get_id(&self) -> Uuid {
        self.id.unwrap_or(Uuid::new_v4())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export, export_to = "common/geo_site.ts")]
pub struct GeoDomainConfig {
    pub name: String,
    pub key: String,
    pub values: Vec<DomainConfig>,
}

impl LandscapeStore for GeoDomainConfig {
    fn get_store_key(&self) -> String {
        format!("{}-{}", self.name, self.key)
    }
}
