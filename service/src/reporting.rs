use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use shifty_utils::{LazyLoad, ShiftyDate};
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
    Unavailable,
    Custom(LazyLoad<Uuid, crate::custom_extra_hours::CustomExtraHours>),
}

impl From<&crate::extra_hours::ExtraHoursCategory> for ExtraHoursReportCategory {
    fn from(category: &crate::extra_hours::ExtraHoursCategory) -> Self {
        match category {
            crate::extra_hours::ExtraHoursCategory::ExtraWork => Self::ExtraWork,
            crate::extra_hours::ExtraHoursCategory::Vacation => Self::Vacation,
            crate::extra_hours::ExtraHoursCategory::SickLeave => Self::SickLeave,
            crate::extra_hours::ExtraHoursCategory::Holiday => Self::Holiday,
            crate::extra_hours::ExtraHoursCategory::Unavailable => Self::Unavailable,
            crate::extra_hours::ExtraHoursCategory::CustomExtraHours(lazy_laod) => {
                Self::Custom(lazy_laod.clone())
            }
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
pub struct CustomExtraHours {
    pub id: Uuid,
    pub name: Arc<str>,
    pub hours: f32,
}

impl
    From<(
        &crate::extra_hours::ExtraHours,
        &crate::custom_extra_hours::CustomExtraHours,
    )> for CustomExtraHours // This refers to crate::reporting::CustomExtraHours
{
    fn from(
        (extra_hours_entry, custom_extra_hours_def): (
            &crate::extra_hours::ExtraHours,
            &crate::custom_extra_hours::CustomExtraHours,
        ),
    ) -> Self {
        Self {
            id: custom_extra_hours_def.id,
            name: custom_extra_hours_def.name.clone(),
            hours: extra_hours_entry.amount,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GroupedReportHours {
    pub from: ShiftyDate,
    pub to: ShiftyDate,
    pub year: u32,
    pub week: u8,
    pub contract_weekly_hours: f32,
    pub expected_hours: f32,
    pub overall_hours: f32,
    pub balance: f32,

    pub days_per_week: u8,
    pub workdays_per_week: f32,

    pub shiftplan_hours: f32,
    pub extra_work_hours: f32,
    pub vacation_hours: f32,
    pub sick_leave_hours: f32,
    pub holiday_hours: f32,

    pub custom_extra_hours: Arc<[CustomExtraHours]>,

    pub days: Arc<[WorkingHoursDay]>,
}
impl GroupedReportHours {
    pub fn hours_per_day(&self) -> f32 {
        if self.workdays_per_week == 0.0 {
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
    pub expected_hours: f32,
    pub overall_hours: f32,
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

    pub vacation_carryover: i32,
    pub vacation_days: f32,
    pub vacation_entitlement: f32,
    pub sick_leave_days: f32,
    pub holiday_days: f32,
    pub absence_days: f32,

    pub carryover_hours: f32,

    pub custom_extra_hours: Arc<[CustomExtraHours]>,

    pub by_week: Arc<[GroupedReportHours]>,
    pub by_month: Arc<[GroupedReportHours]>,
}

#[automock(type Context=(); type Transaction=dao::MockTransaction;)]
#[async_trait]
pub trait ReportingService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    async fn get_reports_for_all_employees(
        &self,
        years: u32,
        until_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[ShortEmployeeReport]>, ServiceError>;

    async fn get_report_for_employee(
        &self,
        sales_person_id: &Uuid,
        years: u32,
        until_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<EmployeeReport, ServiceError>;

    async fn get_report_for_employee_range(
        &self,
        sales_person_id: &Uuid,
        from_date: ShiftyDate,
        to_date: ShiftyDate,
        include_carryover: bool,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<EmployeeReport, ServiceError>;

    async fn get_week(
        &self,
        year: u32,
        week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[ShortEmployeeReport]>, ServiceError>;
}
