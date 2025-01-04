use crate::{slot::DayOfWeek, DaoError};
use async_trait::async_trait;
use mockall::automock;
use std::sync::Arc;
use time::PrimitiveDateTime;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq)]
pub struct EmployeeWorkDetailsEntity {
    pub id: Uuid,
    pub sales_person_id: Uuid,
    pub expected_hours: f32,
    pub from_day_of_week: DayOfWeek,
    pub from_calendar_week: u8,
    pub from_year: u32,
    pub to_day_of_week: DayOfWeek,
    pub to_calendar_week: u8,
    pub to_year: u32,
    pub workdays_per_week: u8,

    pub monday: bool,
    pub tuesday: bool,
    pub wednesday: bool,
    pub thursday: bool,
    pub friday: bool,
    pub saturday: bool,
    pub sunday: bool,

    pub vacation_days: u8,

    pub created: PrimitiveDateTime,
    pub deleted: Option<PrimitiveDateTime>,
    pub version: Uuid,
}

#[automock(type Transaction = crate::MockTransaction;)]
#[async_trait]
pub trait EmployeeWorkDetailsDao {
    type Transaction: crate::Transaction;

    async fn all(
        &self,
        tx: Self::Transaction,
    ) -> Result<Arc<[EmployeeWorkDetailsEntity]>, DaoError>;
    async fn find_by_id(
        &self,
        id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<EmployeeWorkDetailsEntity>, DaoError>;
    async fn find_by_sales_person_id(
        &self,
        sales_person_id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Arc<[EmployeeWorkDetailsEntity]>, DaoError>;
    async fn find_for_week(
        &self,
        calenar_week: u8,
        year: u32,
        tx: Self::Transaction,
    ) -> Result<Arc<[EmployeeWorkDetailsEntity]>, DaoError>;
    async fn create(
        &self,
        entity: &EmployeeWorkDetailsEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;
    async fn update(
        &self,
        entity: &EmployeeWorkDetailsEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;
}
