use std::path::PathBuf;

use landscape_common::{
    concurrency::{spawn_named_thread, spawn_task, task_label, thread_name},
    event::DnsMetricMessage,
    service::{ServiceStatus, WatchService},
    LANDSCAPE_METRIC_DIR_NAME,
};
use tokio::sync::mpsc;
use tokio::sync::oneshot;

#[cfg(feature = "metric-duckdb")]
pub mod duckdb;
#[cfg(not(feature = "metric-duckdb"))]
pub mod memory_store;

#[cfg(feature = "metric-duckdb")]
pub type MetricStore = duckdb::DuckMetricStore;
#[cfg(not(feature = "metric-duckdb"))]
pub type MetricStore = memory_store::MemoryMetricStore;

use landscape_common::config::MetricRuntimeConfig;

#[derive(Clone)]
pub struct MetricService {
    pub enabled: bool,
    pub status: WatchService,
    pub store: MetricStore,
}

impl MetricService {
    pub async fn new(home_path: PathBuf, config: MetricRuntimeConfig) -> Self {
        let metric_path = home_path.join(LANDSCAPE_METRIC_DIR_NAME);
        if !metric_path.exists() {
            if let Err(e) = std::fs::create_dir_all(&metric_path) {
                tracing::error!("Failed to create metric directory: {}", e);
            }
        }
        let status = WatchService::new();
        MetricService {
            enabled: config.enable,
            store: MetricStore::new(metric_path, config).await,
            status,
        }
    }

    pub async fn start_service(&self) {
        if !self.enabled {
            tracing::info!("Metric service disabled by config");
            return;
        }

        let status = self.status.clone();
        if status.is_stop() {
            let metric_store = self.store.clone();
            spawn_task(task_label::task::METRIC_SERVICE_RUN, async move {
                create_metric_service(metric_store, status).await;
            });
        } else {
            tracing::info!("Metric Service is not stopped");
        }
    }

    pub async fn stop_service(&self) {
        self.status.wait_stop().await;
    }

    pub fn get_dns_metric_channel(&self) -> Option<mpsc::Sender<DnsMetricMessage>> {
        self.enabled.then(|| self.store.get_dns_msg_channel())
    }
}

pub async fn create_metric_service(metric_store: MetricStore, service_status: WatchService) {
    service_status.just_change_status(ServiceStatus::Staring);
    let (tx, rx) = oneshot::channel::<()>();
    let (other_tx, other_rx) = oneshot::channel::<()>();
    service_status.just_change_status(ServiceStatus::Running);
    let service_status_clone = service_status.clone();
    spawn_task(task_label::task::METRIC_SERVICE_STOP, async move {
        let stop_wait = service_status_clone.wait_to_stopping();
        tracing::info!("等待外部停止信号");
        let _ = stop_wait.await;
        tracing::info!("接收外部停止信号");
        let _ = tx.send(());
        tracing::info!("向内部发送停止信号");
    });

    let connect_msg_tx = metric_store.get_connect_msg_channel();
    spawn_named_thread(thread_name::fixed::METRIC_EVENT_READER, move || {
        landscape_ebpf::metric::new_metric(rx, connect_msg_tx);
        let _ = other_tx.send(());
    })
    .expect("failed to spawn metric event thread");
    let _ = other_rx.await;
    tracing::info!("结束外部线程阻塞");
    service_status.just_change_status(ServiceStatus::Stop);
}
