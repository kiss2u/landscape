use sea_orm_migration::{prelude::*, schema::*};

use crate::tables::iface_ip::IfaceIpServiceConfigs;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(IfaceIpServiceConfigs::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(IfaceIpServiceConfigs::IfaceName).string().primary_key())
                    .col(ColumnDef::new(IfaceIpServiceConfigs::Enable).boolean())
                    .col(ColumnDef::new(IfaceIpServiceConfigs::IpModel).json().not_null())
                    .col(double(IfaceIpServiceConfigs::UpdateAt).default(0))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(IfaceIpServiceConfigs::Table).to_owned()).await
    }
}
