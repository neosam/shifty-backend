use async_trait::async_trait;
use dao::MockTransaction;
use mockall::automock;
use std::fmt::Debug;
use std::sync::Arc;
use uuid::Uuid;

use crate::{permission::Authentication, slot::DayOfWeek, ServiceError};

#[derive(Clone, Debug, PartialEq)]
pub struct ShiftplanReportDay {
    pub sales_person_id: Uuid,
    pub hours: f32,
    pub year: u32,
    pub calendar_week: u8,
    pub day_of_week: DayOfWeek,
}

impl ShiftplanReportDay {
    pub fn to_date(&self) -> Result<time::Date, time::error::ComponentRange> {
        let weekday = match self.day_of_week {
            DayOfWeek::Monday => time::Weekday::Monday,
            DayOfWeek::Tuesday => time::Weekday::Tuesday,
            DayOfWeek::Wednesday => time::Weekday::Wednesday,
            DayOfWeek::Thursday => time::Weekday::Thursday,
            DayOfWeek::Friday => time::Weekday::Friday,
            DayOfWeek::Saturday => time::Weekday::Saturday,
            DayOfWeek::Sunday => time::Weekday::Sunday,
        };
        time::Date::from_iso_week_date(self.year as i32, self.calendar_week, weekday)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ShiftplanQuickOverview {
    pub sales_person_id: Uuid,
    pub hours: f32,
    pub year: u32,
}

#[automock(type Context=(); type Transaction = MockTransaction;)]
#[async_trait]
pub trait ShiftplanReportService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    async fn extract_shiftplan_report(
        &self,
        sales_person_id: Uuid,
        from_year: u32,
        from_week: u8,
        to_year: u32,
        to_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[ShiftplanReportDay]>, ServiceError>;

    async fn extract_quick_shiftplan_report(
        &self,
        year: u32,
        until_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[ShiftplanQuickOverview]>, ServiceError>;

    async fn extract_shiftplan_report_for_week(
        &self,
        year: u32,
        calendar_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[ShiftplanReportDay]>, ServiceError>;
}
