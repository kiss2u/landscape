use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum MssClampServiceConfigs {
    Table,
    IfaceName,
    Enable,
    ClampSize,
    UpdateAt,
}
