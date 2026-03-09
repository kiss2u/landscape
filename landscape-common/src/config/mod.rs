pub mod api;
pub mod init;
pub mod loader;
pub mod runtime;
pub mod settings;

pub use api::{
    GetDnsConfigResponse, GetMetricConfigResponse, GetUIConfigResponse, UpdateDnsConfigRequest,
    UpdateMetricConfigRequest, UpdateUIConfigRequest,
};
pub use init::InitConfig;
pub use runtime::{
    AuthRuntimeConfig, DnsRuntimeConfig, LogRuntimeConfig, MetricRuntimeConfig, RuntimeConfig,
    StoreRuntimeConfig, WebRuntimeConfig,
};
pub use settings::{
    LandscapeAuthConfig, LandscapeConfig, LandscapeDnsConfig, LandscapeLogConfig,
    LandscapeMetricConfig, LandscapeStoreConfig, LandscapeUIConfig, LandscapeWebConfig,
};

use uuid::Uuid;

pub type FlowId = u32;
pub type ConfigId = Uuid;
