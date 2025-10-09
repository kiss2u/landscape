use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
pub enum IPV6RAServiceConfigs {
    #[sea_orm(iden = "ipv6_ra_service_configs")]
    Table,
    IfaceName, // 主键
    Enable,
    SubnetPrefix,
    SubnetIndex,
    DependIface,
    RaPreferredLifetime,
    RaValidLifetime,
    RaFlag,
    UpdateAt,
}

#[derive(DeriveIden)]
pub enum IPV6RAServiceConfigs7_1_0 {
    #[sea_orm(iden = "ipv6_ra_service_configs")]
    Table,
    IfaceName, // 主键
    Enable,
    Config,
    UpdateAt,
}
