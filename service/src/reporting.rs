use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use uuid::Uuid;

use crate::permission::Authentication;
use crate::sales_person::SalesPerson;
use crate::ServiceError;

#[derive(Clone, Debug, PartialEq)]
pub enum ExtraHoursReportCategory {
    Shiftplan,
    ExtraWork,
    Vacation,
    SickLeave,
    Holiday,
}

impl From<&crate::extra_hours::ExtraHoursCategory> for ExtraHoursReportCategory {
    fn from(category: &crate::extra_hours::ExtraHoursCategory) -> Self {
        match category {
            crate::extra_hours::ExtraHoursCategory::ExtraWork => Self::ExtraWork,
            crate::extra_hours::ExtraHoursCategory::Vacation => Self::Vacation,
            crate::extra_hours::ExtraHoursCategory::SickLeave => Self::SickLeave,
            crate::extra_hours::ExtraHoursCategory::Holiday => Self::Holiday,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorkingHoursDay {
    pub date: time::Date,
    pub hours: f32,
    pub category: ExtraHoursReportCategory,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GroupedReportHours {
    pub from: time::Date,
    pub to: time::Date,
    pub contract_weekly_hours: f32,
    pub expected_hours: f32,
    pub overall_hours: f32,
    pub balance: f32,

    pub days_per_week: u8,
    pub workdays_per_week: u8,

    pub shiftplan_hours: f32,
    pub extra_work_hours: f32,
    pub vacation_hours: f32,
    pub sick_leave_hours: f32,
    pub holiday_hours: f32,

    pub days: Arc<[WorkingHoursDay]>,
}
impl GroupedReportHours {
    pub fn hours_per_day(&self) -> f32 {
        if self.workdays_per_week == 0 {
            return 0.0;
        }
        self.contract_weekly_hours / self.workdays_per_week as f32
    }
    pub fn hours_per_holiday(&self) -> f32 {
        if self.days_per_week == 0 {
            return 0.0;
        }
        self.contract_weekly_hours / self.days_per_week as f32
    }

    pub fn vacation_days(&self) -> f32 {
        if self.hours_per_day() == 0.0 {
            return 0.0;
        }
        self.vacation_hours / self.hours_per_day()
    }

    pub fn sick_leave_days(&self) -> f32 {
        if self.hours_per_day() == 0.0 {
            return 0.0;
        }
        self.sick_leave_hours / self.hours_per_day()
    }

    pub fn holiday_days(&self) -> f32 {
        if self.hours_per_day() == 0.0 {
            return 0.0;
        }
        self.holiday_hours / self.hours_per_holiday()
    }

    pub fn absence_days(&self) -> f32 {
        if self.hours_per_day() == 0.0 {
            return 0.0;
        }
        (self.vacation_hours + self.sick_leave_hours + self.holiday_hours) / self.hours_per_day()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ShortEmployeeReport {
    pub sales_person: Arc<SalesPerson>,
    pub balance_hours: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EmployeeReport {
    pub sales_person: Arc<SalesPerson>,

    pub balance_hours: f32,
    pub overall_hours: f32,
    pub expected_hours: f32,

    pub shiftplan_hours: f32,
    pub extra_work_hours: f32,
    pub vacation_hours: f32,
    pub sick_leave_hours: f32,
    pub holiday_hours: f32,

    pub vacation_days: f32,
    pub sick_leave_days: f32,
    pub holiday_days: f32,
    pub absence_days: f32,

    pub by_week: Arc<[GroupedReportHours]>,
    pub by_month: Arc<[GroupedReportHours]>,
}

#[automock(type Context=();)]
#[async_trait]
pub trait ReportingService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;

    async fn get_reports_for_all_employees(
        &self,
        years: u32,
        until_week: u8,
        context: Authentication<Self::Context>,
    ) -> Result<Arc<[ShortEmployeeReport]>, ServiceError>;

    async fn get_report_for_employee(
        &self,
        sales_person_id: &Uuid,
        years: u32,
        until_week: u8,
        context: Authentication<Self::Context>,
    ) -> Result<EmployeeReport, ServiceError>;
}
