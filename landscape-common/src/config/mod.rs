pub mod api;
pub mod init;
pub mod loader;
pub mod runtime;
pub mod settings;

pub use crate::gateway::settings::LandscapeGatewayConfig;
pub use api::{
    GetDnsConfigResponse, GetGatewayConfigResponse, GetMetricConfigResponse, GetTimeConfigResponse,
    GetUIConfigResponse, UpdateDnsConfigRequest, UpdateGatewayConfigRequest,
    UpdateMetricConfigRequest, UpdateTimeConfigRequest, UpdateUIConfigRequest,
};
pub use init::InitConfig;
pub use runtime::{
    AuthRuntimeConfig, DnsRuntimeConfig, LogRuntimeConfig, MetricRuntimeConfig, RuntimeConfig,
    StoreRuntimeConfig, TimeRuntimeConfig, WebRuntimeConfig,
};
pub use settings::{
    LandscapeAuthConfig, LandscapeConfig, LandscapeDnsConfig, LandscapeLogConfig,
    LandscapeMetricConfig, LandscapeStoreConfig, LandscapeTimeConfig, LandscapeUIConfig,
    LandscapeWebConfig, MetricMode,
};

use uuid::Uuid;

pub type FlowId = u32;
pub type ConfigId = Uuid;
