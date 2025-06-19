use std::path::PathBuf;

use landscape_common::{
    metric::MetricData,
    service::{DefaultWatchServiceStatus, ServiceStatus},
    LANDSCAPE_METRIC_DIR_NAME,
};
use tokio::sync::oneshot;

#[derive(Clone)]
pub struct MetricService {
    pub status: DefaultWatchServiceStatus,
    pub data: MetricData,
}

impl MetricService {
    pub async fn new(home_path: PathBuf) -> Self {
        let metric_path = home_path.join(LANDSCAPE_METRIC_DIR_NAME);
        let status = DefaultWatchServiceStatus::new();
        MetricService { data: MetricData::new(metric_path).await, status }
    }

    pub async fn start_service(&self) {
        let status = self.status.clone();
        if status.is_stop() {
            let metric_service = self.data.clone();
            tokio::spawn(async move {
                create_metric_service(metric_service, status).await;
            });
        } else {
            tracing::info!("Metric Service is not stopped");
        }
    }

    pub async fn stop_service(&self) {
        self.status.wait_stop().await;
    }
}

pub async fn create_metric_service(
    metric_service: MetricData,
    service_status: DefaultWatchServiceStatus,
) {
    service_status.just_change_status(ServiceStatus::Staring);
    let (tx, rx) = oneshot::channel::<()>();
    let (other_tx, other_rx) = oneshot::channel::<()>();
    service_status.just_change_status(ServiceStatus::Running);
    let service_status_clone = service_status.clone();
    tokio::spawn(async move {
        let stop_wait = service_status_clone.wait_to_stopping();
        tracing::info!("等待外部停止信号");
        let _ = stop_wait.await;
        tracing::info!("接收外部停止信号");
        let _ = tx.send(());
        tracing::info!("向内部发送停止信号");
    });
    std::thread::spawn(move || {
        landscape_ebpf::metric::new_metric(rx, metric_service);
        let _ = other_tx.send(());
    });
    let _ = other_rx.await;
    tracing::info!("结束外部线程阻塞");
    service_status.just_change_status(ServiceStatus::Stop);
}
