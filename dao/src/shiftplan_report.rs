use async_trait::async_trait;
use mockall::automock;
use std::sync::Arc;
use uuid::Uuid;

use crate::slot::DayOfWeek;
use crate::DaoError;

#[derive(Clone, Debug, PartialEq)]
pub struct ShiftplanReportEntity {
    pub sales_person_id: Uuid,
    pub hours: f32,
    pub year: u32,
    pub calendar_week: u8,
    pub day_of_week: DayOfWeek,
}

pub struct ShiftplanQuickOverviewEntity {
    pub sales_person_id: Uuid,
    pub hours: f32,
    pub year: u32,
}

#[automock]
#[async_trait]
pub trait ShiftplanReportDao {
    /// A report which contains the worked hours of a sales person for each day.
    async fn extract_shiftplan_report(
        &self,
        sales_person_id: Uuid,
        year: u32,
        until_week: u8,
    ) -> Result<Arc<[ShiftplanReportEntity]>, DaoError>;

    /// A report which shows the summed up yearly work hours of all sales persons.
    async fn extract_quick_shiftplan_report(
        &self,
        year: u32,
        until_week: u8,
    ) -> Result<Arc<[ShiftplanQuickOverviewEntity]>, DaoError>;

    /// A report which contains the worked hours of all sales persons for a specific week.
    async fn extract_shiftplan_report_for_week(
        &self,
        year: u32,
        calendar_week: u8,
    ) -> Result<Arc<[ShiftplanReportEntity]>, DaoError>;
}
