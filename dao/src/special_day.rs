use crate::DaoError;
use async_trait::async_trait;
use mockall::automock;
use std::sync::Arc;
use uuid::Uuid;

use crate::slot::DayOfWeek;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpecialDayTypeEntity {
    Holiday,
    ShortDay,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpecialDayEntity {
    pub id: Uuid,
    pub year: u32,
    pub calendar_week: u8,
    pub day_of_week: DayOfWeek,
    pub day_type: SpecialDayTypeEntity,
    pub time_of_day: Option<time::Time>,
    pub created: time::PrimitiveDateTime,
    pub deleted: Option<time::PrimitiveDateTime>,
    pub version: Uuid,
}

#[automock]
#[async_trait]
pub trait SpecialDayDao {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<SpecialDayEntity>, DaoError>;
    async fn find_by_week(
        &self,
        year: u32,
        calendar_week: u8,
    ) -> Result<Arc<[SpecialDayEntity]>, DaoError>;
    async fn create(&self, entity: &SpecialDayEntity, process: &str) -> Result<(), DaoError>;
    async fn update(&self, entity: &SpecialDayEntity, process: &str) -> Result<(), DaoError>;
}
