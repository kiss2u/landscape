pub use sea_orm_migration::prelude::*;

mod m20250511_170500_dns_config;
mod m20250517_083437_iface_config;
mod m20250518_081203_dhcp_v4_server;
mod tables;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250511_170500_dns_config::Migration),
            Box::new(m20250517_083437_iface_config::Migration),
            Box::new(m20250518_081203_dhcp_v4_server::Migration),
        ]
    }
}
