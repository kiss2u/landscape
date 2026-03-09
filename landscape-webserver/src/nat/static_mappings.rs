use axum::extract::{Path, State};
use landscape_common::api_response::LandscapeApiResp as CommonApiResp;
use landscape_common::config::ConfigId;
use landscape_common::iface::nat::StaticNatMappingConfig;
use landscape_common::service::controller::ConfigController;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use landscape_common::iface::nat::StaticNatError;

use crate::api::JsonBody;
use crate::LandscapeApp;
use crate::{api::LandscapeApiResp, error::LandscapeApiResult};

pub fn get_static_nat_mapping_config_paths() -> OpenApiRouter<LandscapeApp> {
    OpenApiRouter::new()
        .routes(routes!(get_static_nat_mappings, add_static_nat_mappings))
        .routes(routes!(add_many_static_nat_mappings))
        .routes(routes!(get_static_nat_mapping, del_static_nat_mappings))
}

#[utoipa::path(
    get,
    path = "/static_mappings",
    tag = "Static NAT Mappings",
    responses((status = 200, body = CommonApiResp<Vec<StaticNatMappingConfig>>))
)]
async fn get_static_nat_mappings(
    State(state): State<LandscapeApp>,
) -> LandscapeApiResult<Vec<StaticNatMappingConfig>> {
    let result = state.static_nat_mapping_config_service.list().await;
    LandscapeApiResp::success(result)
}

#[utoipa::path(
    get,
    path = "/static_mappings/{id}",
    tag = "Static NAT Mappings",
    params(("id" = Uuid, Path, description = "Static NAT mapping ID")),
    responses(
        (status = 200, body = CommonApiResp<StaticNatMappingConfig>),
        (status = 404, description = "Not found")
    )
)]
async fn get_static_nat_mapping(
    State(state): State<LandscapeApp>,
    Path(id): Path<ConfigId>,
) -> LandscapeApiResult<StaticNatMappingConfig> {
    let result = state.static_nat_mapping_config_service.find_by_id(id).await;
    if let Some(config) = result {
        LandscapeApiResp::success(config)
    } else {
        Err(StaticNatError::NotFound(id))?
    }
}

#[utoipa::path(
    post,
    path = "/static_mappings/batch",
    tag = "Static NAT Mappings",
    request_body = Vec<StaticNatMappingConfig>,
    responses((status = 200, description = "Success"))
)]
async fn add_many_static_nat_mappings(
    State(state): State<LandscapeApp>,
    JsonBody(static_nat_mappings): JsonBody<Vec<StaticNatMappingConfig>>,
) -> LandscapeApiResult<()> {
    for m in &static_nat_mappings {
        m.validate()?;
    }
    state.static_nat_mapping_config_service.checked_set_list(static_nat_mappings).await?;
    LandscapeApiResp::success(())
}

#[utoipa::path(
    post,
    path = "/static_mappings",
    tag = "Static NAT Mappings",
    request_body = StaticNatMappingConfig,
    responses((status = 200, body = CommonApiResp<StaticNatMappingConfig>))
)]
async fn add_static_nat_mappings(
    State(state): State<LandscapeApp>,
    JsonBody(static_nat_mapping): JsonBody<StaticNatMappingConfig>,
) -> LandscapeApiResult<StaticNatMappingConfig> {
    static_nat_mapping.validate()?;
    let result = state.static_nat_mapping_config_service.checked_set(static_nat_mapping).await?;
    LandscapeApiResp::success(result)
}

#[utoipa::path(
    delete,
    path = "/static_mappings/{id}",
    tag = "Static NAT Mappings",
    params(("id" = Uuid, Path, description = "Static NAT mapping ID")),
    responses(
        (status = 200, description = "Success"),
        (status = 404, description = "Not found")
    )
)]
async fn del_static_nat_mappings(
    State(state): State<LandscapeApp>,
    Path(id): Path<ConfigId>,
) -> LandscapeApiResult<()> {
    state.static_nat_mapping_config_service.delete(id).await;
    LandscapeApiResp::success(())
}
