use landscape_common::{
    config::dns::DNSRuleConfig,
    database::{
        repository::{
            Repository,
            // LandscapeDBStore,  UpdateActiveModel
        },
        LandscapeDBTrait, LandscapeFlowTrait,
    },
    // error::LdError,
};
// use migration::LockType;
use sea_orm::{
    DatabaseConnection,
    DbErr,
    EntityTrait,
    // ActiveModelTrait, ColumnTrait,  IntoActiveModel,
    // QueryFilter, QuerySelect, TransactionTrait,
};

use crate::{
    dns_rule::entity::{
        // DNSRuleConfigColumn,
        DNSRuleConfigActiveModel,
        DNSRuleConfigEntity,
        DNSRuleConfigModel,
    },
    // dst_ip_rule::entity::{DstIpRuleConfigColumn, DstIpRuleConfigEntity},
    DBId,
};

#[derive(Clone)]
pub struct DNSRuleRepository {
    db: DatabaseConnection,
}

impl DNSRuleRepository {
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
impl LandscapeDBTrait for DNSRuleRepository {
    // async fn set(&self, config: DNSRuleConfig) -> Result<DNSRuleConfig, LdError> {
    //     let txn = self.db.begin().await?;

    //     let locked_model = DNSRuleConfigEntity::find()
    //         .filter(DNSRuleConfigColumn::Id.eq(config.get_id()))
    //         .lock(LockType::Update)
    //         .one(&txn)
    //         .await?;

    //     let dst_result = DstIpRuleConfigEntity::find()
    //         .filter(DstIpRuleConfigColumn::Index.eq(config.index))
    //         .one(&txn)
    //         .await?;

    //     let dns_result = DNSRuleConfigEntity::find()
    //         .filter(
    //             DNSRuleConfigColumn::Index
    //                 .eq(config.index)
    //                 .and(DNSRuleConfigColumn::Id.eq(config.id).not()),
    //         )
    //         .one(&txn)
    //         .await?;

    //     if dst_result.is_some() || dns_result.is_some() {
    //         return Err(LdError::DbMsg("rule index duplicated".to_string()));
    //     }

    //     let result = if let Some(locked_model) = locked_model {
    //         if locked_model.update_at != config.update_at {
    //             return Err(LdError::DataIsExpired);
    //         }

    //         let mut active = locked_model.into_active_model();
    //         config.update(&mut active);
    //         active.update(&txn).await.map(|model| DNSRuleConfig::from(model))?
    //     } else {
    //         let active: DNSRuleConfigActiveModel = config.into();
    //         active.insert(&txn).await?.into()
    //     };
    //     txn.commit().await?;
    //     Ok(result)
    // }
}

#[async_trait::async_trait]
impl LandscapeFlowTrait for DNSRuleRepository {}

#[async_trait::async_trait]
impl Repository for DNSRuleRepository {
    type Model = DNSRuleConfigModel;
    type Entity = DNSRuleConfigEntity;
    type ActiveModel = DNSRuleConfigActiveModel;
    type Data = DNSRuleConfig;
    type Id = DBId;

    fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}
