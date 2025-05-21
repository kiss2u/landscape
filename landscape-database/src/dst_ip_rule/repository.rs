use landscape_common::database::{repository::Repository, LandscapeDBTrait, LandscapeFlowTrait};
use landscape_common::ip_mark::WanIpRuleConfig;
use sea_orm::DatabaseConnection;

use crate::{dst_ip_rule::entity::DstIpRuleConfigEntity, DBId};

use super::entity::{DstIpRuleConfigActiveModel, DstIpRuleConfigModel};

#[derive(Clone)]
pub struct DstIpRuleRepository {
    db: DatabaseConnection,
}

impl DstIpRuleRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl LandscapeDBTrait for DstIpRuleRepository {}

#[async_trait::async_trait]
impl LandscapeFlowTrait for DstIpRuleRepository {}

#[async_trait::async_trait]
impl Repository for DstIpRuleRepository {
    type Model = DstIpRuleConfigModel;
    type Entity = DstIpRuleConfigEntity;
    type ActiveModel = DstIpRuleConfigActiveModel;
    type Data = WanIpRuleConfig;
    type Id = DBId;

    fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}
