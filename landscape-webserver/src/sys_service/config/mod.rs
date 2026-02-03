use axum::{
    extract::State,
    routing::{get, post},
    Router,
};

use crate::api::LandscapeApiResp;
use crate::error::LandscapeApiResult;
use crate::LandscapeApp;

pub mod metric;
pub mod ui;

pub async fn get_config_paths() -> Router<LandscapeApp> {
    Router::new()
        .route("/config/export", get(export_init_config))
        // UI Config
        .route("/config/ui", get(ui::get_ui_config_fast))
        .route("/config/edit/ui", get(ui::get_ui_config))
        .route("/config/edit/ui", post(ui::update_ui_config))
        // Metric Config
        .route("/config/metric", get(metric::get_metric_config_fast))
        .route("/config/edit/metric", get(metric::get_metric_config))
        .route("/config/edit/metric", post(metric::update_metric_config))
}

async fn export_init_config(State(state): State<LandscapeApp>) -> LandscapeApiResult<String> {
    let config = state.config_service.export_init_config().await;
    let config_file_content = toml::to_string(&config).unwrap();

    LandscapeApiResp::success(config_file_content)
}
