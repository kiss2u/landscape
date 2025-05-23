use std::collections::HashMap;

use landscape_common::{config::dns::DomainConfig, event::dns::DnsEvent};
use landscape_database::provider::LandscapeDBServiceProvider;
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct GeoSiteService {}

impl GeoSiteService {
    pub async fn new(
        _store: LandscapeDBServiceProvider,
        _dns_events_tx: mpsc::Sender<DnsEvent>,
    ) -> Self {
        Self {}
    }

    pub fn get_geo_site_config(&self) -> HashMap<String, Vec<DomainConfig>> {
        HashMap::new()
    }
}
