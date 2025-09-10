use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "common/docker.d.ts")]
pub struct PullImageReq {
    pub image_name: String,
    pub tag: Option<String>,
}

pub struct PullImgTask {
    pub img_name: String,
    pub layer_current_info: Arc<RwLock<HashMap<Option<String>, PullImgTaskItem>>>,
}

#[derive(Default, Clone, Serialize, Deserialize, Debug, TS)]
#[ts(export, export_to = "common/docker.d.ts")]
pub struct PullImgTaskItem {
    pub id: Option<String>,
    pub current: Option<i64>,
    pub total: Option<i64>,
}

#[derive(Clone, Serialize, Debug)]
pub struct ImgPullEvent {
    pub img_name: String,
    pub id: Option<String>,
    pub current: Option<i64>,
    pub total: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "common/docker.d.ts")]
pub struct PullManagerInfo {
    pub tasks: HashMap<String, HashMap<Option<String>, PullImgTaskItem>>,
}
