use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use bollard::Docker;

use landscape::docker::create_docker_event_spawn;

#[tokio::main]
async fn main() {
    let docker = Docker::connect_with_socket_defaults();
    let docker = docker.unwrap();
    println!();
    println!("{:?}", docker.ping().await);
    println!();
    println!("{:?}", docker.info().await);
    println!();
    create_docker_event_spawn().await;
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .unwrap();

    while running.load(Ordering::SeqCst) {
        tokio::time::sleep(Duration::new(1, 0)).await;
    }
}
