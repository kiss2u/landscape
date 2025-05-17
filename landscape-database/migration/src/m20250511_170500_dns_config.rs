use sea_orm_migration::prelude::*;

use crate::tables::dns::DNSRuleConfigs;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(DNSRuleConfigs::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(DNSRuleConfigs::Id).uuid().primary_key())
                    .col(ColumnDef::new(DNSRuleConfigs::Index).unsigned().not_null())
                    .col(ColumnDef::new(DNSRuleConfigs::Name).string())
                    .col(ColumnDef::new(DNSRuleConfigs::Enable).boolean())
                    .col(ColumnDef::new(DNSRuleConfigs::Filter).json().not_null())
                    .col(ColumnDef::new(DNSRuleConfigs::ResolveMode).string())
                    .col(ColumnDef::new(DNSRuleConfigs::Mark).unsigned())
                    .col(ColumnDef::new(DNSRuleConfigs::Source).text().not_null())
                    .col(ColumnDef::new(DNSRuleConfigs::FlowId).unsigned())
                    .col(ColumnDef::new(DNSRuleConfigs::UpdateAt).double().default(0).not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(DNSRuleConfigs::Table).to_owned()).await
    }
}
