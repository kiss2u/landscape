use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::net::IpAddr;
use uuid::Uuid;

use crate::database::repository::LandscapeDBStore;
use crate::utils::id::gen_database_uuid;
use crate::utils::time::get_f64_timestamp;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum IpFamily {
    Ipv4,
    Ipv6,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(tag = "t", rename_all = "snake_case")]
pub enum DdnsSource {
    LocalWan { iface_name: String, family: IpFamily },
    EnrolledDevice { device_id: Uuid, family: IpFamily },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum DdnsJobStatus {
    Idle,
    Syncing,
    Success,
    Error,
}

impl Default for DdnsJobStatus {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DdnsRecordConfig {
    pub name: String,
    #[serde(default = "default_enable")]
    pub enable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DdnsFamilyRuntime {
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(required = false, nullable = false, value_type = String))]
    pub last_published_ip: Option<IpAddr>,
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(required = false, nullable = false))]
    pub last_sync_at: Option<f64>,
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(required = false, nullable = false))]
    pub last_error: Option<String>,
    #[serde(default)]
    pub status: DdnsJobStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DdnsRecordRuntime {
    pub name: String,
    pub ipv4: DdnsFamilyRuntime,
    pub ipv6: DdnsFamilyRuntime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DdnsJobRuntime {
    #[cfg_attr(feature = "openapi", schema(value_type = String))]
    pub job_id: Uuid,
    pub status: DdnsJobStatus,
    pub records: Vec<DdnsRecordRuntime>,
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(required = false, nullable = false))]
    pub last_update_at: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DdnsJob {
    #[serde(default = "gen_database_uuid")]
    #[cfg_attr(feature = "openapi", schema(required = false))]
    pub id: Uuid,
    pub name: String,
    #[serde(default = "default_enable")]
    pub enable: bool,
    pub sources: Vec<DdnsSource>,
    pub zone_name: String,
    pub provider_profile_id: Uuid,
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(required = false, nullable = false))]
    pub ttl: Option<u32>,
    #[serde(default)]
    pub records: Vec<DdnsRecordConfig>,
    #[serde(default = "get_f64_timestamp")]
    #[cfg_attr(feature = "openapi", schema(required = false))]
    pub update_at: f64,
}

fn default_enable() -> bool {
    true
}

impl LandscapeDBStore<Uuid> for DdnsJob {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_update_at(&self) -> f64 {
        self.update_at
    }

    fn set_update_at(&mut self, ts: f64) {
        self.update_at = ts;
    }
}

impl DdnsJob {
    pub fn validate(&self) -> Result<(), String> {
        let zone_name = normalize_zone_name(&self.zone_name)?;
        if let Some(ttl) = self.ttl {
            if ttl == 0 {
                return Err("ttl must be greater than 0 when provided".to_string());
            }
        }
        if self.records.is_empty() {
            return Err("at least one DDNS record is required".to_string());
        }
        if self.sources.is_empty() {
            return Err("at least one DDNS source is required".to_string());
        }

        for source in &self.sources {
            match source {
                DdnsSource::LocalWan { iface_name, .. } => {
                    if iface_name.trim().is_empty() {
                        return Err("DDNS source iface_name must not be empty".to_string());
                    }
                }
                DdnsSource::EnrolledDevice { .. } => {}
            }
        }

        let mut seen = HashSet::new();
        for record in &self.records {
            let normalized = normalize_record_name(&record.name)?;
            if !seen.insert(normalized.clone()) {
                return Err(format!(
                    "duplicate DDNS record '{}' under zone '{}'",
                    normalized, zone_name
                ));
            }
        }
        Ok(())
    }
}

impl DdnsJobRuntime {
    pub fn from_config(job: &DdnsJob) -> Self {
        Self {
            job_id: job.id,
            status: DdnsJobStatus::Idle,
            records: job
                .records
                .iter()
                .map(|record| DdnsRecordRuntime {
                    name: record.name.clone(),
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
                })
                .collect(),
            last_update_at: None,
        }
    }
}

pub fn normalize_zone_name(zone_name: &str) -> Result<String, String> {
    let zone_name = zone_name.trim().trim_end_matches('.').to_ascii_lowercase();
    if zone_name.is_empty() {
        return Err("zone_name must not be empty".to_string());
    }
    if zone_name.contains('*') {
        return Err("zone_name must not contain wildcard characters".to_string());
    }
    if zone_name.split('.').any(|label| label.is_empty()) {
        return Err(format!("invalid zone_name '{zone_name}'"));
    }
    Ok(zone_name)
}

pub fn normalize_record_name(name: &str) -> Result<String, String> {
    let name = name.trim().trim_end_matches('.').to_ascii_lowercase();
    if name.is_empty() {
        return Err("record name must not be empty".to_string());
    }
    if name == "@" || name == "*" {
        return Ok(name);
    }

    let labels: Vec<&str> = name.split('.').collect();
    if labels.iter().any(|label| label.is_empty()) {
        return Err(format!("invalid DDNS record name '{name}'"));
    }
    for (idx, label) in labels.iter().enumerate() {
        if *label == "*" {
            if idx != 0 {
                return Err(format!(
                    "wildcard DDNS record '{name}' must only appear as the leading label"
                ));
            }
            continue;
        }
        if label.contains('*') {
            return Err(format!("invalid wildcard DDNS record '{name}'"));
        }
    }
    Ok(name)
}

pub fn fqdn_for_zone_record(zone_name: &str, record_name: &str) -> Result<String, String> {
    let zone_name = normalize_zone_name(zone_name)?;
    let record_name = normalize_record_name(record_name)?;
    if record_name == "@" {
        Ok(zone_name)
    } else {
        Ok(format!("{record_name}.{zone_name}"))
    }
}
