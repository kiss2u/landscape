use axum::{extract::State, routing::get, Json, Router};

use landscape::{dev::LandscapeInterface, get_sys_running_status, LandscapeStatus};
use landscape_common::info::{LandscapeSystemInfo, WatchResource, LAND_SYS_BASE_INFO};

type SysStatus = WatchResource<LandscapeStatus>;

/// return SYS base info
pub fn get_sys_info_route() -> Router {
    let watchs = get_sys_running_status();

    Router::new()
        .route("/sys", get(basic_sys_info))
        .route("/interval_fetch_info", get(interval_fetch_info))
        .with_state(watchs)
        .route("/net_dev", get(net_dev))
}

async fn net_dev() -> Json<Vec<LandscapeInterface>> {
    let devs = landscape::get_all_devices().await;
    Json(devs)
}

async fn basic_sys_info() -> Json<LandscapeSystemInfo> {
    Json(LAND_SYS_BASE_INFO.clone())
}

async fn interval_fetch_info(State(state): State<SysStatus>) -> Json<SysStatus> {
    Json(state)
}
