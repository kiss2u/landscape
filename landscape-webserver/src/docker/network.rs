use axum::{routing::get, Router};

use landscape::docker::network::inspect_all_networks;
use landscape_common::docker::network::LandscapeDockerNetwork;

use crate::LandscapeApp;
use crate::{api::LandscapeApiResp, error::LandscapeApiResult};

pub async fn get_docker_networks_paths() -> Router<LandscapeApp> {
    Router::new().route("/", get(get_all_networks))
}

async fn get_all_networks() -> LandscapeApiResult<Vec<LandscapeDockerNetwork>> {
    let networks = inspect_all_networks().await;

    LandscapeApiResp::success(networks)
}
