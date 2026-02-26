use std::collections::HashMap;

use axum::extract::{Path, State};
use landscape_common::api_response::LandscapeApiResp as CommonApiResp;
use landscape_common::config::ra::IPV6RAServiceConfig;
use landscape_common::dhcp::v6_server::status::DHCPv6OfferInfo;
use landscape_common::lan_services::ipv6_ra::IPv6NAInfo;
use landscape_common::service::controller::ControllerService;
use landscape_common::service::{ServiceStatus, WatchService};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use landscape_common::service::ServiceConfigError;

use crate::api::JsonBody;
use crate::LandscapeApp;
use crate::{api::LandscapeApiResp, error::LandscapeApiResult};

pub fn get_iface_icmpv6ra_paths() -> OpenApiRouter<LandscapeApp> {
    OpenApiRouter::new()
        .routes(routes!(get_all_status))
        .routes(routes!(handle_iface_icmpv6))
        .routes(routes!(get_iface_icmpv6_config, delete_and_stop_iface_icmpv6))
        .routes(routes!(get_assigned_ips_by_iface_name))
        .routes(routes!(get_all_iface_assigned_ips))
        .routes(routes!(get_dhcpv6_assigned_by_iface_name))
        .routes(routes!(get_all_dhcpv6_assigned))
}

#[utoipa::path(
    get,
    path = "/icmpv6ra/assigned_ips",
    tag = "ICMPv6 RA",
    operation_id = "get_all_icmpv6ra_assigned_ips",
    responses((status = 200, body = CommonApiResp<HashMap<String, IPv6NAInfo>>))
)]
async fn get_all_iface_assigned_ips(
    State(state): State<LandscapeApp>,
) -> LandscapeApiResult<HashMap<String, IPv6NAInfo>> {
    LandscapeApiResp::success(state.ipv6_ra_service.get_assigned_ips().await)
}

#[utoipa::path(
    get,
    path = "/icmpv6ra/{iface_name}/assigned_ips",
    tag = "ICMPv6 RA",
    operation_id = "get_icmpv6ra_assigned_ips_by_iface_name",
    params(("iface_name" = String, Path, description = "Interface name")),
    responses((status = 200, body = CommonApiResp<Option<IPv6NAInfo>>))
)]
async fn get_assigned_ips_by_iface_name(
    State(state): State<LandscapeApp>,
    Path(iface_name): Path<String>,
) -> LandscapeApiResult<Option<IPv6NAInfo>> {
    LandscapeApiResp::success(
        state.ipv6_ra_service.get_assigned_ips_by_iface_name(iface_name).await,
    )
}

#[utoipa::path(
    get,
    path = "/icmpv6ra/status",
    tag = "ICMPv6 RA",
    operation_id = "get_all_icmpv6ra_status",
    responses((status = 200, body = CommonApiResp<HashMap<String, ServiceStatus>>))
)]
async fn get_all_status(
    State(state): State<LandscapeApp>,
) -> LandscapeApiResult<HashMap<String, WatchService>> {
    LandscapeApiResp::success(state.ipv6_ra_service.get_all_status().await)
}

#[utoipa::path(
    get,
    path = "/icmpv6ra/{iface_name}",
    tag = "ICMPv6 RA",
    params(("iface_name" = String, Path, description = "Interface name")),
    responses(
        (status = 200, body = CommonApiResp<IPV6RAServiceConfig>),
        (status = 404, description = "Not found")
    )
)]
async fn get_iface_icmpv6_config(
    State(state): State<LandscapeApp>,
    Path(iface_name): Path<String>,
) -> LandscapeApiResult<IPV6RAServiceConfig> {
    if let Some(iface_config) = state.ipv6_ra_service.get_config_by_name(iface_name).await {
        LandscapeApiResp::success(iface_config)
    } else {
        Err(ServiceConfigError::NotFound { service_name: "IPv6 RA" })?
    }
}

#[utoipa::path(
    put,
    path = "/icmpv6ra",
    tag = "ICMPv6 RA",
    request_body = IPV6RAServiceConfig,
    responses((status = 200, description = "Success"))
)]
async fn handle_iface_icmpv6(
    State(state): State<LandscapeApp>,
    JsonBody(config): JsonBody<IPV6RAServiceConfig>,
) -> LandscapeApiResult<()> {
    state.validate_zone(&config).await?;
    config.config.validate()?;
    state.ipv6_ra_service.handle_service_config(config).await?;
    LandscapeApiResp::success(())
}

#[utoipa::path(
    delete,
    path = "/icmpv6ra/{iface_name}",
    tag = "ICMPv6 RA",
    params(("iface_name" = String, Path, description = "Interface name")),
    responses((status = 200, body = CommonApiResp<Option<ServiceStatus>>))
)]
async fn delete_and_stop_iface_icmpv6(
    State(state): State<LandscapeApp>,
    Path(iface_name): Path<String>,
) -> LandscapeApiResult<Option<WatchService>> {
    LandscapeApiResp::success(state.ipv6_ra_service.delete_and_stop_iface_service(iface_name).await)
}

#[utoipa::path(
    get,
    path = "/icmpv6ra/{iface_name}/dhcpv6_assigned",
    tag = "ICMPv6 RA",
    operation_id = "get_dhcpv6_assigned_by_iface_name",
    params(("iface_name" = String, Path, description = "Interface name")),
    responses((status = 200, body = CommonApiResp<Option<DHCPv6OfferInfo>>))
)]
async fn get_dhcpv6_assigned_by_iface_name(
    State(state): State<LandscapeApp>,
    Path(iface_name): Path<String>,
) -> LandscapeApiResult<Option<DHCPv6OfferInfo>> {
    LandscapeApiResp::success(
        state.ipv6_ra_service.get_dhcpv6_assigned_by_iface_name(iface_name).await,
    )
}

#[utoipa::path(
    get,
    path = "/icmpv6ra/dhcpv6_assigned",
    tag = "ICMPv6 RA",
    operation_id = "get_all_dhcpv6_assigned",
    responses((status = 200, body = CommonApiResp<HashMap<String, DHCPv6OfferInfo>>))
)]
async fn get_all_dhcpv6_assigned(
    State(state): State<LandscapeApp>,
) -> LandscapeApiResult<HashMap<String, DHCPv6OfferInfo>> {
    LandscapeApiResp::success(state.ipv6_ra_service.get_dhcpv6_assigned().await)
}
