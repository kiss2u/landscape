use std::time::Duration;

use landscape_common::config::ConfigId;
use landscape_common::database::LandscapeStore;
use landscape_common::error::LdError;
use landscape_common::gateway::settings::GatewayRuntimeConfig;
use landscape_common::gateway::HttpUpstreamRuleConfig;
use landscape_database::gateway::repository::GatewayHttpUpstreamRepository;
#[cfg(feature = "gateway")]
use landscape_database::repository::Repository;

#[cfg(feature = "gateway")]
#[derive(Debug, Clone)]
pub struct GatewayTlsConfig {
    inner: landscape_gateway::GatewayTlsConfig,
}

#[cfg(feature = "gateway")]
impl GatewayTlsConfig {
    pub fn new(server_config: std::sync::Arc<rustls::ServerConfig>) -> Self {
        Self {
            inner: landscape_gateway::GatewayTlsConfig { server_config },
        }
    }
}

#[cfg(feature = "gateway")]
#[derive(Clone)]
pub struct GatewayService {
    inner: landscape_gateway::service::GatewayService,
}

#[cfg(feature = "gateway")]
impl GatewayService {
    pub async fn init_service(
        store: GatewayHttpUpstreamRepository,
        config: GatewayRuntimeConfig,
        tls_config: Option<GatewayTlsConfig>,
    ) -> Self {
        let inner = landscape_gateway::service::GatewayService::init_service(
            store,
            config,
            tls_config.map(|config| config.inner),
        )
        .await;
        Self { inner }
    }

    pub fn is_supported(&self) -> bool {
        true
    }

    pub fn is_running(&self) -> bool {
        self.inner.is_running()
    }

    pub fn has_https_listener(&self) -> bool {
        self.inner.has_https_listener()
    }

    pub fn config(&self) -> &GatewayRuntimeConfig {
        self.inner.config()
    }

    pub async fn shutdown_and_wait(&self, timeout: Duration) {
        self.inner.shutdown_and_wait(timeout).await;
    }

    pub async fn list_rules(&self) -> Result<Vec<HttpUpstreamRuleConfig>, LdError> {
        self.inner.store().list().await
    }

    pub async fn save_rule(
        &self,
        rule: HttpUpstreamRuleConfig,
    ) -> Result<HttpUpstreamRuleConfig, LdError> {
        self.inner.store().set(rule).await
    }

    pub async fn find_rule(&self, id: ConfigId) -> Result<Option<HttpUpstreamRuleConfig>, LdError> {
        Repository::find_by_id(self.inner.store(), id).await
    }

    pub async fn delete_rule(&self, id: ConfigId) -> Result<(), LdError> {
        self.inner.store().delete(id).await
    }

    pub async fn reload_rules(&self) {
        self.inner.reload_rules().await;
    }

    pub async fn stored_rule_count(&self) -> usize {
        self.list_rules().await.map(|rules| rules.len()).unwrap_or_default()
    }
}

#[cfg(not(feature = "gateway"))]
#[derive(Debug, Clone)]
pub struct GatewayTlsConfig;

#[cfg(not(feature = "gateway"))]
impl GatewayTlsConfig {
    #[allow(dead_code)]
    pub fn new(_server_config: std::sync::Arc<rustls::ServerConfig>) -> Self {
        Self
    }
}

#[cfg(not(feature = "gateway"))]
#[derive(Clone)]
pub struct GatewayService {
    store: GatewayHttpUpstreamRepository,
    config: GatewayRuntimeConfig,
}

#[cfg(not(feature = "gateway"))]
impl GatewayService {
    pub async fn init_service(
        store: GatewayHttpUpstreamRepository,
        config: GatewayRuntimeConfig,
        _tls_config: Option<GatewayTlsConfig>,
    ) -> Self {
        Self { store, config }
    }

    pub fn is_supported(&self) -> bool {
        false
    }

    pub fn is_running(&self) -> bool {
        false
    }

    pub fn has_https_listener(&self) -> bool {
        false
    }

    pub fn config(&self) -> &GatewayRuntimeConfig {
        &self.config
    }

    pub async fn shutdown_and_wait(&self, _timeout: Duration) {}

    pub async fn list_rules(&self) -> Result<Vec<HttpUpstreamRuleConfig>, LdError> {
        Err(LdError::ConfigError(
            "gateway is not supported on this target architecture".to_string(),
        ))
    }

    pub async fn save_rule(
        &self,
        _rule: HttpUpstreamRuleConfig,
    ) -> Result<HttpUpstreamRuleConfig, LdError> {
        Err(LdError::ConfigError(
            "gateway is not supported on this target architecture".to_string(),
        ))
    }

    pub async fn find_rule(
        &self,
        _id: ConfigId,
    ) -> Result<Option<HttpUpstreamRuleConfig>, LdError> {
        Err(LdError::ConfigError(
            "gateway is not supported on this target architecture".to_string(),
        ))
    }

    pub async fn delete_rule(&self, _id: ConfigId) -> Result<(), LdError> {
        Err(LdError::ConfigError(
            "gateway is not supported on this target architecture".to_string(),
        ))
    }

    pub async fn reload_rules(&self) {}

    pub async fn stored_rule_count(&self) -> usize {
        self.store.list().await.map(|rules| rules.len()).unwrap_or_default()
    }
}
