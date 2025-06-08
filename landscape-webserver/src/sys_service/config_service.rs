use axum::response::IntoResponse;
use axum::{
    extract::State,
    http::{header, HeaderMap, HeaderValue},
    response::Response,
    routing::get,
    Router,
};
use chrono::Local;

use crate::LandscapeApp;

pub async fn get_config_paths() -> Router<LandscapeApp> {
    Router::new().route("/config/export", get(export_init_config))
}

async fn export_init_config(State(state): State<LandscapeApp>) -> Response {
    let config = state.config_service.export_init_config().await;
    let config_file_content = toml::to_string(&config).unwrap();

    // 获取当前时间
    let now = Local::now();
    let time_str = now.format("%Y-%m-%d_%H-%M-%S").to_string();

    // 构造 HTTP 头部
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("application/octet-stream"));
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!(
            "attachment; filename=\"landscape_init_{}.toml\"",
            time_str
        ))
        .unwrap(),
    );

    // 返回 Response
    (headers, config_file_content).into_response()
}
