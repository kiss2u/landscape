use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum RouteWanServiceConfigs {
    Table,
    IfaceName,
    Enable,
    UpdateAt,
}

#[derive(Iden)]
pub enum RouteLanServiceConfigs {
    Table,
    IfaceName,
    Enable,
    UpdateAt,
}
