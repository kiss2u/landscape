use landscape::metric::MetricData;
use landscape_common::concurrency::{spawn_named_thread, thread_name};
use landscape_common::LANDSCAPE_METRIC_DIR_NAME;
use landscape_ebpf::metric::new_metric;
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::sync::oneshot;

// cargo run --package landscape-ebpf --bin event_metric_loop
#[tokio::main]
async fn main() {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .unwrap();

    let metric_path = PathBuf::from("/root/.landscape-router").join(LANDSCAPE_METRIC_DIR_NAME);

    let (tx, rx) = oneshot::channel::<()>();
    let (other_tx, other_rx) = oneshot::channel::<()>();

    let metric_service = MetricData::new(
        metric_path,
        landscape_common::config::MetricRuntimeConfig {
            enable: landscape_common::DEFAULT_METRIC_ENABLE,
            connect_second_window_minutes:
                landscape_common::DEFAULT_METRIC_CONNECT_SECOND_WINDOW_MINUTES,
            connect_1m_retention_days: landscape_common::DEFAULT_METRIC_CONNECT_1M_RETENTION_DAYS,
            connect_1h_retention_days: landscape_common::DEFAULT_METRIC_CONNECT_1H_RETENTION_DAYS,
            connect_1d_retention_days: landscape_common::DEFAULT_METRIC_CONNECT_1D_RETENTION_DAYS,
            dns_retention_days: landscape_common::DEFAULT_DNS_METRIC_RETENTION_DAYS,
            write_batch_size: landscape_common::DEFAULT_METRIC_WRITE_BATCH_SIZE,
            write_flush_interval_secs: landscape_common::DEFAULT_METRIC_WRITE_FLUSH_INTERVAL_SECS,
            db_max_memory_mb: 128,
            db_max_threads: 1,
            cleanup_interval_secs: landscape_common::DEFAULT_METRIC_CLEANUP_INTERVAL_SECS,
            cleanup_time_budget_ms: landscape_common::DEFAULT_METRIC_CLEANUP_TIME_BUDGET_MS,
            cleanup_slice_window_secs: landscape_common::DEFAULT_METRIC_CLEANUP_SLICE_WINDOW_SECS,
        },
    )
    .await;
    let metric_service_clone = metric_service.clone();
    spawn_named_thread(thread_name::fixed::METRIC_EVENT_READER, move || {
        new_metric(rx, metric_service_clone.connect_metric.get_msg_channel());
        let _ = other_tx.send(());
    })
    .expect("failed to spawn metric loop thread");

    while running.load(Ordering::SeqCst) {
        tokio::time::sleep(Duration::new(1, 0)).await;
        println!("data: {:?}", metric_service.connect_metric.connect_infos().await);
    }

    let _ = tx.send(());
    let _ = other_rx.await;
}
