use sea_orm_migration::{prelude::*, schema::*};

use crate::tables::dns_rule::DNSRuleConfigs;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(DNSRuleConfigs::Table)
                    .add_column_if_not_exists(json(DNSRuleConfigs::BindConfig).default("{}"))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(DNSRuleConfigs::Table)
                    .drop_column(DNSRuleConfigs::BindConfig)
                    .to_owned(),
            )
            .await
    }
}
