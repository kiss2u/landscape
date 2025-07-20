use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use bollard::Docker;

use landscape::docker::create_docker_event_spawn;
use landscape::route::IpRouteService;
use landscape_database::provider::LandscapeDBServiceProvider;
use tokio::sync::mpsc;

/// cargo run --package landscape --bin docker_test
#[tokio::main]
async fn main() {
    landscape_common::init_tracing!();
    let docker = Docker::connect_with_socket_defaults();
    let docker = docker.unwrap();
    println!();
    println!("{:?}", docker.ping().await);
    println!();
    println!("{:?}", docker.info().await);
    println!();

    let db_store_provider = LandscapeDBServiceProvider::mem_test_db().await;
    let flow_repo = db_store_provider.flow_rule_store();
    let (_, route_rx) = mpsc::channel(1);
    let route_service = IpRouteService::new(route_rx, flow_repo);

    create_docker_event_spawn(route_service).await;
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
