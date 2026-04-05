use sea_orm_migration::prelude::*;

use super::tables::ddns::DnsProviderProfiles;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(DnsProviderProfiles::Table)
                    .add_column(ColumnDef::new(DnsProviderProfiles::DdnsDefaultTtl).unsigned())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(DnsProviderProfiles::Table)
                    .drop_column(DnsProviderProfiles::DdnsDefaultTtl)
                    .to_owned(),
            )
            .await
    }
}
