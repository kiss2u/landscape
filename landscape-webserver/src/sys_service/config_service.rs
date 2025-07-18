use axum::{extract::State, routing::get, Router};

use crate::api::LandscapeApiResp;
use crate::error::LandscapeApiResult;
use crate::LandscapeApp;

pub async fn get_config_paths() -> Router<LandscapeApp> {
    Router::new().route("/config/export", get(export_init_config))
}

async fn export_init_config(State(state): State<LandscapeApp>) -> LandscapeApiResult<String> {
    let config = state.config_service.export_init_config().await;
    let config_file_content = toml::to_string(&config).unwrap();

    LandscapeApiResp::success(config_file_content)
}
