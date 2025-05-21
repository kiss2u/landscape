use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum FirewallRuleConfigs {
    Table,
    Id,
    Index,
    Enable,
    Remark,
    Items, // 存储 JSON 的字段
    Mark,
    UpdateAt,
}
