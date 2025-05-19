use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum FlowWanServiceConfigs {
    Table,
    IfaceName,
    Enable,
    UpdateAt,
}
