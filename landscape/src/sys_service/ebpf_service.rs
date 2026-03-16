use std::sync::Arc;

use landscape_common::concurrency::{spawn_named_thread, thread_name};
use tokio::sync::{oneshot, Mutex};

#[derive(Clone)]
#[allow(dead_code)]
pub struct LandscapeEbpfService {
    tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

impl LandscapeEbpfService {
    pub fn new() -> Self {
        let (tx, rx) = oneshot::channel::<()>();
        spawn_named_thread(thread_name::fixed::EBPF_NEIGH_UPDATE, move || {
            landscape_ebpf::base::ip_mac::neigh_update(rx).unwrap();
        })
        .expect("failed to spawn ebpf neigh_update thread");

        LandscapeEbpfService { tx: Arc::new(Mutex::new(Some(tx))) }
    }

    pub async fn stop(&self) {
        if let Some(tx) = self.tx.lock().await.take() {
            let _ = tx.send(());
            tracing::info!("eBPF neigh_update service stop signal sent");
        }
    }
}
