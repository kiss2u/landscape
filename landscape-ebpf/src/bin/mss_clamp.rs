use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use tokio::{sync::oneshot, time::sleep};

// cargo run --package landscape-ebpf --bin mss_clamp
#[tokio::main]
pub async fn main() {
    landscape_common::init_tracing!();
    landscape_ebpf::setting_libbpf_log();

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .unwrap();

    let ifindex = 2;
    let (tx, rx) = oneshot::channel::<()>();
    let (other_tx, other_rx) = oneshot::channel::<()>();

    std::thread::spawn(move || {
        println!("启动 run_mss_clamp 在 ifindex: {:?}", ifindex);
        landscape_ebpf::mss_clamp::run_mss_clamp(ifindex, 1492, true, rx);
        println!("向外部线程发送解除阻塞信号");
        let _ = other_tx.send(());
    });

    while running.load(Ordering::SeqCst) {
        sleep(Duration::new(1, 0)).await;
    }

    let _ = tx.send(());
    let _ = other_rx.await;
}
