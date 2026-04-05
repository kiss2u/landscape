use landscape_common::cert::CertError;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use reqwest::Client;
use serde::Deserialize;

use super::common::{record_name, RecordStore};
use super::{DnsChallengeSolver, DnsRecordUpdater};

const CF_API_BASE: &str = "https://api.cloudflare.com/client/v4";

pub struct CloudflareSolver {
    client: Client,
    api_token: String,
    base_url: String,
    records: RecordStore<(String, String)>,
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

impl CloudflareSolver {
    pub fn new(api_token: String) -> Self {
        Self::with_base_url(api_token, CF_API_BASE)
    }

    pub fn with_base_url(api_token: String, base_url: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            api_token,
            base_url: base_url.into(),
            records: RecordStore::new(),
        }
    }

    fn auth_header(&self) -> String {
        format!("Bearer {}", self.api_token)
    }

    fn api_url(&self, path: &str) -> String {
        format!("{}{}", self.base_url.trim_end_matches('/'), path)
    }

    fn cf_error(resp: &Option<Vec<CfError>>) -> String {
        resp.as_ref()
            .and_then(|errs| errs.first().map(|e| e.message.clone()))
            .unwrap_or_else(|| "unknown Cloudflare API error".to_string())
    }

    async fn find_zone_id(&self, domain: &str) -> Result<String, CertError> {
        for candidate in super::common::candidate_zones(domain) {
            let url = self.api_url(&format!("/zones?name={candidate}"));
            let resp = self
                .client
                .get(url)
                .header(AUTHORIZATION, self.auth_header())
                .send()
                .await
                .map_err(|e| {
                    CertError::DnsChallengeSetupFailed(format!(
                        "Cloudflare API request failed: {e}"
                    ))
                })?;

            let text = resp.text().await.map_err(|e| {
                CertError::DnsChallengeSetupFailed(format!(
                    "Failed to read Cloudflare response: {e}"
                ))
            })?;

            let body: CfResponse<Vec<CfZone>> = serde_json::from_str(&text).map_err(|e| {
                CertError::DnsChallengeSetupFailed(format!(
                    "Failed to parse Cloudflare response: {e}"
                ))
            })?;

            if body.success {
                if let Some(zone) = body.result.and_then(|zones| zones.into_iter().next()) {
                    return Ok(zone.id);
                }
            }
        }

        Err(CertError::DnsChallengeSetupFailed(format!(
            "Could not find Cloudflare zone for domain: {domain}"
        )))
    }

    async fn upsert_dns_record(
        &self,
        zone_name: &str,
        record_name: &str,
        value: &str,
        record_type: &str,
        ttl: u32,
    ) -> Result<(), CertError> {
        let zone_id = self.find_zone_id(zone_name).await?;
        let fqdn = if record_name == "@" {
            zone_name.to_string()
        } else {
            format!("{record_name}.{zone_name}")
        };
        let list_url =
            self.api_url(&format!("/zones/{zone_id}/dns_records?type={record_type}&name={fqdn}"));
        let list_resp = self
            .client
            .get(list_url)
            .header(AUTHORIZATION, self.auth_header())
            .send()
            .await
            .map_err(|e| {
                CertError::DnsChallengeSetupFailed(format!("Cloudflare API request failed: {e}"))
            })?;
        let list_text = list_resp.text().await.map_err(|e| {
            CertError::DnsChallengeSetupFailed(format!("Failed to read Cloudflare response: {e}"))
        })?;
        let list_body: CfResponse<Vec<CfDnsRecord>> =
            serde_json::from_str(&list_text).map_err(|e| {
                CertError::DnsChallengeSetupFailed(format!(
                    "Failed to parse Cloudflare response: {e}"
                ))
            })?;
        if !list_body.success {
            return Err(CertError::DnsChallengeSetupFailed(format!(
                "Cloudflare DNS record lookup failed: {}",
                Self::cf_error(&list_body.errors)
            )));
        }

        let payload = serde_json::json!({
            "type": record_type,
            "name": fqdn,
            "content": value,
            "ttl": ttl,
            "proxied": false,
        });
        let existing_id =
            list_body.result.and_then(|records| records.into_iter().next()).map(|r| r.id);
        let url = if let Some(ref record_id) = existing_id {
            self.api_url(&format!("/zones/{zone_id}/dns_records/{record_id}"))
        } else {
            self.api_url(&format!("/zones/{zone_id}/dns_records"))
        };
        let request =
            if existing_id.is_some() { self.client.put(url) } else { self.client.post(url) };
        let resp = request
            .header(AUTHORIZATION, self.auth_header())
            .header(CONTENT_TYPE, "application/json")
            .body(payload.to_string())
            .send()
            .await
            .map_err(|e| {
                CertError::DnsChallengeSetupFailed(format!("Cloudflare update failed: {e}"))
            })?;
        let text = resp.text().await.map_err(|e| {
            CertError::DnsChallengeSetupFailed(format!("Failed to read Cloudflare response: {e}"))
        })?;
        let body: CfResponse<serde_json::Value> = serde_json::from_str(&text).map_err(|e| {
            CertError::DnsChallengeSetupFailed(format!("Failed to parse Cloudflare response: {e}"))
        })?;
        if body.success {
            Ok(())
        } else {
            Err(CertError::DnsChallengeSetupFailed(format!(
                "Cloudflare DNS update failed: {}",
                Self::cf_error(&body.errors)
            )))
        }
    }
}

#[async_trait::async_trait]
impl DnsRecordUpdater for CloudflareSolver {
    async fn upsert_record(
        &self,
        zone_name: &str,
        record_name: &str,
        value: &str,
        record_type: &str,
        ttl: u32,
    ) -> Result<(), CertError> {
        self.upsert_dns_record(zone_name, record_name, value, record_type, ttl).await
    }
}

#[async_trait::async_trait]
impl DnsChallengeSolver for CloudflareSolver {
    async fn create_txt_record(&self, domain: &str, value: &str) -> Result<(), CertError> {
        let zone_id = self.find_zone_id(domain).await?;
        let record_name = record_name(domain);
        let url = self.api_url(&format!("/zones/{zone_id}/dns_records"));
        let payload = serde_json::json!({
            "type": "TXT",
            "name": record_name,
            "content": value,
            "ttl": 120
        });

        let resp = self
            .client
            .post(url)
            .header(AUTHORIZATION, self.auth_header())
            .header(CONTENT_TYPE, "application/json")
            .body(payload.to_string())
            .send()
            .await
            .map_err(|e| {
                CertError::DnsChallengeSetupFailed(format!("Failed to create DNS record: {e}"))
            })?;

        let text = resp.text().await.map_err(|e| {
            CertError::DnsChallengeSetupFailed(format!("Failed to read Cloudflare response: {e}"))
        })?;

        let body: CfResponse<CfDnsRecord> = serde_json::from_str(&text).map_err(|e| {
            CertError::DnsChallengeSetupFailed(format!("Failed to parse Cloudflare response: {e}"))
        })?;

        if !body.success {
            return Err(CertError::DnsChallengeSetupFailed(format!(
                "Cloudflare DNS record creation failed: {}",
                Self::cf_error(&body.errors)
            )));
        }

        if let Some(record) = body.result {
            self.records.insert(domain, value, (zone_id, record.id));
        }

        tracing::info!("Created Cloudflare TXT record for {domain}");
        Ok(())
    }

    async fn cleanup_txt_record(&self, domain: &str, value: &str) -> Result<(), CertError> {
        let Some((zone_id, record_id)) = self.records.get_cloned(domain, value) else {
            tracing::warn!("No Cloudflare TXT record found to clean up for {domain}");
            return Ok(());
        };

        let url = self.api_url(&format!("/zones/{zone_id}/dns_records/{record_id}"));
        let resp = self
            .client
            .delete(url)
            .header(AUTHORIZATION, self.auth_header())
            .send()
            .await
            .map_err(|e| {
            CertError::DnsChallengeSetupFailed(format!("Failed to delete DNS record: {e}"))
        })?;

        let text = resp.text().await.map_err(|e| {
            CertError::DnsChallengeSetupFailed(format!("Failed to read Cloudflare response: {e}"))
        })?;

        let body: CfResponse<serde_json::Value> = serde_json::from_str(&text).map_err(|e| {
            CertError::DnsChallengeSetupFailed(format!("Failed to parse Cloudflare response: {e}"))
        })?;

        if !body.success {
            tracing::warn!(
                "Failed to clean up Cloudflare TXT record for {domain}: {}",
                Self::cf_error(&body.errors)
            );
        } else {
            tracing::info!("Cleaned up Cloudflare TXT record for {domain}");
        }

        self.records.remove(domain, value);
        Ok(())
    }
}
