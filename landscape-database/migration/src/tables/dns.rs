use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum DNSRuleConfigs {
    Table,
    Id,
    Index,
    Name,
    Enable,
    Filter,
    ResolveMode,
    Mark,
    Source,
    FlowId,
}
