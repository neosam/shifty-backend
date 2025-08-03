use async_trait::async_trait;
use mockall::automock;
use shifty_utils::DayOfWeek;
use std::sync::Arc;
use time::error::ComponentRange;
use uuid::Uuid;

use crate::DaoError;

#[derive(Clone, Debug, PartialEq)]
pub struct ShiftplanReportEntity {
    pub sales_person_id: Uuid,
    pub hours: f32,
    pub year: u32,
    pub calendar_week: u8,
    pub day_of_week: DayOfWeek,
}

impl ShiftplanReportEntity {
    pub fn to_date(&self) -> Result<time::Date, ComponentRange> {
        let result = time::Date::from_iso_week_date(
            self.year as i32,
            self.calendar_week,
            time::Weekday::Monday.nth_next(self.day_of_week.to_number() - 1),
        );
        if result.is_err() {
            tracing::warn!(
                "Failed to convert ShiftplanReportEntity to time::Date: {:?}",
                self
            );
        }
        result
    }
}

pub struct ShiftplanQuickOverviewEntity {
    pub sales_person_id: Uuid,
    pub hours: f32,
    pub year: u32,
}

#[automock(type Transaction = crate::MockTransaction;)]
#[async_trait]
pub trait ShiftplanReportDao {
    type Transaction: crate::Transaction;

    /// A report which contains the worked hours of a sales person for each day.
    async fn extract_shiftplan_report(
        &self,
        sales_person_id: Uuid,
        from_year: u32,
        from_week: u8,
        to_year: u32,
        to_week: u8,
        tx: Self::Transaction,
    ) -> Result<Arc<[ShiftplanReportEntity]>, DaoError>;

    /// A report which shows the summed up yearly work hours of all sales persons.
    async fn extract_quick_shiftplan_report(
        &self,
        year: u32,
        until_week: u8,
        tx: Self::Transaction,
    ) -> Result<Arc<[ShiftplanQuickOverviewEntity]>, DaoError>;

    /// A report which contains the worked hours of all sales persons for a specific week.
    async fn extract_shiftplan_report_for_week(
        &self,
        year: u32,
        calendar_week: u8,
        tx: Self::Transaction,
    ) -> Result<Arc<[ShiftplanReportEntity]>, DaoError>;
}
