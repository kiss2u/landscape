use crate::LandscapeApp;
use crate::{api::LandscapeApiResp, error::LandscapeApiResult};
use axum::{routing::post, Router};

pub async fn get_route_paths() -> Router<LandscapeApp> {
    Router::new().route("/route/reset_cache", post(reset_cache))
}

async fn reset_cache() -> LandscapeApiResult<()> {
    landscape_ebpf::map_setting::route::cache::recreate_route_lan_cache_inner_map();
    // landscape_ebpf::map_setting::route::cache::recreate_route_wan_cache_inner_map();
    LandscapeApiResp::success(())
}
