use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use uuid::Uuid;

use dao::DaoError;

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
    pub deleted: Option<time::PrimitiveDateTime>,
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
            deleted: extra_hours.deleted,
        }
    }
}
impl From<&ExtraHours> for dao::extra_hours::ExtraHoursEntity {
    fn from(extra_hours: &ExtraHours) -> Self {
        Self {
            id: extra_hours.id,
            sales_person_id: extra_hours.sales_person_id,
            amount: extra_hours.amount,
            category: (&extra_hours.category).into(),
            description: extra_hours.description.clone(),
            date_time: extra_hours.date_time,
            deleted: extra_hours.deleted,
        }
    }
}

#[automock]
#[async_trait]
pub trait ExtraHoursService {
    fn find_by_sales_person_id_and_year(
        &self,
        sales_person_id: Uuid,
        year: u32,
        until_week: u8,
    ) -> Result<Arc<[ExtraHours]>, DaoError>;
    fn create(&self, entity: &ExtraHours, process: &str) -> Result<(), DaoError>;
    fn update(&self, entity: &ExtraHours, process: &str) -> Result<(), DaoError>;
    fn delete(&self, id: Uuid, process: &str) -> Result<(), DaoError>;
}
