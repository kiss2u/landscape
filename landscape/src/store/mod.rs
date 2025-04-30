use std::{path::PathBuf, sync::Arc};

use landscape_common::store::storev2::StoreFileManager;
use tokio::sync::Mutex;

use crate::{boot::InitConfig, iface::config::NetworkIfaceConfig};

pub type LockStore<T> = Arc<Mutex<StoreFileManager<T>>>;

/// 存储服务提供者
#[derive(Debug, Clone)]
pub struct LandscapeStoreServiceProvider {
    pub iface_store: LockStore<NetworkIfaceConfig>,
}

impl LandscapeStoreServiceProvider {
    pub fn new(home_path: PathBuf) -> Self {
        //

        let iface_store: StoreFileManager<NetworkIfaceConfig> =
            StoreFileManager::new(home_path.clone(), "iface".to_string());

        LandscapeStoreServiceProvider { iface_store: Arc::new(Mutex::new(iface_store)) }
    }

    /// 清空数据并且从配置从初始化
    pub async fn truncate_and_fit_from(&mut self, config: Option<InitConfig>) {
        if let Some(config) = config {
            let mut iface_store = self.iface_store.lock().await;
            iface_store.truncate();
            for each_config in config.ifaces {
                iface_store.set(each_config);
            }
        }
    }
}

// trait LandscapeStoreService {
//     fn get_iface_store(&self) -> LockStore<NetworkIfaceConfig>;
// }
