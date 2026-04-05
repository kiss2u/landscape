use sea_orm_migration::prelude::*;

use super::tables::ddns::{DdnsJobs, DnsProviderProfiles};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(DnsProviderProfiles::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(DnsProviderProfiles::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(DnsProviderProfiles::Name).string().not_null())
                    .col(ColumnDef::new(DnsProviderProfiles::ProviderConfig).json().not_null())
                    .col(ColumnDef::new(DnsProviderProfiles::Remark).text())
                    .col(
                        ColumnDef::new(DnsProviderProfiles::UpdateAt)
                            .double()
                            .not_null()
                            .default(0.0),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-dns-provider-profiles-name")
                    .table(DnsProviderProfiles::Table)
                    .col(DnsProviderProfiles::Name)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(DdnsJobs::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(DdnsJobs::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(DdnsJobs::Name).string().not_null())
                    .col(ColumnDef::new(DdnsJobs::Enable).boolean().not_null().default(true))
                    .col(ColumnDef::new(DdnsJobs::Source).json().not_null())
                    .col(ColumnDef::new(DdnsJobs::ZoneName).string().not_null())
                    .col(ColumnDef::new(DdnsJobs::ProviderProfileId).uuid().not_null())
                    .col(ColumnDef::new(DdnsJobs::Ttl).unsigned())
                    .col(ColumnDef::new(DdnsJobs::Records).json().not_null())
                    .col(ColumnDef::new(DdnsJobs::UpdateAt).double().not_null().default(0.0))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-ddns-job-provider-profile")
                            .from(DdnsJobs::Table, DdnsJobs::ProviderProfileId)
                            .to(DnsProviderProfiles::Table, DnsProviderProfiles::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-ddns-jobs-provider-profile")
                    .table(DdnsJobs::Table)
                    .col(DdnsJobs::ProviderProfileId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(DdnsJobs::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(DnsProviderProfiles::Table).to_owned()).await
    }
}
