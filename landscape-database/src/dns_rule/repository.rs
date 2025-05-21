use landscape_common::{
    config::dns::DNSRuleConfig,
    database::{repository::Repository, LandscapeDBTrait, LandscapeFlowTrait},
};
use sea_orm::{DatabaseConnection, DbErr, EntityTrait};

use crate::{dns_rule::entity::DNSRuleConfigEntity, DBId};

use super::entity::{DNSRuleConfigActiveModel, DNSRuleConfigModel};

#[derive(Clone)]
pub struct DNSConfigRepository {
    db: DatabaseConnection,
}

impl DNSConfigRepository {
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

#[async_trait::async_trait]
impl LandscapeDBTrait for DNSConfigRepository {}

#[async_trait::async_trait]
impl LandscapeFlowTrait for DNSConfigRepository {}

#[async_trait::async_trait]
impl Repository for DNSConfigRepository {
    type Model = DNSRuleConfigModel;
    type Entity = DNSRuleConfigEntity;
    type ActiveModel = DNSRuleConfigActiveModel;
    type Data = DNSRuleConfig;
    type Id = DBId;

    fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}
