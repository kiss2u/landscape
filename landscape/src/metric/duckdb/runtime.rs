use duckdb::DuckdbConnectionManager;
use landscape_common::concurrency::task_label;
use landscape_common::error::{LdError, LdResult};

use super::store::DuckMetricStore;

impl DuckMetricStore {
    pub(crate) fn get_disk_conn(&self) -> r2d2::PooledConnection<DuckdbConnectionManager> {
        self.disk_pool.get().expect("Failed to get disk connection from pool")
    }

    pub(crate) async fn run_query_default<T, F>(&self, op: &'static str, f: F) -> T
    where
        T: Default + Send + 'static,
        F: FnOnce(Self) -> T + Send + 'static,
    {
        let store = self.clone();
        let span = tracing::info_span!("task", task = task_label::task::METRIC_QUERY, op = op);
        tokio::task::spawn_blocking(move || span.in_scope(|| f(store))).await.unwrap_or_default()
    }

    pub(crate) async fn run_query_result<T, F>(&self, op: &'static str, f: F) -> LdResult<T>
    where
        T: Send + 'static,
        F: FnOnce(Self) -> LdResult<T> + Send + 'static,
    {
        let store = self.clone();
        let span = tracing::info_span!("task", task = task_label::task::METRIC_QUERY, op = op);
        tokio::task::spawn_blocking(move || span.in_scope(|| f(store)))
            .await
            .map_err(|error| LdError::DbMsg(format!("metric query task join failed: {error}")))?
    }
}
