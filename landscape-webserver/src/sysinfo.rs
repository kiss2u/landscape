use axum::{extract::State, routing::get, Router};

use landscape::{dev::LandscapeInterface, sys_service::routerstatus::get_sys_running_status};
use landscape_common::info::{
    LandscapeStatus, LandscapeSystemInfo, WatchResource, LAND_SYS_BASE_INFO,
};

use crate::{api::LandscapeApiResp, error::LandscapeApiResult};

type SysStatus = WatchResource<LandscapeStatus>;

async fn get_cpu_count(State(state): State<SysStatus>) -> LandscapeApiResult<usize> {
    let cpu_count = state.0.borrow().cpus.len();
    LandscapeApiResp::success(cpu_count)
}

/// return SYS base info
pub fn get_sys_info_route() -> Router {
    let watchs = get_sys_running_status();

    Router::new()
        .route("/sys", get(basic_sys_info))
        .route("/interval_fetch_info", get(interval_fetch_info))
        .route("/cpu_count", get(get_cpu_count))
        .with_state(watchs)
        .route("/net_dev", get(net_dev))
}

async fn net_dev() -> LandscapeApiResult<Vec<LandscapeInterface>> {
    let devs = landscape::get_all_devices().await;
    LandscapeApiResp::success(devs)
}

async fn basic_sys_info() -> LandscapeApiResult<LandscapeSystemInfo> {
    LandscapeApiResp::success(LAND_SYS_BASE_INFO.clone())
}

async fn interval_fetch_info(State(state): State<SysStatus>) -> LandscapeApiResult<SysStatus> {
    LandscapeApiResp::success(state)
}
