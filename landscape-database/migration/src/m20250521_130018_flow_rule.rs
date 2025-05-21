use sea_orm_migration::prelude::*;

use crate::tables::flow_rule::FlowConfigs;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(FlowConfigs::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(FlowConfigs::Id).uuid().primary_key())
                    .col(ColumnDef::new(FlowConfigs::Enable).boolean().not_null())
                    .col(ColumnDef::new(FlowConfigs::FlowId).unsigned().not_null())
                    .col(ColumnDef::new(FlowConfigs::FlowMatchRules).json().not_null())
                    .col(ColumnDef::new(FlowConfigs::PacketHandleIfaceName).json().not_null())
                    .col(ColumnDef::new(FlowConfigs::Remark).string().not_null())
                    .col(ColumnDef::new(FlowConfigs::UpdateAt).double().not_null().default(0))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(FlowConfigs::Table).to_owned()).await
    }
}
