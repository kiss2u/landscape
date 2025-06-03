use landscape_common::{
    database::{repository::Repository, LandscapeDBTrait},
    firewall::FirewallRuleConfig,
};
use sea_orm::DatabaseConnection;

use crate::{firewall_rule::entity::FirewallRuleConfigEntity, DBId};

use super::entity::{FirewallRuleConfigActiveModel, FirewallRuleConfigModel};

#[derive(Clone)]
pub struct FirewallRuleRepository {
    db: DatabaseConnection,
}

impl FirewallRuleRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl LandscapeDBTrait for FirewallRuleRepository {}

#[async_trait::async_trait]
impl Repository for FirewallRuleRepository {
    type Model = FirewallRuleConfigModel;
    type Entity = FirewallRuleConfigEntity;
    type ActiveModel = FirewallRuleConfigActiveModel;
    type Data = FirewallRuleConfig;
    type Id = DBId;

    fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}
