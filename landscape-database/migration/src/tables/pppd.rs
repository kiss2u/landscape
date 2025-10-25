use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
pub enum PPPDServiceConfigs {
    #[sea_orm(iden = "pppd_service_configs")]
    Table,
    IfaceName, // 主键
    AttachIfaceName,
    Enable,
    DefaultRoute,
    PeerId,
    Password,
    UpdateAt,

    /// Since 0.8.1
    Ac,
}
