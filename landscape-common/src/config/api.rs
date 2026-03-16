use serde::{Deserialize, Serialize};

use crate::config::settings::{
    LandscapeDnsConfig, LandscapeMetricConfig, LandscapeTimeConfig, LandscapeUIConfig,
};
use crate::gateway::settings::LandscapeGatewayConfig;

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

#[derive(Serialize, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetTimeConfigResponse {
    pub time: LandscapeTimeConfig,
    pub hash: String,
}

#[derive(Deserialize, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateTimeConfigRequest {
    pub new_time: LandscapeTimeConfig,
    pub expected_hash: String,
}

#[derive(Serialize, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetGatewayConfigResponse {
    pub gateway: LandscapeGatewayConfig,
    pub hash: String,
}

#[derive(Deserialize, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateGatewayConfigRequest {
    pub new_gateway: LandscapeGatewayConfig,
    pub expected_hash: String,
}
