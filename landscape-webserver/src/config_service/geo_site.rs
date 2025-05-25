use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use landscape_common::config::{
    geo::{GeoDomainConfig, GeoSiteConfig},
    ConfigId,
};
use landscape_common::service::controller_service::ConfigController;

use crate::{
    error::{LandscapeApiError, LandscapeResult},
    LandscapeApp, SimpleResult,
};

pub async fn get_geo_site_config_paths() -> Router<LandscapeApp> {
    Router::new()
        .route("/geo_sites", get(get_geo_sites).post(add_geo_site))
        .route("/geo_sites/:id", get(get_geo_rule).delete(del_geo_site))
        .route("/geo_sites/cache", get(get_geo_site_cache).post(refresh_geo_site_cache))
}

async fn get_geo_site_cache(State(state): State<LandscapeApp>) -> Json<Vec<GeoDomainConfig>> {
    let result = state.geo_site_service.list_all_keys().await;
    tracing::debug!("keys len: {}", result.len());
    Json(result)
}

async fn refresh_geo_site_cache(State(state): State<LandscapeApp>) -> Json<SimpleResult> {
    state.geo_site_service.refresh().await;
    Json(SimpleResult { success: true })
}

async fn get_geo_sites(State(state): State<LandscapeApp>) -> Json<Vec<GeoSiteConfig>> {
    let result = state.geo_site_service.list().await;
    Json(result)
}

async fn get_geo_rule(
    State(state): State<LandscapeApp>,
    Path(id): Path<ConfigId>,
) -> LandscapeResult<Json<GeoSiteConfig>> {
    let result = state.geo_site_service.find_by_id(id).await;
    if let Some(config) = result {
        Ok(Json(config))
    } else {
        Err(LandscapeApiError::NotFound(format!("Dns Rule id: {:?}", id)))
    }
}

async fn add_geo_site(
    State(state): State<LandscapeApp>,
    Json(dns_rule): Json<GeoSiteConfig>,
) -> Json<GeoSiteConfig> {
    let result = state.geo_site_service.set(dns_rule).await;
    Json(result)
}

async fn del_geo_site(
    State(state): State<LandscapeApp>,
    Path(id): Path<ConfigId>,
) -> Json<SimpleResult> {
    state.geo_site_service.delete(id).await;
    Json(SimpleResult { success: true })
}
