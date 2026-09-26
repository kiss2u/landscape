use sea_orm_migration::{prelude::*, schema::string_null};

use crate::tables::{
    dns_rule::{DNSRedirectRuleConfigs, DNSUpstreamConfigs},
    dst_ip_rule::DstIpRuleConfigs,
    firewall_blacklist::FirewallBlacklistConfigs,
    firewall_rule::FirewallRuleConfigs,
    nat::{StaticNatMappingV4Configs, StaticNatMappingV6Configs},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        add_name(manager, DstIpRuleConfigs::Table, DstIpRuleConfigs::Name).await?;
        add_name(manager, DNSRedirectRuleConfigs::Table, DNSRedirectRuleConfigs::Name).await?;
        add_name(manager, DNSUpstreamConfigs::Table, DNSUpstreamConfigs::Name).await?;
        add_name(manager, FirewallRuleConfigs::Table, FirewallRuleConfigs::Name).await?;
        add_name(manager, FirewallBlacklistConfigs::Table, FirewallBlacklistConfigs::Name).await?;
        add_name(manager, StaticNatMappingV4Configs::Table, StaticNatMappingV4Configs::Name)
            .await?;
        add_name(manager, StaticNatMappingV6Configs::Table, StaticNatMappingV6Configs::Name).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        drop_name(manager, StaticNatMappingV6Configs::Table, StaticNatMappingV6Configs::Name)
            .await?;
        drop_name(manager, StaticNatMappingV4Configs::Table, StaticNatMappingV4Configs::Name)
            .await?;
        drop_name(manager, FirewallBlacklistConfigs::Table, FirewallBlacklistConfigs::Name).await?;
        drop_name(manager, FirewallRuleConfigs::Table, FirewallRuleConfigs::Name).await?;
        drop_name(manager, DNSUpstreamConfigs::Table, DNSUpstreamConfigs::Name).await?;
        drop_name(manager, DNSRedirectRuleConfigs::Table, DNSRedirectRuleConfigs::Name).await?;
        drop_name(manager, DstIpRuleConfigs::Table, DstIpRuleConfigs::Name).await
    }
}

async fn add_name<T, C>(manager: &SchemaManager<'_>, table: T, column: C) -> Result<(), DbErr>
where
    T: IntoIden + 'static,
    C: IntoIden + 'static,
{
    manager
        .alter_table(Table::alter().table(table).add_column(string_null(column)).to_owned())
        .await
}

async fn drop_name<T, C>(manager: &SchemaManager<'_>, table: T, column: C) -> Result<(), DbErr>
where
    T: IntoIden + 'static,
    C: IntoIden + 'static,
{
    manager.alter_table(Table::alter().table(table).drop_column(column).to_owned()).await
}
