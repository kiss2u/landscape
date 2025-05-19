use async_trait::async_trait;
use sea_orm::{
    ActiveModelBehavior, ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, FromQueryResult,
    IntoActiveModel, PrimaryKeyTrait,
};

/// 通用 Repository Trait
#[async_trait]
pub(crate) trait Repository
where
    Self: Sync + Send,
{
    type Model: Send + Into<Self::Data> + FromQueryResult + IntoActiveModel<Self::ActiveModel>;
    type Entity: EntityTrait<Model = Self::Model, ActiveModel = Self::ActiveModel>;
    type ActiveModel: ActiveModelTrait<Entity = Self::Entity> + Send + ActiveModelBehavior;
    type Data: Send + Sync + Into<Self::ActiveModel> + From<Self::Model>;
    type Id: Into<<<Self::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType>
        + Send
        + Sync;

    /// 提供数据库连接
    fn db(&self) -> &DatabaseConnection;

    /// 列出所有数据
    #[allow(dead_code)]
    async fn list_all(&self) -> Result<Vec<Self::Data>, DbErr> {
        let models: Vec<Self::Model> = <Self::Entity as EntityTrait>::find().all(self.db()).await?;
        Ok(models.into_iter().map(From::from).collect())
    }

    /// 插入数据
    #[allow(dead_code)]
    async fn set_model(&self, data: Self::Data) -> Result<Self::Data, DbErr> {
        let active_model: Self::ActiveModel = data.into();
        let inserted = active_model.insert(self.db()).await?;
        Ok(inserted.into())
    }

    /// 删除
    #[allow(dead_code)]
    async fn delete_model(&self, id: Self::Id) -> Result<(), DbErr> {
        <Self::Entity as EntityTrait>::delete_by_id(id).exec(self.db()).await?;
        Ok(())
    }

    /// 查找指定 ID
    #[allow(dead_code)]
    async fn find_by_id(&self, id: Self::Id) -> Result<Option<Self::Data>, DbErr> {
        let pk_value = id.into();
        let result = <Self::Entity as EntityTrait>::find_by_id(pk_value).one(self.db()).await?;
        Ok(result.map(From::from))
    }

    /// 清空
    #[allow(dead_code)]
    async fn truncate_table(&self) -> Result<(), DbErr> {
        <Self::Entity as EntityTrait>::delete_many().exec(self.db()).await?;
        Ok(())
    }
}
