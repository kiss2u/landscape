use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use landscape::{dev::LandScapeInterface, CpuUsage, LandscapeStatic, LandscapeStatus, MemUsage};
use serde::{Deserialize, Serialize};

/// return SYS base info
pub fn get_sys_info_route() -> Router {
    let (static_info, watchs) = landscape::init_landscape();

    Router::new()
        .route("/sys", get(basic_sys_info))
        .with_state(static_info)
        .route("/cpu", get(cpu_info))
        .route("/mem", get(mem_info))
        .with_state(watchs)
        .route("/net_dev", get(net_dev))
}
async fn net_dev() -> Json<Vec<LandScapeInterface>> {
    let devs = landscape::get_all_devices().await;
    Json(devs)
}
async fn basic_sys_info(State(state): State<LandscapeStatic>) -> Json<LandscapeStatic> {
    Json(state)
}

async fn cpu_info(State(state): State<LandscapeStatus>) -> Json<Vec<CpuUsage>> {
    let recv = state.cpu_info_watch.subscribe();
    let data = recv.borrow().clone();
    Json(data)
}
async fn mem_info(State(state): State<LandscapeStatus>) -> Json<MemUsage> {
    let recv = state.mem_info_watch.subscribe();
    let data = recv.borrow().clone();
    Json(data)
}
