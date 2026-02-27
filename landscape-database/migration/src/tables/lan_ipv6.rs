use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
pub enum LanIPv6ServiceConfigs {
    #[sea_orm(iden = "lan_ipv6_service_configs")]
    Table,
    IfaceName,
    Enable,
    Config,
    UpdateAt,
}
