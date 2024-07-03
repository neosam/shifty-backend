use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use time::PrimitiveDateTime;
use uuid::Uuid;

use crate::permission::Authentication;
use crate::ServiceError;

#[derive(Clone, Debug, PartialEq)]
pub struct WorkingHours {
    pub id: Uuid,
    pub sales_person_id: Uuid,
    pub expected_hours: f32,
    pub from_calendar_week: u8,
    pub from_year: u32,
    pub to_calendar_week: u8,
    pub to_year: u32,
    pub workdays_per_week: u8,
    pub days_per_week: u8,
    pub created: Option<PrimitiveDateTime>,
    pub deleted: Option<PrimitiveDateTime>,
    pub version: Uuid,
}
impl From<&dao::working_hours::WorkingHoursEntity> for WorkingHours {
    fn from(working_hours: &dao::working_hours::WorkingHoursEntity) -> Self {
        Self {
            id: working_hours.id,
            sales_person_id: working_hours.sales_person_id,
            expected_hours: working_hours.expected_hours,
            from_calendar_week: working_hours.from_calendar_week,
            from_year: working_hours.from_year,
            to_calendar_week: working_hours.to_calendar_week,
            to_year: working_hours.to_year,
            workdays_per_week: working_hours.workdays_per_week,
            days_per_week: working_hours.days_per_week,
            created: Some(working_hours.created),
            deleted: working_hours.deleted,
            version: working_hours.version,
        }
    }
}

impl WorkingHours {
    pub fn hours_per_day(&self) -> f32 {
        self.expected_hours / self.workdays_per_week as f32
    }

    pub fn holiday_hours(&self) -> f32 {
        self.expected_hours / self.days_per_week as f32
    }
}

impl TryFrom<&WorkingHours> for dao::working_hours::WorkingHoursEntity {
    type Error = ServiceError;
    fn try_from(working_hours: &WorkingHours) -> Result<Self, Self::Error> {
        Ok(Self {
            id: working_hours.id,
            sales_person_id: working_hours.sales_person_id,
            expected_hours: working_hours.expected_hours,
            from_calendar_week: working_hours.from_calendar_week,
            from_year: working_hours.from_year,
            to_calendar_week: working_hours.to_calendar_week,
            to_year: working_hours.to_year,
            workdays_per_week: working_hours.workdays_per_week,
            days_per_week: working_hours.days_per_week,
            created: working_hours
                .created
                .ok_or_else(|| ServiceError::InternalError)?,
            deleted: working_hours.deleted,
            version: working_hours.version,
        })
    }
}

#[automock(type Context=();)]
#[async_trait]
pub trait WorkingHoursService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;

    async fn all(
        &self,
        context: Authentication<Self::Context>,
    ) -> Result<Arc<[WorkingHours]>, ServiceError>;
    async fn find_by_sales_person_id(
        &self,
        sales_person_id: Uuid,
        context: Authentication<Self::Context>,
    ) -> Result<Arc<[WorkingHours]>, ServiceError>;
    async fn create(
        &self,
        entity: &WorkingHours,
        context: Authentication<Self::Context>,
    ) -> Result<WorkingHours, ServiceError>;
    async fn update(
        &self,
        entity: &WorkingHours,
        context: Authentication<Self::Context>,
    ) -> Result<WorkingHours, ServiceError>;
}
