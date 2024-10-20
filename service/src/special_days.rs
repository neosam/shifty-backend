use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use dao::special_day::{SpecialDayEntity, SpecialDayTypeEntity};
use mockall::automock;
use uuid::Uuid;

use crate::{permission::Authentication, slot::DayOfWeek, ServiceError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpecialDayType {
    Holiday,
    ShortDay,
}
impl From<&SpecialDayTypeEntity> for SpecialDayType {
    fn from(entity: &SpecialDayTypeEntity) -> Self {
        match entity {
            SpecialDayTypeEntity::Holiday => Self::Holiday,
            SpecialDayTypeEntity::ShortDay => Self::ShortDay,
        }
    }
}
impl From<&SpecialDayType> for SpecialDayTypeEntity {
    fn from(special_day_type: &SpecialDayType) -> Self {
        match special_day_type {
            SpecialDayType::Holiday => Self::Holiday,
            SpecialDayType::ShortDay => Self::ShortDay,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpecialDay {
    pub id: Uuid,
    pub year: u32,
    pub calendar_week: u8,
    pub day_of_week: DayOfWeek,
    pub day_type: SpecialDayType,
    pub time_of_day: Option<time::Time>,
    pub created: Option<time::PrimitiveDateTime>,
    pub deleted: Option<time::PrimitiveDateTime>,
    pub version: Uuid,
}
impl From<&SpecialDayEntity> for SpecialDay {
    fn from(entity: &SpecialDayEntity) -> Self {
        Self {
            id: entity.id,
            year: entity.year,
            calendar_week: entity.calendar_week,
            day_of_week: entity.day_of_week.into(),
            day_type: (&entity.day_type).into(),
            time_of_day: entity.time_of_day,
            created: Some(entity.created),
            deleted: entity.deleted,
            version: entity.version,
        }
    }
}
impl TryFrom<&SpecialDay> for SpecialDayEntity {
    type Error = ServiceError;

    fn try_from(special_day: &SpecialDay) -> Result<Self, Self::Error> {
        Ok(Self {
            id: special_day.id,
            year: special_day.year,
            calendar_week: special_day.calendar_week,
            day_of_week: special_day.day_of_week.into(),
            day_type: (&special_day.day_type).into(),
            time_of_day: special_day.time_of_day,
            created: special_day
                .created
                .ok_or_else(|| ServiceError::InternalError)?,
            deleted: special_day.deleted,
            version: special_day.version,
        })
    }
}

#[automock(type Context=();)]
#[async_trait]
pub trait SpecialDayService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    async fn get_by_week(
        &self,
        year: u32,
        calendar_week: u8,
        context: Authentication<Self::Context>,
    ) -> Result<Arc<[SpecialDay]>, ServiceError>;
    async fn create(
        &self,
        special_day: &SpecialDay,
        context: Authentication<Self::Context>,
    ) -> Result<SpecialDay, ServiceError>;
    async fn delete(
        &self,
        special_day_id: Uuid,
        context: Authentication<Self::Context>,
    ) -> Result<SpecialDay, ServiceError>;
}
