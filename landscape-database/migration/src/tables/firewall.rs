use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum FirewallServiceConfigs {
    Table,
    IfaceName,
    Enable,
    UpdateAt,
}
