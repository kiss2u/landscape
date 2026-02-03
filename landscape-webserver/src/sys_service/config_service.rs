use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use landscape_common::config::{GetUIConfigResponse, LandscapeUIConfig, UpdateUIConfigRequest};

use crate::api::LandscapeApiResp;
use crate::error::LandscapeApiResult;
use crate::LandscapeApp;

pub async fn get_config_paths() -> Router<LandscapeApp> {
    Router::new()
        .route("/config/export", get(export_init_config))
        .route("/config/ui", get(get_ui_config_fast))
        .route("/config/edit/ui", get(get_ui_config))
        .route("/config/edit/ui", post(update_ui_config))
}

async fn export_init_config(State(state): State<LandscapeApp>) -> LandscapeApiResult<String> {
    let config = state.config_service.export_init_config().await;
    let config_file_content = toml::to_string(&config).unwrap();

    LandscapeApiResp::success(config_file_content)
}

async fn get_ui_config(
    State(state): State<LandscapeApp>,
) -> LandscapeApiResult<GetUIConfigResponse> {
    let (config, hash) = state.config_service.get_config_with_hash().await?;
    LandscapeApiResp::success(GetUIConfigResponse { ui: config.ui, hash })
}

async fn get_ui_config_fast(
    State(state): State<LandscapeApp>,
) -> LandscapeApiResult<LandscapeUIConfig> {
    let ui_config = state.config_service.get_ui_config_from_memory();
    LandscapeApiResp::success(ui_config)
}

async fn update_ui_config(
    State(state): State<LandscapeApp>,
    Json(payload): Json<UpdateUIConfigRequest>,
) -> LandscapeApiResult<()> {
    state.config_service.update_ui_config(payload.new_ui, payload.expected_hash).await?;
    LandscapeApiResp::success(())
}
