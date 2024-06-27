use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use uuid::Uuid;

use crate::{permission::Authentication, ServiceError};

#[derive(Clone, Debug, PartialEq)]
pub enum ExtraHoursCategory {
    ExtraWork,
    Vacation,
    SickLeave,
    Holiday,
}
impl From<&dao::extra_hours::ExtraHoursCategoryEntity> for ExtraHoursCategory {
    fn from(category: &dao::extra_hours::ExtraHoursCategoryEntity) -> Self {
        match category {
            dao::extra_hours::ExtraHoursCategoryEntity::ExtraWork => Self::ExtraWork,
            dao::extra_hours::ExtraHoursCategoryEntity::Vacation => Self::Vacation,
            dao::extra_hours::ExtraHoursCategoryEntity::SickLeave => Self::SickLeave,
            dao::extra_hours::ExtraHoursCategoryEntity::Holiday => Self::Holiday,
        }
    }
}
impl From<&ExtraHoursCategory> for dao::extra_hours::ExtraHoursCategoryEntity {
    fn from(category: &ExtraHoursCategory) -> Self {
        match category {
            ExtraHoursCategory::ExtraWork => Self::ExtraWork,
            ExtraHoursCategory::Vacation => Self::Vacation,
            ExtraHoursCategory::SickLeave => Self::SickLeave,
            ExtraHoursCategory::Holiday => Self::Holiday,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExtraHours {
    pub id: Uuid,
    pub sales_person_id: Uuid,
    pub amount: f32,
    pub category: ExtraHoursCategory,
    pub description: Arc<str>,
    pub date_time: time::PrimitiveDateTime,
    pub created: Option<time::PrimitiveDateTime>,
    pub deleted: Option<time::PrimitiveDateTime>,
    pub version: Uuid,
}
impl From<&dao::extra_hours::ExtraHoursEntity> for ExtraHours {
    fn from(extra_hours: &dao::extra_hours::ExtraHoursEntity) -> Self {
        Self {
            id: extra_hours.id,
            sales_person_id: extra_hours.sales_person_id,
            amount: extra_hours.amount,
            category: (&extra_hours.category).into(),
            description: extra_hours.description.clone(),
            date_time: extra_hours.date_time,
            created: Some(extra_hours.created),
            deleted: extra_hours.deleted,
            version: extra_hours.version,
        }
    }
}
impl TryFrom<&ExtraHours> for dao::extra_hours::ExtraHoursEntity {
    type Error = ServiceError;
    fn try_from(extra_hours: &ExtraHours) -> Result<Self, Self::Error> {
        Ok(Self {
            id: extra_hours.id,
            sales_person_id: extra_hours.sales_person_id,
            amount: extra_hours.amount,
            category: (&extra_hours.category).into(),
            description: extra_hours.description.clone(),
            date_time: extra_hours.date_time,
            created: extra_hours
                .created
                .ok_or_else(|| ServiceError::InternalError)?,
            deleted: extra_hours.deleted,
            version: extra_hours.version,
        })
    }
}

#[automock(type Context=();)]
#[async_trait]
pub trait ExtraHoursService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;

    async fn find_by_sales_person_id_and_year(
        &self,
        sales_person_id: Uuid,
        year: u32,
        until_week: u8,
        context: Authentication<Self::Context>,
    ) -> Result<Arc<[ExtraHours]>, ServiceError>;
    async fn create(
        &self,
        entity: &ExtraHours,
        context: Authentication<Self::Context>,
    ) -> Result<ExtraHours, ServiceError>;
    async fn update(
        &self,
        entity: &ExtraHours,
        context: Authentication<Self::Context>,
    ) -> Result<ExtraHours, ServiceError>;
    async fn delete(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
    ) -> Result<(), ServiceError>;
}
