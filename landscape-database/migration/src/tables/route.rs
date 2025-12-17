use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum RouteWanServiceConfigs {
    Table,
    IfaceName,
    Enable,
    UpdateAt,
}

#[derive(DeriveIden)]
pub enum RouteLanServiceConfigs {
    #[sea_orm(iden = "route_lan_service_configs")]
    Table,
    IfaceName,
    Enable,
    UpdateAt,
}

#[derive(DeriveIden)]
#[allow(unused)]
pub enum RouteLanServiceConfigsV2 {
    #[sea_orm(iden = "route_lan_service_configs")]
    Table,
    IfaceName,
    Enable,
    UpdateAt,
    StaticRoutes,
}
