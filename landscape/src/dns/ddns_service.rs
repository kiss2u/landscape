use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use landscape_common::cert::order::DnsProviderConfig;
use landscape_common::database::LandscapeStore;
use landscape_common::ddns::{
    fqdn_for_zone_record, DdnsFamilyRuntime, DdnsJob, DdnsJobRuntime, DdnsJobStatus,
    DdnsRecordRuntime, DdnsSource, IpFamily,
};
use landscape_common::{error::LdError, service::controller::ConfigController};
use landscape_database::{
    ddns::repository::DdnsJobRepository,
    dns_provider_profile::repository::DnsProviderProfileRepository,
    provider::LandscapeDBServiceProvider,
};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use reqwest::Client;
use serde::Deserialize;
use tokio::sync::RwLock;
use tokio::time::MissedTickBehavior;
use uuid::Uuid;

use crate::route::IpRouteService;

const DDNS_SYNC_INTERVAL_SECS: u64 = 60;

type DdnsRuntimeMap = Arc<RwLock<HashMap<Uuid, DdnsJobRuntime>>>;

#[derive(Clone)]
pub struct DdnsService {
    store: DdnsJobRepository,
    profile_store: DnsProviderProfileRepository,
    route_service: IpRouteService,
    client: Client,
    runtime: DdnsRuntimeMap,
}

impl DdnsService {
    pub async fn new(store: LandscapeDBServiceProvider, route_service: IpRouteService) -> Self {
        let service = Self {
            store: store.ddns_job_store(),
            profile_store: store.dns_provider_profile_store(),
            route_service,
            client: Client::new(),
            runtime: Arc::new(RwLock::new(HashMap::new())),
        };
        service.refresh_runtime_from_store().await;
        service.spawn_sync_loop();
        service
    }

    pub async fn get_runtime_statuses(&self) -> HashMap<Uuid, DdnsJobRuntime> {
        self.runtime.read().await.clone()
    }

    fn spawn_sync_loop(&self) {
        let service = self.clone();
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(Duration::from_secs(DDNS_SYNC_INTERVAL_SECS));
            ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
            loop {
                ticker.tick().await;
                if let Err(e) = service.sync_all_enabled_jobs().await {
                    tracing::warn!("ddns sync pass failed: {e:?}");
                }
            }
        });
    }

    pub async fn checked_set_job(&self, config: DdnsJob) -> Result<DdnsJob, LdError> {
        config.validate().map_err(LdError::ConfigError)?;
        self.profile_store.find_by_id(config.provider_profile_id).await?.ok_or_else(|| {
            LdError::ConfigError(format!(
                "DNS provider profile {} not found",
                config.provider_profile_id
            ))
        })?;
        let saved = self.checked_set(config).await?;
        self.sync_runtime_for_job(&saved).await;
        Ok(saved)
    }

    async fn refresh_runtime_from_store(&self) {
        let jobs = self.store.list().await.unwrap_or_default();
        let mut runtime = self.runtime.write().await;
        runtime.clear();
        for job in jobs {
            runtime.insert(job.id, DdnsJobRuntime::from_config(&job));
        }
    }

    async fn sync_runtime_for_job(&self, job: &DdnsJob) {
        let mut runtime = self.runtime.write().await;
        let next = if let Some(current) = runtime.remove(&job.id) {
            preserve_runtime(job, current)
        } else {
            DdnsJobRuntime::from_config(job)
        };
        runtime.insert(job.id, next);
    }

    async fn sync_all_enabled_jobs(&self) -> Result<(), LdError> {
        for job in self.store.find_enabled().await? {
            let runtime = self.sync_one_job(&job).await;
            self.runtime.write().await.insert(job.id, runtime);
        }
        Ok(())
    }

    async fn sync_one_job(&self, job: &DdnsJob) -> DdnsJobRuntime {
        let mut runtime = {
            let current = self.runtime.read().await.get(&job.id).cloned();
            current
                .map(|state| preserve_runtime(job, state))
                .unwrap_or_else(|| DdnsJobRuntime::from_config(job))
        };

        let profile = match self.profile_store.find_by_id(job.provider_profile_id).await {
            Ok(Some(profile)) => profile,
            Ok(None) => {
                apply_job_error(&mut runtime, "DNS provider profile not found".to_string());
                return runtime;
            }
            Err(e) => {
                apply_job_error(&mut runtime, e.to_string());
                return runtime;
            }
        };

        for record in &mut runtime.records {
            let enabled = job
                .records
                .iter()
                .find(|cfg| cfg.name == record.name)
                .map(|cfg| cfg.enable)
                .unwrap_or(false);
            if !enabled {
                record.ipv4.status = DdnsJobStatus::Idle;
                record.ipv6.status = DdnsJobStatus::Idle;
                continue;
            }

            for family in [IpFamily::Ipv4, IpFamily::Ipv6] {
                let current_ip = self.resolve_record_ip(&job.sources, family).await.ok();
                self.sync_one_record_family(
                    &profile.provider_config,
                    &job.zone_name,
                    record,
                    family,
                    current_ip,
                    job.ttl,
                )
                .await;
            }
        }

        runtime.last_update_at = Some(landscape_common::utils::time::get_f64_timestamp());
        runtime.status = aggregate_runtime_status(&runtime);
        runtime
    }

    async fn resolve_record_ip(
        &self,
        sources: &[DdnsSource],
        wanted_family: IpFamily,
    ) -> Result<IpAddr, String> {
        for source in sources {
            match source {
                DdnsSource::LocalWan { iface_name, family } if *family == wanted_family => {
                    let route = match family {
                        IpFamily::Ipv4 => self.route_service.get_ipv4_wan_route(iface_name).await,
                        IpFamily::Ipv6 => self.route_service.get_ipv6_wan_route(iface_name).await,
                    };
                    if let Some(route) = route {
                        return Ok(route.iface_ip);
                    }
                }
                DdnsSource::EnrolledDevice { .. } => {
                    return Err("enrolled device DDNS source is not implemented yet".to_string());
                }
                _ => {}
            }
        }

        Err(format!("no matching WAN route found for {:?}", wanted_family))
    }

    async fn sync_one_record_family(
        &self,
        provider: &DnsProviderConfig,
        zone_name: &str,
        record: &mut DdnsRecordRuntime,
        family: IpFamily,
        current_ip: Option<IpAddr>,
        ttl: Option<u32>,
    ) {
        let ts = landscape_common::utils::time::get_f64_timestamp();
        let family_runtime = match family {
            IpFamily::Ipv4 => &mut record.ipv4,
            IpFamily::Ipv6 => &mut record.ipv6,
        };

        let Some(current_ip) = current_ip else {
            family_runtime.status = DdnsJobStatus::Idle;
            return;
        };

        family_runtime.status = DdnsJobStatus::Syncing;
        family_runtime.last_sync_at = Some(ts);

        if family_runtime.last_published_ip == Some(current_ip) {
            family_runtime.status = DdnsJobStatus::Success;
            family_runtime.last_error = None;
            return;
        }

        match update_dns_record(&self.client, provider, zone_name, &record.name, current_ip, ttl)
            .await
        {
            Ok(()) => {
                family_runtime.last_published_ip = Some(current_ip);
                family_runtime.last_error = None;
                family_runtime.status = DdnsJobStatus::Success;
            }
            Err(e) => {
                family_runtime.last_error = Some(e);
                family_runtime.status = DdnsJobStatus::Error;
            }
        }
    }
}

fn preserve_runtime(job: &DdnsJob, current: DdnsJobRuntime) -> DdnsJobRuntime {
    let current_records: HashMap<String, DdnsRecordRuntime> = current
        .records
        .into_iter()
        .map(|record| (record.name.to_ascii_lowercase(), record))
        .collect();
    let mut next = DdnsJobRuntime::from_config(job);
    next.records = job
        .records
        .iter()
        .map(|cfg| {
            current_records.get(&cfg.name.to_ascii_lowercase()).cloned().unwrap_or_else(|| {
                DdnsRecordRuntime {
                    name: cfg.name.clone(),
                    ipv4: DdnsFamilyRuntime {
                        last_published_ip: None,
                        last_sync_at: None,
                        last_error: None,
                        status: DdnsJobStatus::Idle,
                    },
                    ipv6: DdnsFamilyRuntime {
                        last_published_ip: None,
                        last_sync_at: None,
                        last_error: None,
                        status: DdnsJobStatus::Idle,
                    },
                }
            })
        })
        .collect();
    next.status = aggregate_runtime_status(&next);
    next.last_update_at = current.last_update_at;
    next
}

fn apply_job_error(runtime: &mut DdnsJobRuntime, message: String) {
    let ts = landscape_common::utils::time::get_f64_timestamp();
    runtime.last_update_at = Some(ts);
    runtime.status = DdnsJobStatus::Error;
    for record in &mut runtime.records {
        record.ipv4.status = DdnsJobStatus::Error;
        record.ipv4.last_error = Some(message.clone());
        record.ipv4.last_sync_at = Some(ts);
        record.ipv6.status = DdnsJobStatus::Error;
        record.ipv6.last_error = Some(message.clone());
        record.ipv6.last_sync_at = Some(ts);
    }
}

fn aggregate_runtime_status(runtime: &DdnsJobRuntime) -> DdnsJobStatus {
    let all = runtime.records.iter().flat_map(|record| [&record.ipv4.status, &record.ipv6.status]);
    let statuses: Vec<DdnsJobStatus> = all.cloned().collect();
    if statuses.iter().any(|status| *status == DdnsJobStatus::Error) {
        DdnsJobStatus::Error
    } else if statuses.iter().any(|status| *status == DdnsJobStatus::Syncing) {
        DdnsJobStatus::Syncing
    } else if statuses.iter().any(|status| *status == DdnsJobStatus::Success) {
        DdnsJobStatus::Success
    } else {
        DdnsJobStatus::Idle
    }
}

async fn update_dns_record(
    client: &Client,
    provider: &DnsProviderConfig,
    zone_name: &str,
    record_name: &str,
    ip: IpAddr,
    ttl: Option<u32>,
) -> Result<(), String> {
    match provider {
        DnsProviderConfig::Cloudflare { api_token } => {
            update_cloudflare_record(client, api_token, zone_name, record_name, ip, ttl).await
        }
        DnsProviderConfig::Manual => {
            Err("manual DNS provider does not support DDNS updates".to_string())
        }
        _ => Err("selected DNS provider does not support DDNS updates yet".to_string()),
    }
}

async fn update_cloudflare_record(
    client: &Client,
    api_token: &str,
    zone_name: &str,
    record_name: &str,
    ip: IpAddr,
    ttl: Option<u32>,
) -> Result<(), String> {
    let zone_id = find_cloudflare_zone_id(client, api_token, zone_name).await?;
    let fqdn = fqdn_for_zone_record(zone_name, record_name)?;
    let record_type = match ip {
        IpAddr::V4(_) => "A",
        IpAddr::V6(_) => "AAAA",
    };
    let ttl = ttl.unwrap_or(120);
    let existing = find_cloudflare_record(client, api_token, &zone_id, &fqdn, record_type).await?;
    let payload = serde_json::json!({
        "type": record_type,
        "name": fqdn,
        "content": ip.to_string(),
        "ttl": ttl,
        "proxied": false,
    });

    let url = if let Some(ref record_id) = existing {
        format!("https://api.cloudflare.com/client/v4/zones/{zone_id}/dns_records/{record_id}")
    } else {
        format!("https://api.cloudflare.com/client/v4/zones/{zone_id}/dns_records")
    };
    let request = if existing.is_some() { client.put(url) } else { client.post(url) };
    let response = request
        .header(AUTHORIZATION, format!("Bearer {api_token}"))
        .header(CONTENT_TYPE, "application/json")
        .body(payload.to_string())
        .send()
        .await
        .map_err(|e| format!("Cloudflare request failed: {e}"))?;
    let text =
        response.text().await.map_err(|e| format!("Cloudflare response read failed: {e}"))?;
    let body: CfResponse<serde_json::Value> = serde_json::from_str(&text)
        .map_err(|e| format!("Cloudflare response parse failed: {e}"))?;
    if body.success {
        Ok(())
    } else {
        Err(cf_error(&body.errors))
    }
}

async fn find_cloudflare_zone_id(
    client: &Client,
    api_token: &str,
    zone: &str,
) -> Result<String, String> {
    let response = client
        .get(format!("https://api.cloudflare.com/client/v4/zones?name={zone}"))
        .header(AUTHORIZATION, format!("Bearer {api_token}"))
        .send()
        .await
        .map_err(|e| format!("Cloudflare zone lookup failed: {e}"))?;
    let text = response.text().await.map_err(|e| format!("Cloudflare zone read failed: {e}"))?;
    let body: CfResponse<Vec<CfZone>> =
        serde_json::from_str(&text).map_err(|e| format!("Cloudflare zone parse failed: {e}"))?;
    if body.success {
        body.result
            .and_then(|zones| zones.into_iter().next())
            .map(|zone| zone.id)
            .ok_or_else(|| format!("Cloudflare zone {zone} not found"))
    } else {
        Err(cf_error(&body.errors))
    }
}

async fn find_cloudflare_record(
    client: &Client,
    api_token: &str,
    zone_id: &str,
    record_name: &str,
    record_type: &str,
) -> Result<Option<String>, String> {
    let response = client
        .get(format!(
            "https://api.cloudflare.com/client/v4/zones/{zone_id}/dns_records?type={record_type}&name={record_name}"
        ))
        .header(AUTHORIZATION, format!("Bearer {api_token}"))
        .send()
        .await
        .map_err(|e| format!("Cloudflare record lookup failed: {e}"))?;
    let text = response.text().await.map_err(|e| format!("Cloudflare record read failed: {e}"))?;
    let body: CfResponse<Vec<CfDnsRecord>> =
        serde_json::from_str(&text).map_err(|e| format!("Cloudflare record parse failed: {e}"))?;
    if body.success {
        Ok(body.result.and_then(|records| records.into_iter().next()).map(|record| record.id))
    } else {
        Err(cf_error(&body.errors))
    }
}

#[async_trait::async_trait]
impl ConfigController for DdnsService {
    type Id = Uuid;
    type Config = DdnsJob;
    type DatabseAction = DdnsJobRepository;

    fn get_repository(&self) -> &Self::DatabseAction {
        &self.store
    }
}

#[derive(Deserialize)]
struct CfResponse<T> {
    success: bool,
    result: Option<T>,
    errors: Option<Vec<CfError>>,
}

#[derive(Deserialize)]
struct CfError {
    message: String,
}

#[derive(Deserialize)]
struct CfZone {
    id: String,
}

#[derive(Deserialize)]
struct CfDnsRecord {
    id: String,
}

fn cf_error(errors: &Option<Vec<CfError>>) -> String {
    errors
        .as_ref()
        .and_then(|errs| errs.first().map(|err| err.message.clone()))
        .unwrap_or_else(|| "unknown Cloudflare API error".to_string())
}
