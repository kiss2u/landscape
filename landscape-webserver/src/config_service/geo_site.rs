use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use landscape_common::config::{
    geo::{
        GeoDomainConfig, GeoFileCacheKey, GeoSiteSourceConfig, QueryGeoDomainConfig, QueryGeoKey,
    },
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
        .route("/geo_sites/set_many", post(add_many_geo_sites))
        .route("/geo_sites/:id", get(get_geo_rule).delete(del_geo_site))
        .route("/geo_sites/cache", get(get_geo_site_cache).post(refresh_geo_site_cache))
        .route("/geo_sites/cache/search", get(search_geo_site_cache))
        .route("/geo_sites/cache/detail", get(get_geo_site_cache_detail))
}

async fn get_geo_site_cache_detail(
    State(state): State<LandscapeApp>,
    Query(key): Query<GeoFileCacheKey>,
) -> LandscapeResult<Json<GeoDomainConfig>> {
    let result = state.geo_site_service.get_cache_value_by_key(&key).await;
    if let Some(result) = result {
        Ok(Json(result))
    } else {
        Err(LandscapeApiError::NotFound(format!("{key:?}")))
    }
}

async fn search_geo_site_cache(
    State(state): State<LandscapeApp>,
    Query(query): Query<QueryGeoKey>,
) -> Json<Vec<GeoFileCacheKey>> {
    tracing::debug!("query: {:?}", query);
    let key = query.key.map(|k| k.to_ascii_uppercase());
    let name = query.name;
    tracing::debug!("name: {name:?}");
    tracing::debug!("key: {key:?}");
    let result: Vec<GeoFileCacheKey> = state
        .geo_site_service
        .list_all_keys()
        .await
        .into_iter()
        .filter(|e| key.as_ref().map_or(true, |key| e.key.contains(key)))
        .filter(|e| name.as_ref().map_or(true, |name| e.name.contains(name)))
        .collect();

    tracing::debug!("keys len: {}", result.len());
    Json(result)
}

async fn get_geo_site_cache(State(state): State<LandscapeApp>) -> Json<Vec<GeoFileCacheKey>> {
    let result = state.geo_site_service.list_all_keys().await;
    Json(result)
}

async fn refresh_geo_site_cache(State(state): State<LandscapeApp>) -> Json<SimpleResult> {
    state.geo_site_service.refresh(true).await;
    Json(SimpleResult { success: true })
}

async fn get_geo_sites(
    State(state): State<LandscapeApp>,
    Query(q): Query<QueryGeoDomainConfig>,
) -> Json<Vec<GeoSiteSourceConfig>> {
    let result = state.geo_site_service.query_geo_by_name(q.name).await;
    Json(result)
}

async fn get_geo_rule(
    State(state): State<LandscapeApp>,
    Path(id): Path<ConfigId>,
) -> LandscapeResult<Json<GeoSiteSourceConfig>> {
    let result = state.geo_site_service.find_by_id(id).await;
    if let Some(config) = result {
        Ok(Json(config))
    } else {
        Err(LandscapeApiError::NotFound(format!("Dns Rule id: {:?}", id)))
    }
}

async fn add_geo_site(
    State(state): State<LandscapeApp>,
    Json(dns_rule): Json<GeoSiteSourceConfig>,
) -> Json<GeoSiteSourceConfig> {
    let result = state.geo_site_service.set(dns_rule).await;
    Json(result)
}

async fn add_many_geo_sites(
    State(state): State<LandscapeApp>,
    Json(rules): Json<Vec<GeoSiteSourceConfig>>,
) -> Json<SimpleResult> {
    state.geo_site_service.set_list(rules).await;
    Json(SimpleResult { success: true })
}

async fn del_geo_site(
    State(state): State<LandscapeApp>,
    Path(id): Path<ConfigId>,
) -> Json<SimpleResult> {
    state.geo_site_service.delete(id).await;
    Json(SimpleResult { success: true })
}
