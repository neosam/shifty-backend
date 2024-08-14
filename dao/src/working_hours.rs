use crate::DaoError;
use async_trait::async_trait;
use mockall::automock;
use std::sync::Arc;
use time::PrimitiveDateTime;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq)]
pub struct WorkingHoursEntity {
    pub id: Uuid,
    pub sales_person_id: Uuid,
    pub expected_hours: f32,
    pub from_calendar_week: u8,
    pub from_year: u32,
    pub to_calendar_week: u8,
    pub to_year: u32,
    pub workdays_per_week: u8,
    pub days_per_week: u8,
    pub created: PrimitiveDateTime,
    pub deleted: Option<PrimitiveDateTime>,
    pub version: Uuid,
}

#[automock]
#[async_trait]
pub trait WorkingHoursDao {
    async fn all(&self) -> Result<Arc<[WorkingHoursEntity]>, DaoError>;
    async fn find_by_sales_person_id(
        &self,
        sales_person_id: Uuid,
    ) -> Result<Arc<[WorkingHoursEntity]>, DaoError>;
    async fn find_for_week(
        &self,
        calenar_week: u8,
        year: u32,
    ) -> Result<Arc<[WorkingHoursEntity]>, DaoError>;
    async fn create(&self, entity: &WorkingHoursEntity, process: &str) -> Result<(), DaoError>;
    async fn update(&self, entity: &WorkingHoursEntity, process: &str) -> Result<(), DaoError>;
}
