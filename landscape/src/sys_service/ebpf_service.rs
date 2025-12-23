use std::sync::Arc;

use tokio::sync::oneshot;

#[derive(Clone)]
#[allow(dead_code)]
pub struct LandscapeEbpfService {
    tx: Arc<oneshot::Sender<()>>,
}

impl LandscapeEbpfService {
    pub fn new() -> Self {
        let (tx, rx) = oneshot::channel::<()>();
        std::thread::spawn(move || {
            landscape_ebpf::base::ip_mac::neigh_update(rx).unwrap();
        });

        LandscapeEbpfService { tx: Arc::new(tx) }
    }
}
