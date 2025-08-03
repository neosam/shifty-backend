use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use shifty_utils::DayOfWeek;
use uuid::Uuid;

use crate::DaoError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SalesPersonUnavailableEntity {
    pub id: Uuid,
    pub sales_person_id: Uuid,
    pub year: u32,
    pub calendar_week: u8,
    pub day_of_week: DayOfWeek,
    pub created: time::PrimitiveDateTime,
    pub deleted: Option<time::PrimitiveDateTime>,
    pub version: Uuid,
}

#[automock(type Transaction = crate::MockTransaction;)]
#[async_trait]
pub trait SalesPersonUnavailableDao {
    type Transaction: crate::Transaction;

    async fn find_by_id(
        &self,
        id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<SalesPersonUnavailableEntity>, DaoError>;
    async fn find_all_by_sales_person_id(
        &self,
        sales_person_id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Arc<[SalesPersonUnavailableEntity]>, DaoError>;
    async fn find_by_week_and_sales_person_id(
        &self,
        sales_person_id: Uuid,
        year: u32,
        calendar_week: u8,
        tx: Self::Transaction,
    ) -> Result<Arc<[SalesPersonUnavailableEntity]>, DaoError>;
    async fn find_by_week(
        &self,
        year: u32,
        calendar_week: u8,
        tx: Self::Transaction,
    ) -> Result<Arc<[SalesPersonUnavailableEntity]>, DaoError>;
    async fn create(
        &self,
        entity: &SalesPersonUnavailableEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;
    async fn update(
        &self,
        entity: &SalesPersonUnavailableEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;
}
