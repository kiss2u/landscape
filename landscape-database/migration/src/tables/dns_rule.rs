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
    UpstreamId,
    Mark,
    Source,
    BindConfig,
    FlowId,
    UpdateAt,
}

#[derive(Iden)]
pub enum DNSRedirectRuleConfigs {
    Table,
    Id,
    Remark,
    Enable,
    MatchRules,
    ResultInfo,
    ApplyFlows,
    UpdateAt,
}

#[derive(Iden)]
pub enum DNSUpstreamConfigs {
    Table,
    Id,
    Remark,
    Mode,
    Ips,
    Port,
    /// Append at 0.8.0
    EnableIpValidation,
    UpdateAt,
}
