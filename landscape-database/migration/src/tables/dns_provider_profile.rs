use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
pub enum DnsProviderProfiles {
    #[sea_orm(iden = "dns_provider_profiles")]
    Table,
    Id,
    Name,
    ProviderConfig,
    Remark,
    DdnsDefaultTtl,
    UpdateAt,
}
