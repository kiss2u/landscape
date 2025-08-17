use sea_orm_migration::prelude::*;

use crate::tables::dst_ip_rule::DstIpRuleConfigs;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(DstIpRuleConfigs::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(DstIpRuleConfigs::Id).uuid().primary_key())
                    .col(ColumnDef::new(DstIpRuleConfigs::Index).unsigned().not_null())
                    .col(ColumnDef::new(DstIpRuleConfigs::Enable).boolean().not_null())
                    .col(ColumnDef::new(DstIpRuleConfigs::Mark).unsigned().not_null()) // FlowMark 映射为 u32
                    .col(ColumnDef::new(DstIpRuleConfigs::Source).json().not_null()) // Vec<WanIPRuleSource> 映射为 JSON
                    .col(ColumnDef::new(DstIpRuleConfigs::Remark).string().not_null())
                    .col(ColumnDef::new(DstIpRuleConfigs::FlowId).unsigned().not_null())
                    .col(ColumnDef::new(DstIpRuleConfigs::OverrideDns).boolean().not_null())
                    .col(
                        ColumnDef::new(DstIpRuleConfigs::UpdateAt).double().not_null().default(0.0),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(DstIpRuleConfigs::Table).to_owned()).await
    }
}
