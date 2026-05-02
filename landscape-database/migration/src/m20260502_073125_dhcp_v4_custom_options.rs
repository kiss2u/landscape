use sea_orm_migration::prelude::*;

use crate::tables::dhcp_v4_server::DHCPv4ServerConfigs;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(DHCPv4ServerConfigs::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(DHCPv4ServerConfigs::CustomOptions)
                            .json()
                            .not_null()
                            .default("[]"),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(DHCPv4ServerConfigs::Table)
                    .drop_column(DHCPv4ServerConfigs::CustomOptions)
                    .to_owned(),
            )
            .await
    }
}
