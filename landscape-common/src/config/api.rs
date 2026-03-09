use serde::{Deserialize, Serialize};

use crate::config::settings::{LandscapeDnsConfig, LandscapeMetricConfig, LandscapeUIConfig};

#[derive(Serialize, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetUIConfigResponse {
    pub ui: LandscapeUIConfig,
    pub hash: String,
}

#[derive(Deserialize, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateUIConfigRequest {
    pub new_ui: LandscapeUIConfig,
    pub expected_hash: String,
}

#[derive(Serialize, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetMetricConfigResponse {
    pub metric: LandscapeMetricConfig,
    pub hash: String,
}

#[derive(Deserialize, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateMetricConfigRequest {
    pub new_metric: LandscapeMetricConfig,
    pub expected_hash: String,
}

#[derive(Serialize, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetDnsConfigResponse {
    pub dns: LandscapeDnsConfig,
    pub hash: String,
}

#[derive(Deserialize, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateDnsConfigRequest {
    pub new_dns: LandscapeDnsConfig,
    pub expected_hash: String,
}
