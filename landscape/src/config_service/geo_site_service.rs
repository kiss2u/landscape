use landscape_common::{
    config::{
        dns::{DNSRuleConfig, DNSRuntimeRule, DomainConfig, RuleSource},
        geo::{GeoConfigKey, GeoDomainConfig},
    },
    database::LandscapeDBTrait,
    service::controller_service::ConfigController,
    utils::time::{get_f64_timestamp, MILL_A_DAY},
};
use uuid::Uuid;

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::{Duration, Instant},
};

use landscape_common::{
    args::LAND_HOME_PATH, config::geo::GeoSiteSourceConfig, event::dns::DnsEvent,
    store::storev3::StoreFileManager, LANDSCAPE_GEO_CACHE_TMP_DIR,
};
use landscape_database::{
    geo_site::repository::GeoSiteConfigRepository, provider::LandscapeDBServiceProvider,
};
use reqwest::Client;
use tokio::sync::{mpsc, Mutex};

const A_DAY: u64 = 60 * 60 * 24;

pub type GeoDomainCacheStore = Arc<Mutex<StoreFileManager<GeoConfigKey, GeoDomainConfig>>>;

#[derive(Clone)]
pub struct GeoSiteService {
    store: GeoSiteConfigRepository,
    file_cache: GeoDomainCacheStore,
    dns_events_tx: mpsc::Sender<DnsEvent>,
}

impl GeoSiteService {
    pub async fn new(
        store: LandscapeDBServiceProvider,
        dns_events_tx: mpsc::Sender<DnsEvent>,
    ) -> Self {
        let store = store.geo_site_rule_store();

        let file_cache = Arc::new(Mutex::new(StoreFileManager::new(
            LAND_HOME_PATH.join(LANDSCAPE_GEO_CACHE_TMP_DIR),
            "site".to_string(),
        )));

        let service = Self { store, file_cache, dns_events_tx };
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
        configs: Vec<DNSRuleConfig>,
    ) -> Vec<DNSRuntimeRule> {
        let mut lock = self.file_cache.lock().await;
        let mut result = Vec::with_capacity(configs.len());
        for config in configs.into_iter() {
            let mut usage_keys = HashSet::new();
            let mut source: HashSet<DomainConfig> = HashSet::new();

            let mut inverse_keys: HashMap<String, HashSet<String>> = HashMap::new();
            for each in config.source.into_iter() {
                match each {
                    RuleSource::GeoKey(k) if k.inverse => {
                        inverse_keys.entry(k.name).or_default().insert(k.key);
                    }
                    RuleSource::GeoKey(k) => {
                        if let Some(domains) = lock.get(&k) {
                            source.extend(domains.values.into_iter());
                        }
                        usage_keys.insert(k);
                    }
                    RuleSource::Config(c) => {
                        source.insert(c);
                    }
                }
            }

            if inverse_keys.len() > 0 {
                let all_keys: Vec<_> = lock.keys();
                tracing::debug!("all_keys {:?}", all_keys.len());
                tracing::debug!("{:?}", inverse_keys);
                for (inverse_key, excluded_names) in inverse_keys {
                    for key in all_keys.iter().filter(|k| k.name == inverse_key) {
                        if !excluded_names.contains(&key.key) {
                            if let Some(domains) = lock.get(key) {
                                usage_keys.insert(key.clone());
                                source.extend(domains.values.into_iter());
                            }
                            // } else {
                            //     tracing::debug!("excluded_names: {:#?}", key);
                        }
                    }
                }
                tracing::debug!("using key len: {:#?}", usage_keys.len());
            }

            result.push(DNSRuntimeRule {
                source: source.into_iter().collect(),
                id: config.id,
                name: config.name,
                index: config.index,
                enable: config.enable,
                filter: config.filter,
                resolve_mode: config.resolve_mode,
                mark: config.mark,
                flow_id: config.flow_id,
            });
        }
        result
    }

    pub async fn refresh(&self, force: bool) {
        // 读取当前规则
        let mut configs: Vec<GeoSiteSourceConfig> = self.store.list().await.unwrap();

        if !force {
            let now = get_f64_timestamp();
            configs = configs.into_iter().filter(|e| e.next_update_at < now).collect();
        } else {
            let mut file_cache_lock = self.file_cache.lock().await;
            file_cache_lock.truncate();
            drop(file_cache_lock);
        }

        let client = Client::new();
        for mut config in configs {
            let url = config.url.clone();

            tracing::debug!("download file: {}", url);
            let time = Instant::now();

            match client.get(&url).send().await {
                Ok(resp) if resp.status().is_success() => match resp.bytes().await {
                    Ok(bytes) => {
                        let result = landscape_protobuf::read_geo_sites_from_bytes(bytes).await;
                        // tracing::debug!("get response file: {:?}", result);

                        let mut file_cache_lock = self.file_cache.lock().await;
                        for (key, values) in result {
                            file_cache_lock.set(GeoDomainConfig {
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

                        let _ = self.dns_events_tx.send(DnsEvent::GeositeUpdated).await;
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

impl GeoSiteService {
    pub async fn list_all_keys(&self) -> Vec<GeoConfigKey> {
        let lock = self.file_cache.lock().await;
        lock.keys()
    }

    pub async fn get_cache_value_by_key(&self, key: &GeoConfigKey) -> Option<GeoDomainConfig> {
        let mut lock = self.file_cache.lock().await;
        lock.get(key)
    }

    pub async fn query_geo_by_name(&self, name: Option<String>) -> Vec<GeoSiteSourceConfig> {
        self.store.query_by_name(name).await.unwrap()
    }
}

#[async_trait::async_trait]
impl ConfigController for GeoSiteService {
    type Id = Uuid;

    type Config = GeoSiteSourceConfig;

    type DatabseAction = GeoSiteConfigRepository;

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
