use landscape_common::dns::DNSRuleConfig;
use sea_orm::{DatabaseConnection, DbErr, EntityTrait};

use crate::{entity::dns::DNSRuleConfigEntity, DBId};

pub struct DNSRepository {
    db: DatabaseConnection,
}

impl DNSRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn find_by_id(&self, id: DBId) -> Result<Option<DNSRuleConfig>, DbErr> {
        Ok(DNSRuleConfigEntity::find_by_id(id)
            .one(&self.db)
            .await?
            .map(|model| DNSRuleConfig::from(model)))
    }
}
