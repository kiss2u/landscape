use std::fmt::Debug;

#[async_trait::async_trait]
pub trait LandscapeDBTrait {
    type Data;
    type DBErr: Debug;
    type ID: Clone;
    async fn set(&self, config: Self::Data) -> Result<Self::Data, Self::DBErr>;

    async fn list(&self) -> Result<Vec<Self::Data>, Self::DBErr>;

    async fn delete(&self, id: Self::ID) -> Result<(), Self::DBErr>;
}

#[async_trait::async_trait]
pub trait LandscapeServiceDBTrait: LandscapeDBTrait {
    async fn find_by_iface_name(&self, id: Self::ID) -> Result<Option<Self::Data>, Self::DBErr>;
}
