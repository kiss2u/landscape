use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
pub enum IfaceIpServiceConfigs {
    #[sea_orm(iden = "iface_ip_service_configs")]
    Table,
    IfaceName,
    Enable,
    IpModel,
    UpdateAt,
}
