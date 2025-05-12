use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use tokio::{sync::oneshot, time::sleep};

// cargo run --package landscape-ebpf --bin flow_test
#[tokio::main]
pub async fn main() {
    landscape_common::init_tracing!();
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .unwrap();

    let ifindex = 9;
    let (tx, rx) = oneshot::channel::<()>();
    let (other_tx, other_rx) = oneshot::channel::<()>();

    std::thread::spawn(move || {
        println!("启动 attach_match_flow 在 ifindex: {:?}", ifindex);
        landscape_ebpf::flow::mark::attach_match_flow(ifindex, true, rx).unwrap();
        println!("向外部线程发送解除阻塞信号");
        let _ = other_tx.send(());
    });

    while running.load(Ordering::SeqCst) {
        sleep(Duration::new(1, 0)).await;
    }

    let _ = tx.send(());
    let _ = other_rx.await;
}
