use landscape_common::{
    config::geo::{GeoConfigKey, GeoIpConfig, GeoIpSourceConfig},
    database::LandscapeDBTrait,
    ip_mark::{IpMarkInfo, WanIPRuleSource, WanIpRuleConfig},
    service::controller_service::ConfigController,
    utils::time::{get_f64_timestamp, MILL_A_DAY},
};
use uuid::Uuid;

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use landscape_common::{
    args::LAND_HOME_PATH, event::dns::DstIpEvent, store::storev3::StoreFileManager,
    LANDSCAPE_GEO_CACHE_TMP_DIR,
};
use landscape_database::{
    geo_ip::repository::GeoIpSourceConfigRepository, provider::LandscapeDBServiceProvider,
};
use reqwest::Client;
use tokio::sync::{mpsc, Mutex};

const A_DAY: u64 = 60 * 60 * 24;

pub type GeoDomainCacheStore = Arc<Mutex<StoreFileManager<GeoConfigKey, GeoIpConfig>>>;

#[derive(Clone)]
pub struct GeoIpService {
    store: GeoIpSourceConfigRepository,
    file_cache: GeoDomainCacheStore,
    dst_ip_events_tx: mpsc::Sender<DstIpEvent>,
}

impl GeoIpService {
    pub async fn new(
        store: LandscapeDBServiceProvider,
        dst_ip_events_tx: mpsc::Sender<DstIpEvent>,
    ) -> Self {
        let store = store.geo_ip_rule_store();

        let file_cache = Arc::new(Mutex::new(StoreFileManager::new(
            LAND_HOME_PATH.join(LANDSCAPE_GEO_CACHE_TMP_DIR),
            "ip".to_string(),
        )));

        let service = Self { store, file_cache, dst_ip_events_tx };
        let service_clone = service.clone();
        tokio::spawn(async move {
            //
            let mut ticker = tokio::time::interval(Duration::from_secs(A_DAY));
            loop {
                service_clone.refresh(false).await;
                // 等待下一次 tick
                ticker.tick().await;
            }
        });
        service
    }

    pub async fn convert_config_to_runtime_rule(
        &self,
        configs: Vec<WanIpRuleConfig>,
    ) -> Vec<IpMarkInfo> {
        let mut lock = self.file_cache.lock().await;
        let mut result = vec![];
        for config in configs.into_iter() {
            let mut source = vec![];
            for each in config.source.iter() {
                match each {
                    WanIPRuleSource::GeoKey(config_key) => {
                        if let Some(ips) = lock.get(config_key) {
                            source.extend(ips.values.iter().cloned());
                        }
                    }
                    WanIPRuleSource::Config(c) => {
                        source.push(c.clone());
                    }
                }
            }

            let ip_marks = source.into_iter().map(|cidr| IpMarkInfo {
                mark: config.mark,
                cidr,
                override_dns: config.override_dns,
            });
            result.extend(ip_marks);
        }
        result
    }

    pub async fn refresh(&self, force: bool) {
        // 读取当前规则
        let mut configs: Vec<GeoIpSourceConfig> = self.store.list().await.unwrap();

        if !force {
            let now = get_f64_timestamp();
            configs = configs.into_iter().filter(|e| e.next_update_at < now).collect();
        }

        let client = Client::new();
        for mut config in configs {
            let url = config.url.clone();

            tracing::debug!("download file: {}", url);
            let time = Instant::now();

            match client.get(&url).send().await {
                Ok(resp) if resp.status().is_success() => match resp.bytes().await {
                    Ok(bytes) => {
                        let result = landscape_protobuf::read_geo_ips_from_bytes(bytes).await;
                        // tracing::debug!("get response file: {:?}", result);

                        let mut file_cache_lock = self.file_cache.lock().await;
                        for (key, values) in result {
                            file_cache_lock.set(GeoIpConfig {
                                name: config.name.clone(),
                                key: key.to_ascii_uppercase(),
                                values,
                            });
                        }
                        drop(file_cache_lock);

                        config.next_update_at = get_f64_timestamp() + MILL_A_DAY as f64;
                        let _ = self.store.set(config).await;

                        tracing::debug!(
                            "handle file done: {}, time: {}s",
                            url,
                            time.elapsed().as_secs()
                        );
                        let _ = self.dst_ip_events_tx.send(DstIpEvent::GeoIpUpdated).await;
                    }
                    Err(e) => tracing::error!("read {} response error: {}", url, e),
                },
                Ok(resp) => {
                    tracing::error!("download {} error, HTTP status: {}", url, resp.status());
                }
                Err(e) => {
                    tracing::error!("request {} error: {}", url, e);
                }
            }
        }
    }
}

impl GeoIpService {
    pub async fn list_all_keys(&self) -> Vec<GeoConfigKey> {
        let lock = self.file_cache.lock().await;
        lock.keys()
    }

    pub async fn get_cache_value_by_key(&self, key: &GeoConfigKey) -> Option<GeoIpConfig> {
        let mut lock = self.file_cache.lock().await;
        lock.get(key)
    }

    pub async fn query_geo_by_name(&self, name: Option<String>) -> Vec<GeoIpSourceConfig> {
        self.store.query_by_name(name).await.unwrap()
    }
}

#[async_trait::async_trait]
impl ConfigController for GeoIpService {
    type Id = Uuid;

    type Config = GeoIpSourceConfig;

    type DatabseAction = GeoIpSourceConfigRepository;

    fn get_repository(&self) -> &Self::DatabseAction {
        &self.store
    }

    async fn after_update_config(
        &self,
        _new_configs: Vec<Self::Config>,
        _old_configs: Vec<Self::Config>,
    ) {
    }
}
