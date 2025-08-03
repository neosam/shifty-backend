use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use dao::sales_person_unavailable::SalesPersonUnavailableEntity;
use dao::MockTransaction;
use mockall::automock;
use uuid::Uuid;

use crate::permission::Authentication;
use crate::ServiceError;
use shifty_utils::DayOfWeek;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SalesPersonUnavailable {
    pub id: Uuid,
    pub sales_person_id: Uuid,
    pub year: u32,
    pub calendar_week: u8,
    pub day_of_week: DayOfWeek,
    pub created: Option<time::PrimitiveDateTime>,
    pub deleted: Option<time::PrimitiveDateTime>,
    pub version: Uuid,
}
impl From<&SalesPersonUnavailableEntity> for SalesPersonUnavailable {
    fn from(entity: &SalesPersonUnavailableEntity) -> Self {
        Self {
            id: entity.id,
            sales_person_id: entity.sales_person_id,
            year: entity.year,
            calendar_week: entity.calendar_week,
            day_of_week: entity.day_of_week.into(),
            created: Some(entity.created),
            deleted: entity.deleted,
            version: entity.version,
        }
    }
}
impl TryFrom<&SalesPersonUnavailable> for SalesPersonUnavailableEntity {
    type Error = ServiceError;
    fn try_from(entity: &SalesPersonUnavailable) -> Result<Self, Self::Error> {
        Ok(Self {
            id: entity.id,
            sales_person_id: entity.sales_person_id,
            year: entity.year,
            calendar_week: entity.calendar_week,
            day_of_week: entity.day_of_week.into(),
            created: entity.created.ok_or_else(|| ServiceError::InternalError)?,
            deleted: entity.deleted,
            version: entity.version,
        })
    }
}

#[automock(type Context=(); type Transaction = MockTransaction;)]
#[async_trait]
pub trait SalesPersonUnavailableService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    async fn get_all_for_sales_person(
        &self,
        sales_person_id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[SalesPersonUnavailable]>, ServiceError>;
    async fn get_by_week_for_sales_person(
        &self,
        sales_person_id: Uuid,
        year: u32,
        calendar_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[SalesPersonUnavailable]>, ServiceError>;
    async fn get_by_week(
        &self,
        year: u32,
        calendar_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[SalesPersonUnavailable]>, ServiceError>;
    async fn create(
        &self,
        entity: &SalesPersonUnavailable,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<SalesPersonUnavailable, ServiceError>;
    async fn delete(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;
}
