use std::time::Duration;

use sea_orm::{Database, DatabaseConnection};

use migration::{Migrator, MigratorTrait};

use crate::repository::dns::DNSRepository;

pub struct LandscapeDBServiceProvider {
    database: DatabaseConnection,
}

impl LandscapeDBServiceProvider {
    pub async fn new(db_url: String) -> Self {
        let mut opt: migration::sea_orm::ConnectOptions = db_url.into();
        let (lever, _) = opt.get_sqlx_slow_statements_logging_settings();
        opt.sqlx_slow_statements_logging_settings(lever, Duration::from_secs(10));

        let database = Database::connect(opt).await.expect("Database connection failed");
        Migrator::up(&database, None).await.unwrap();
        Self { database }
    }

    pub fn dns_store(&self) -> DNSRepository {
        DNSRepository::new(self.database.clone())
    }
}

#[cfg(test)]
mod tests {
    use crate::provider::LandscapeDBServiceProvider;

    #[tokio::test]
    pub async fn test_run_database() {
        landscape_common::init_tracing!();

        let _provider =
            LandscapeDBServiceProvider::new("sqlite://../db.sqlite?mode=rwc".to_string()).await;
    }
}
