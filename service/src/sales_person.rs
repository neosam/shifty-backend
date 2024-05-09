use std::sync::Arc;
use std::fmt::Debug;

use async_trait::async_trait;
use mockall::automock;
use uuid::Uuid;

use crate::ServiceError;
use crate::permission::Authentication;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SalesPerson {
    pub id: Uuid,
    pub name: Arc<str>,
    pub inactive: bool,
    pub deleted: Option<time::PrimitiveDateTime>,
    pub version: Uuid,
}
impl From<&dao::sales_person::SalesPersonEntity> for SalesPerson {
    fn from(sales_person: &dao::sales_person::SalesPersonEntity) -> Self {
        Self {
            id: sales_person.id,
            name: sales_person.name.clone(),
            inactive: sales_person.inactive,
            deleted: sales_person.deleted,
            version: sales_person.version,
        }
    }
}
impl From<&SalesPerson> for dao::sales_person::SalesPersonEntity {
    fn from(sales_person: &SalesPerson) -> Self {
        Self {
            id: sales_person.id,
            name: sales_person.name.clone(),
            inactive: sales_person.inactive,
            deleted: sales_person.deleted,
            version: sales_person.version,
        }
    }
}

#[automock(type Context=();)]
#[async_trait]
pub trait SalesPersonService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;

    async fn get_all(&self, context: Authentication<Self::Context>) -> Result<Arc<[SalesPerson]>, ServiceError>;
    async fn get(&self, id: Uuid, context: Authentication<Self::Context>) -> Result<SalesPerson, ServiceError>;
    async fn exists(&self, id: Uuid, context: Authentication<Self::Context>) -> Result<bool, ServiceError>;
    async fn create(
        &self,
        item: &SalesPerson,
        context: Authentication<Self::Context>,
    ) -> Result<SalesPerson, ServiceError>;
    async fn update(
        &self,
        item: &SalesPerson,
        context: Authentication<Self::Context>,
    ) -> Result<SalesPerson, ServiceError>;
    async fn delete(&self, id: Uuid, context: Authentication<Self::Context>) -> Result<(), ServiceError>;
    async fn get_assigned_user(
        &self,
        sales_person_id: Uuid,
        context: Authentication<Self::Context>,
    ) -> Result<Option<Arc<str>>, ServiceError>;
    async fn set_user(
        &self,
        sales_person_id: Uuid,
        user_id: Option<Arc<str>>,
        context: Authentication<Self::Context>,
    ) -> Result<(), ServiceError>;
}
