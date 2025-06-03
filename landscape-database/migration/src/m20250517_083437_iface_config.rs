use sea_orm_migration::prelude::*;

use crate::tables::iface::NetIfaceConfigs;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(NetIfaceConfigs::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(NetIfaceConfigs::Name).string().primary_key())
                    .col(ColumnDef::new(NetIfaceConfigs::UpdateAt).double().default(0).not_null())
                    .col(
                        ColumnDef::new(NetIfaceConfigs::CreateDevType)
                            .string()
                            .not_null()
                            .default("no_need_to_create"),
                    )
                    .col(ColumnDef::new(NetIfaceConfigs::ControllerName).string().null())
                    .col(
                        ColumnDef::new(NetIfaceConfigs::ZoneType)
                            .string()
                            .not_null()
                            .default("undefined"),
                    )
                    .col(
                        ColumnDef::new(NetIfaceConfigs::EnableInBoot)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(NetIfaceConfigs::WifiMode)
                            .string()
                            .not_null()
                            .default("undefined"),
                    )
                    .col(ColumnDef::new(NetIfaceConfigs::XpsRps).json().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(NetIfaceConfigs::Table).to_owned()).await
    }
}
