use core::fmt;
use std::{
    fmt::{Display, Formatter},
    rc::Rc,
};

use rest_types::{
    EmployeeReportTO, ExtraHoursCategoryTO, ExtraHoursReportCategoryTO, ExtraHoursTO,
    ShortEmployeeReportTO, WorkingHoursReportTO,
};
use uuid::Uuid;

use crate::i18n;

use super::shiftplan::SalesPerson;

/// The expected amount of time an employee should work in a given time period.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkingSchedule {
    pub required_hours_amount: f32,
    pub valid_from: time::Date,
    pub valid_to: time::Date,
}

/// The category of working hours.
#[derive(Debug, Clone, PartialEq)]
pub enum WorkingHoursCategory {
    Shiftplan,
    ExtraWork(Rc<str>),
    Vacation,
    SickLeave,
    Holiday,
    Unavailable,
}
impl WorkingHoursCategory {
    pub fn is_extra_work(&self) -> bool {
        matches!(self, WorkingHoursCategory::ExtraWork(_))
    }
    pub fn is_vacation(&self) -> bool {
        matches!(self, WorkingHoursCategory::Vacation)
    }
    pub fn is_sick_leave(&self) -> bool {
        matches!(self, WorkingHoursCategory::SickLeave)
    }
    pub fn is_holiday(&self) -> bool {
        matches!(self, WorkingHoursCategory::Holiday)
    }
    pub fn is_unavailable(&self) -> bool {
        matches!(self, WorkingHoursCategory::Unavailable)
    }
    pub fn identifier(&self) -> Rc<str> {
        match self {
            WorkingHoursCategory::Shiftplan => "shiftplan".into(),
            WorkingHoursCategory::ExtraWork(_) => "extra_work".into(),
            WorkingHoursCategory::Vacation => "vacation".into(),
            WorkingHoursCategory::SickLeave => "sick_leave".into(),
            WorkingHoursCategory::Holiday => "holiday".into(),
            WorkingHoursCategory::Unavailable => "unavailable".into(),
        }
    }
    pub fn from_identifier(identifier: &str) -> Self {
        match identifier {
            "shiftplan" => WorkingHoursCategory::Shiftplan,
            "extra_work" => WorkingHoursCategory::ExtraWork("".into()),
            "vacation" => WorkingHoursCategory::Vacation,
            "sick_leave" => WorkingHoursCategory::SickLeave,
            "holiday" => WorkingHoursCategory::Holiday,
            "unavailable" => WorkingHoursCategory::Unavailable,
            _ => panic!("Unknown working hours category: {}", identifier),
        }
    }
    pub fn to_i18n_key(&self) -> i18n::Key {
        match self {
            WorkingHoursCategory::Shiftplan => i18n::Key::CategoryShiftplan,
            WorkingHoursCategory::ExtraWork(_) => i18n::Key::CategoryExtraWork,
            WorkingHoursCategory::Vacation => i18n::Key::CategoryVacation,
            WorkingHoursCategory::SickLeave => i18n::Key::CategorySickLeave,
            WorkingHoursCategory::Holiday => i18n::Key::CategoryHolidays,
            WorkingHoursCategory::Unavailable => i18n::Key::CategoryUnavailable,
        }
    }
}

impl Display for WorkingHoursCategory {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            WorkingHoursCategory::Shiftplan => write!(f, "Shiftplan"),
            WorkingHoursCategory::ExtraWork(reason) => write!(f, "Extra work: {}", reason),
            WorkingHoursCategory::Vacation => write!(f, "Vacation"),
            WorkingHoursCategory::SickLeave => write!(f, "Sick leave"),
            WorkingHoursCategory::Holiday => write!(f, "Holiday"),
            WorkingHoursCategory::Unavailable => write!(f, "Unavailable"),
        }
    }
}

impl From<&ExtraHoursReportCategoryTO> for WorkingHoursCategory {
    fn from(category: &ExtraHoursReportCategoryTO) -> Self {
        match category {
            ExtraHoursReportCategoryTO::Shiftplan => WorkingHoursCategory::Shiftplan,
            ExtraHoursReportCategoryTO::ExtraWork => WorkingHoursCategory::ExtraWork("-".into()),
            ExtraHoursReportCategoryTO::Vacation => WorkingHoursCategory::Vacation,
            ExtraHoursReportCategoryTO::SickLeave => WorkingHoursCategory::SickLeave,
            ExtraHoursReportCategoryTO::Holiday => WorkingHoursCategory::Holiday,
            ExtraHoursReportCategoryTO::Unavailable => WorkingHoursCategory::Unavailable,
        }
    }
}
impl From<&WorkingHoursCategory> for ExtraHoursCategoryTO {
    fn from(category: &WorkingHoursCategory) -> Self {
        match category {
            WorkingHoursCategory::ExtraWork(_) => ExtraHoursCategoryTO::ExtraWork,
            WorkingHoursCategory::Vacation => ExtraHoursCategoryTO::Vacation,
            WorkingHoursCategory::SickLeave => ExtraHoursCategoryTO::SickLeave,
            WorkingHoursCategory::Holiday => ExtraHoursCategoryTO::Holiday,
            WorkingHoursCategory::Unavailable => ExtraHoursCategoryTO::Unavailable,
            _ => panic!(
                "Cannot convert working hours category to extra hours category: {:?}",
                category
            ),
        }
    }
}
impl From<&ExtraHoursCategoryTO> for WorkingHoursCategory {
    fn from(category: &ExtraHoursCategoryTO) -> Self {
        match category {
            ExtraHoursCategoryTO::ExtraWork => WorkingHoursCategory::ExtraWork("-".into()),
            ExtraHoursCategoryTO::Vacation => WorkingHoursCategory::Vacation,
            ExtraHoursCategoryTO::SickLeave => WorkingHoursCategory::SickLeave,
            ExtraHoursCategoryTO::Holiday => WorkingHoursCategory::Holiday,
            ExtraHoursCategoryTO::Unavailable => WorkingHoursCategory::Unavailable,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorkingHoursDay {
    pub date: time::Date,
    pub hours: f32,
    pub category: WorkingHoursCategory,
}

/// The amount of time an employee worked in a given time period.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkingHours {
    pub from: time::Date,
    pub to: time::Date,
    pub expected_hours: f32,
    pub overall_hours: f32,
    pub balance: f32,

    pub shiftplan_hours: f32,
    pub extra_work_hours: f32,
    pub vacation_hours: f32,
    pub sick_leave_hours: f32,
    pub holiday_hours: f32,

    pub days: Rc<[WorkingHoursDay]>,
}

impl From<&WorkingHoursReportTO> for WorkingHours {
    fn from(working_hours: &WorkingHoursReportTO) -> Self {
        WorkingHours {
            from: working_hours.from,
            to: working_hours.to,
            expected_hours: working_hours.expected_hours,
            overall_hours: working_hours.overall_hours,
            balance: working_hours.balance,
            shiftplan_hours: working_hours.shiftplan_hours,
            extra_work_hours: working_hours.extra_work_hours,
            vacation_hours: working_hours.vacation_hours,
            sick_leave_hours: working_hours.sick_leave_hours,
            holiday_hours: working_hours.holiday_hours,
            days: working_hours
                .days
                .iter()
                .map(|day| WorkingHoursDay {
                    date: day.date,
                    hours: day.hours,
                    category: (&day.category).into(),
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Employee {
    pub sales_person: SalesPerson,
    pub working_hours_by_week: Rc<[WorkingHours]>,
    pub working_hours_by_month: Rc<[WorkingHours]>,

    pub overall_working_hours: f32,
    pub expected_working_hours: f32,
    pub balance: f32,

    pub shiftplan_hours: f32,
    pub extra_work_hours: f32,
    pub vacation_hours: f32,
    pub sick_leave_hours: f32,
    pub holiday_hours: f32,
}

impl From<&ShortEmployeeReportTO> for Employee {
    fn from(report: &ShortEmployeeReportTO) -> Self {
        Employee {
            sales_person: (&report.sales_person).into(),
            working_hours_by_week: [].into(),
            working_hours_by_month: [].into(),
            overall_working_hours: 0.0,
            expected_working_hours: 0.0,
            balance: report.balance_hours,
            shiftplan_hours: 0.0,
            extra_work_hours: 0.0,
            vacation_hours: 0.0,
            sick_leave_hours: 0.0,
            holiday_hours: 0.0,
        }
    }
}

impl From<&EmployeeReportTO> for Employee {
    fn from(report: &EmployeeReportTO) -> Self {
        Employee {
            sales_person: report.sales_person.as_ref().into(),
            working_hours_by_month: [].into(),
            working_hours_by_week: report.by_week.iter().map(WorkingHours::from).collect(),
            //working_hours_by_month: report
            //    .working_hours_by_month
            //    .iter()
            //    .map(WorkingHours::from)
            //    .collect(),
            overall_working_hours: report.overall_hours,
            expected_working_hours: report.expected_hours,
            balance: report.balance_hours,
            shiftplan_hours: report.shiftplan_hours,
            extra_work_hours: report.extra_work_hours,
            vacation_hours: report.vacation_hours,
            sick_leave_hours: report.sick_leave_hours,
            holiday_hours: report.holiday_hours,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExtraHours {
    pub id: Uuid,
    pub sales_person_id: Uuid,
    pub amount: f32,
    pub category: WorkingHoursCategory,
    pub description: Rc<str>,
    pub date_time: time::PrimitiveDateTime,
    pub version: Uuid,
}
impl From<&ExtraHoursTO> for ExtraHours {
    fn from(extra_hours: &ExtraHoursTO) -> Self {
        ExtraHours {
            id: extra_hours.id,
            sales_person_id: extra_hours.sales_person_id,
            amount: extra_hours.amount,
            category: (&extra_hours.category).into(),
            description: extra_hours.description.as_ref().into(),
            date_time: extra_hours.date_time,
            version: extra_hours.version,
        }
    }
}
