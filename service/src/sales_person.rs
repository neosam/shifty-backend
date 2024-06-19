use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use uuid::Uuid;

use crate::permission::Authentication;
use crate::ServiceError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SalesPerson {
    pub id: Uuid,
    pub name: Arc<str>,
    pub background_color: Arc<str>,
    pub is_paid: Option<bool>,
    pub inactive: bool,
    pub deleted: Option<time::PrimitiveDateTime>,
    pub version: Uuid,
}
impl From<&dao::sales_person::SalesPersonEntity> for SalesPerson {
    fn from(sales_person: &dao::sales_person::SalesPersonEntity) -> Self {
        Self {
            id: sales_person.id,
            name: sales_person.name.clone(),
            background_color: sales_person.background_color.clone(),
            is_paid: Some(sales_person.is_paid),
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
            background_color: sales_person.background_color.clone(),
            is_paid: sales_person.is_paid.unwrap_or(false),
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

    async fn get_all(
        &self,
        context: Authentication<Self::Context>,
    ) -> Result<Arc<[SalesPerson]>, ServiceError>;
    async fn get_all_paid(
        &self,
        context: Authentication<Self::Context>,
    ) -> Result<Arc<[SalesPerson]>, ServiceError>;
    async fn get(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
    ) -> Result<SalesPerson, ServiceError>;
    async fn exists(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
    ) -> Result<bool, ServiceError>;
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
    async fn delete(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
    ) -> Result<(), ServiceError>;
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
    async fn get_sales_person_for_user(
        &self,
        user_id: Arc<str>,
        context: Authentication<Self::Context>,
    ) -> Result<Option<SalesPerson>, ServiceError>;
    async fn get_sales_person_current_user(
        &self,
        context: Authentication<Self::Context>,
    ) -> Result<Option<SalesPerson>, ServiceError>;
}
