use sea_orm_migration::prelude::*;

use crate::tables::dns_rule::DNSUpstreamConfigs;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(DNSUpstreamConfigs::Table)
                    .add_column(
                        ColumnDef::new(DNSUpstreamConfigs::EnableIpValidation)
                            .boolean()
                            .null()
                            .default(Expr::value(false)),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(DNSUpstreamConfigs::Table)
                    .drop_column(DNSUpstreamConfigs::EnableIpValidation)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
