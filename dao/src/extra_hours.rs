use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use shifty_utils::ShiftyDate;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExtraHoursCategoryEntity {
    ExtraWork,
    Vacation,
    SickLeave,
    Holiday,
    Unavailable,
    Custom(Uuid),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExtraHoursEntity {
    pub id: Uuid,
    pub sales_person_id: Uuid,
    pub amount: f32,
    pub category: ExtraHoursCategoryEntity,
    pub description: Arc<str>,
    pub date_time: time::PrimitiveDateTime,
    pub created: time::PrimitiveDateTime,
    pub deleted: Option<time::PrimitiveDateTime>,
    pub version: Uuid,
}

impl ExtraHoursEntity {
    pub fn as_date(&self) -> ShiftyDate {
        ShiftyDate::from_date(self.date_time.date())
    }
}

#[automock(type Transaction = crate::MockTransaction;)]
#[async_trait]
pub trait ExtraHoursDao {
    type Transaction: crate::Transaction;

    async fn find_by_id(
        &self,
        id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<ExtraHoursEntity>, crate::DaoError>;
    async fn find_by_sales_person_id_and_years(
        &self,
        sales_person_id: Uuid,
        from_year: u32,
        to_year: u32,
        tx: Self::Transaction,
    ) -> Result<Arc<[ExtraHoursEntity]>, crate::DaoError>;
    async fn find_by_week(
        &self,
        calendar_week: u8,
        year: u32,
        tx: Self::Transaction,
    ) -> Result<Arc<[ExtraHoursEntity]>, crate::DaoError>;
    async fn create(
        &self,
        entity: &ExtraHoursEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), crate::DaoError>;
    async fn update(
        &self,
        entity: &ExtraHoursEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), crate::DaoError>;
    async fn delete(
        &self,
        id: Uuid,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), crate::DaoError>;
}
