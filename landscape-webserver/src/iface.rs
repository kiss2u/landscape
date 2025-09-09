use axum::{
    extract::{Path, State},
    routing::{delete, get, post},
    Json, Router,
};
use landscape::iface::IfaceTopology;
use landscape_common::{
    config::iface::WifiMode,
    iface::{AddController, ChangeZone},
};
use landscape_common::{
    config::iface::{IfaceCpuSoftBalance, NetworkIfaceConfig},
    iface::BridgeCreate,
};

use crate::{api::LandscapeApiResp, error::LandscapeApiResult, LandscapeApp};

pub async fn get_network_paths() -> Router<LandscapeApp> {
    Router::new()
        .route("/", get(get_ifaces))
        .route("/wan_configs", get(get_wan_ifaces))
        .route("/manage/{iface_name}", post(manage_ifaces))
        .route("/bridge", post(create_bridge))
        .route("/bridge/{bridge_name}", delete(delete_bridge))
        .route("/controller", post(set_controller))
        .route("/zone", post(change_zone))
        .route("/{iface_name}/status/{status}", post(change_dev_status))
        .route("/{iface_name}/wifi_mode/{mode}", post(change_wifi_mode))
        .route("/{iface_name}/cpu_balance", get(get_cpu_balance).post(set_cpu_balance))
}

async fn get_wan_ifaces(
    State(state): State<LandscapeApp>,
) -> LandscapeApiResult<Vec<NetworkIfaceConfig>> {
    let result = state.iface_config_service.get_all_wan_iface_config().await;
    LandscapeApiResp::success(result)
}

async fn manage_ifaces(
    State(state): State<LandscapeApp>,
    Path(iface_name): Path<String>,
) -> LandscapeApiResult<()> {
    state.iface_config_service.manage_dev(iface_name).await;
    LandscapeApiResp::success(())
}

async fn get_ifaces(State(state): State<LandscapeApp>) -> LandscapeApiResult<Vec<IfaceTopology>> {
    let result = state.iface_config_service.old_read_ifaces().await;
    LandscapeApiResp::success(result)
}

async fn create_bridge(
    State(state): State<LandscapeApp>,
    Json(bridge_create_request): Json<BridgeCreate>,
) -> LandscapeApiResult<()> {
    state.iface_config_service.create_bridge(bridge_create_request).await;
    LandscapeApiResp::success(())
}

async fn delete_bridge(
    State(state): State<LandscapeApp>,
    Path(bridge_name): Path<String>,
) -> LandscapeApiResult<()> {
    state.remove_all_iface_service(&bridge_name).await;
    state.iface_config_service.delete_bridge(bridge_name).await;
    LandscapeApiResp::success(())
}

async fn set_controller(
    State(state): State<LandscapeApp>,
    Json(controller): Json<AddController>,
) -> LandscapeApiResult<()> {
    state.iface_config_service.set_controller(controller).await;
    LandscapeApiResp::success(())
}

// 切换 网卡 所属区域
async fn change_zone(
    State(state): State<LandscapeApp>,
    Json(change_zone): Json<ChangeZone>,
) -> LandscapeApiResult<()> {
    state.remove_all_iface_service(&change_zone.iface_name).await;
    state.iface_config_service.change_zone(change_zone).await;
    LandscapeApiResp::success(())
}

async fn change_wifi_mode(
    State(state): State<LandscapeApp>,
    Path((iface_name, mode)): Path<(String, WifiMode)>,
) -> LandscapeApiResult<()> {
    state.iface_config_service.change_wifi_mode(iface_name, mode).await;
    LandscapeApiResp::success(())
}

async fn change_dev_status(
    State(state): State<LandscapeApp>,
    Path((iface_name, enable_in_boot)): Path<(String, bool)>,
) -> LandscapeApiResult<()> {
    state.iface_config_service.change_dev_status(iface_name, enable_in_boot).await;
    LandscapeApiResp::success(())
}

async fn get_cpu_balance(
    State(state): State<LandscapeApp>,
    Path(iface_name): Path<String>,
) -> LandscapeApiResult<Option<IfaceCpuSoftBalance>> {
    let iface = state.iface_config_service.get_iface_config(iface_name).await;
    LandscapeApiResp::success(iface.and_then(|iface| iface.xps_rps))
}

async fn set_cpu_balance(
    State(state): State<LandscapeApp>,
    Path(iface_name): Path<String>,
    Json(balance): Json<Option<IfaceCpuSoftBalance>>,
) -> LandscapeApiResult<()> {
    state.iface_config_service.change_cpu_balance(iface_name, balance).await;
    LandscapeApiResp::success(())
}
