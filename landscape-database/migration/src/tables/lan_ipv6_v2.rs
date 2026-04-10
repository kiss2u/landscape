use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
pub enum LanIPv6ServiceConfigsV2 {
    #[sea_orm(iden = "lan_ipv6_service_configs_v2")]
    Table,
    IfaceName,
    Enable,
    Config,
    UpdateAt,
}
