use sea_orm_migration::prelude::*;

use crate::tables::firewall_rule::FirewallRuleConfigs;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(FirewallRuleConfigs::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(FirewallRuleConfigs::Id).uuid().primary_key())
                    .col(ColumnDef::new(FirewallRuleConfigs::Index).unsigned().not_null())
                    .col(ColumnDef::new(FirewallRuleConfigs::Enable).boolean().not_null())
                    .col(ColumnDef::new(FirewallRuleConfigs::Remark).string().not_null())
                    .col(ColumnDef::new(FirewallRuleConfigs::Items).json().not_null())
                    .col(ColumnDef::new(FirewallRuleConfigs::Mark).unsigned().not_null().default(0))
                    .col(
                        ColumnDef::new(FirewallRuleConfigs::UpdateAt)
                            .double()
                            .not_null()
                            .default(0.0),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(FirewallRuleConfigs::Table).to_owned()).await
    }
}
