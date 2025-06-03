use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum WifiServiceConfigs {
    Table,
    IfaceName,
    Enable,
    Config,
    UpdateAt,
}
