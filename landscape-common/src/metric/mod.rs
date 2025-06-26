use std::path::PathBuf;

use crate::metric::connect::ConnectMetricManager;

pub mod connect;
#[cfg(feature = "duckdb")]
pub mod duckdb;
#[cfg(feature = "polars")]
pub mod polars;

#[derive(Clone)]
pub struct MetricData {
    pub connect_metric: ConnectMetricManager,
}

impl MetricData {
    pub async fn new(home_path: PathBuf) -> Self {
        MetricData {
            connect_metric: ConnectMetricManager::new(home_path).await,
        }
    }
}
