use landscape_common::{
    cert::order::DnsProviderConfig, dns::provider_profile::DnsProviderProfile, error::LdError,
    service::controller::ConfigController,
};
use landscape_database::{
    dns_provider_profile::repository::DnsProviderProfileRepository,
    provider::LandscapeDBServiceProvider,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct DnsProviderProfileService {
    store: DnsProviderProfileRepository,
}

impl DnsProviderProfileService {
    pub async fn new(store: LandscapeDBServiceProvider) -> Self {
        Self { store: store.dns_provider_profile_store() }
    }

    pub async fn checked_set_profile(
        &self,
        config: DnsProviderProfile,
    ) -> Result<DnsProviderProfile, LdError> {
        if matches!(config.provider_config, DnsProviderConfig::Manual) {
            return Err(LdError::ConfigError(
                "manual DNS provider cannot be used as a reusable DNS provider profile".to_string(),
            ));
        }
        if let Some(existing) = self.store.find_by_name(&config.name).await? {
            if existing.id != config.id {
                return Err(LdError::ConfigError(format!(
                    "DNS provider profile name '{}' already exists",
                    config.name
                )));
            }
        }
        self.checked_set(config).await
    }
}

#[async_trait::async_trait]
impl ConfigController for DnsProviderProfileService {
    type Id = Uuid;
    type Config = DnsProviderProfile;
    type DatabseAction = DnsProviderProfileRepository;

    fn get_repository(&self) -> &Self::DatabseAction {
        &self.store
    }
}
