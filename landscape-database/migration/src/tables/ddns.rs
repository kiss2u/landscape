use sea_orm_migration::prelude::Iden;
use sea_orm_migration::sea_query;

#[derive(Iden)]
pub enum DdnsJobs {
    Table,
    Id,
    Name,
    Enable,
    Source,
    ZoneName,
    ProviderProfileId,
    Ttl,
    Records,
    UpdateAt,
}

#[derive(Iden)]
pub enum DnsProviderProfiles {
    Table,
    Id,
    Name,
    ProviderConfig,
    Remark,
    DdnsDefaultTtl,
    UpdateAt,
}
