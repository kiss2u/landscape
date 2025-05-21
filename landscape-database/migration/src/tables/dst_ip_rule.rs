use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum DstIpRuleConfigs {
    Table,
    Id,
    Index,
    Enable,
    Mark,
    Source,
    Remark,
    FlowId,
    OverrideDns,
    UpdateAt,
}
