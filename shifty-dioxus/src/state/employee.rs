use core::fmt;
use std::{
    fmt::{Display, Formatter},
    rc::Rc,
};

use rest_types::{
    EmployeeReportTO, ExtraHoursCategoryTO, ShortEmployeeReportTO, WorkingHoursReportTO,
    WorkingHoursTO,
};
use time::Month;
use uuid::uuid;

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
}

impl Display for WorkingHoursCategory {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            WorkingHoursCategory::Shiftplan => write!(f, "Shiftplan"),
            WorkingHoursCategory::ExtraWork(reason) => write!(f, "Extra work: {}", reason),
            WorkingHoursCategory::Vacation => write!(f, "Vacation"),
            WorkingHoursCategory::SickLeave => write!(f, "Sick leave"),
            WorkingHoursCategory::Holiday => write!(f, "Holiday"),
        }
    }
}

impl From<&ExtraHoursCategoryTO> for WorkingHoursCategory {
    fn from(category: &ExtraHoursCategoryTO) -> Self {
        match category {
            ExtraHoursCategoryTO::Shiftplan => WorkingHoursCategory::Shiftplan,
            ExtraHoursCategoryTO::ExtraWork => WorkingHoursCategory::ExtraWork("-".into()),
            ExtraHoursCategoryTO::Vacation => WorkingHoursCategory::Vacation,
            ExtraHoursCategoryTO::SickLeave => WorkingHoursCategory::SickLeave,
            ExtraHoursCategoryTO::Holiday => WorkingHoursCategory::Holiday,
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

pub fn build_dummy_data() -> Employee {
    Employee {
        sales_person: SalesPerson {
            id: uuid!("b45be9d4-00f2-4f38-99af-8ced70eeec00"),
            name: "Maxime Musterfrau".into(),
            background_color: "#FFF".into(),
        },
        working_hours_by_week: [
            WorkingHours {
                from: time::Date::from_calendar_date(2024, Month::January, 1).unwrap(),
                to: time::Date::from_calendar_date(2024, Month::January, 7).unwrap(),
                expected_hours: 40.0,
                overall_hours: 40.0,
                balance: 0.0,
                shiftplan_hours: 8.0 * 3.0,
                extra_work_hours: 2.0,
                vacation_hours: 0.0,
                sick_leave_hours: 0.0,
                holiday_hours: 16.0,
                days: [
                    WorkingHoursDay {
                        date: time::Date::from_calendar_date(2024, Month::January, 1).unwrap(),
                        hours: 8.0,
                        category: WorkingHoursCategory::Holiday,
                    },
                    WorkingHoursDay {
                        date: time::Date::from_calendar_date(2024, Month::January, 2).unwrap(),
                        hours: 8.0,
                        category: WorkingHoursCategory::Shiftplan,
                    },
                    WorkingHoursDay {
                        date: time::Date::from_calendar_date(2024, Month::January, 2).unwrap(),
                        hours: 2.0,
                        category: WorkingHoursCategory::ExtraWork(
                            "Counted the coffee beans".into(),
                        ),
                    },
                    WorkingHoursDay {
                        date: time::Date::from_calendar_date(2024, Month::January, 3).unwrap(),
                        hours: 8.0,
                        category: WorkingHoursCategory::Shiftplan,
                    },
                    WorkingHoursDay {
                        date: time::Date::from_calendar_date(2024, Month::January, 4).unwrap(),
                        hours: 8.0,
                        category: WorkingHoursCategory::Shiftplan,
                    },
                    WorkingHoursDay {
                        date: time::Date::from_calendar_date(2024, Month::January, 5).unwrap(),
                        hours: 8.0,
                        category: WorkingHoursCategory::Holiday,
                    },
                ]
                .into(),
            },
            WorkingHours {
                from: time::Date::from_calendar_date(2024, Month::January, 8).unwrap(),
                to: time::Date::from_calendar_date(2024, Month::January, 14).unwrap(),
                expected_hours: 40.0,
                overall_hours: 38.0,
                balance: 0.0,
                shiftplan_hours: 38.0,
                extra_work_hours: 0.0,
                vacation_hours: 0.0,
                sick_leave_hours: 0.0,
                holiday_hours: 0.0,
                days: [
                    WorkingHoursDay {
                        date: time::Date::from_calendar_date(2024, Month::January, 8).unwrap(),
                        hours: 8.0,
                        category: WorkingHoursCategory::Shiftplan,
                    },
                    WorkingHoursDay {
                        date: time::Date::from_calendar_date(2024, Month::January, 9).unwrap(),
                        hours: 8.0,
                        category: WorkingHoursCategory::Shiftplan,
                    },
                    WorkingHoursDay {
                        date: time::Date::from_calendar_date(2024, Month::January, 10).unwrap(),
                        hours: 8.0,
                        category: WorkingHoursCategory::Shiftplan,
                    },
                    WorkingHoursDay {
                        date: time::Date::from_calendar_date(2024, Month::January, 11).unwrap(),
                        hours: 8.0,
                        category: WorkingHoursCategory::Shiftplan,
                    },
                    WorkingHoursDay {
                        date: time::Date::from_calendar_date(2024, Month::January, 12).unwrap(),
                        hours: 8.0,
                        category: WorkingHoursCategory::Shiftplan,
                    },
                ]
                .into(),
            },
        ]
        .into(),
        working_hours_by_month: [WorkingHours {
            from: time::Date::from_calendar_date(2024, Month::January, 1).unwrap(),
            to: time::Date::from_calendar_date(2024, Month::January, 31).unwrap(),
            expected_hours: 40.0 * 23.0,
            overall_hours: 40.0 * 23.0 - 3.0,
            balance: -3.0,
            shiftplan_hours: 40.0 * 20.0,
            extra_work_hours: 2.0,
            vacation_hours: 0.0,
            sick_leave_hours: 0.0,
            holiday_hours: 16.0,
            days: [
                WorkingHoursDay {
                    date: time::Date::from_calendar_date(2024, Month::January, 1).unwrap(),
                    hours: 8.0,
                    category: WorkingHoursCategory::Holiday,
                },
                WorkingHoursDay {
                    date: time::Date::from_calendar_date(2024, Month::January, 2).unwrap(),
                    hours: 8.0,
                    category: WorkingHoursCategory::Shiftplan,
                },
                WorkingHoursDay {
                    date: time::Date::from_calendar_date(2024, Month::January, 2).unwrap(),
                    hours: 2.0,
                    category: WorkingHoursCategory::ExtraWork("Counted the coffee beans".into()),
                },
                WorkingHoursDay {
                    date: time::Date::from_calendar_date(2024, Month::January, 3).unwrap(),
                    hours: 8.0,
                    category: WorkingHoursCategory::Shiftplan,
                },
                WorkingHoursDay {
                    date: time::Date::from_calendar_date(2024, Month::January, 4).unwrap(),
                    hours: 8.0,
                    category: WorkingHoursCategory::Shiftplan,
                },
                WorkingHoursDay {
                    date: time::Date::from_calendar_date(2024, Month::January, 5).unwrap(),
                    hours: 8.0,
                    category: WorkingHoursCategory::Holiday,
                },
                WorkingHoursDay {
                    date: time::Date::from_calendar_date(2024, Month::January, 8).unwrap(),
                    hours: 8.0,
                    category: WorkingHoursCategory::Shiftplan,
                },
                WorkingHoursDay {
                    date: time::Date::from_calendar_date(2024, Month::January, 9).unwrap(),
                    hours: 8.0,
                    category: WorkingHoursCategory::Shiftplan,
                },
                WorkingHoursDay {
                    date: time::Date::from_calendar_date(2024, Month::January, 10).unwrap(),
                    hours: 8.0,
                    category: WorkingHoursCategory::Shiftplan,
                },
                WorkingHoursDay {
                    date: time::Date::from_calendar_date(2024, Month::January, 11).unwrap(),
                    hours: 8.0,
                    category: WorkingHoursCategory::Shiftplan,
                },
                WorkingHoursDay {
                    date: time::Date::from_calendar_date(2024, Month::January, 12).unwrap(),
                    hours: 8.0,
                    category: WorkingHoursCategory::Shiftplan,
                },
                WorkingHoursDay {
                    date: time::Date::from_calendar_date(2024, Month::January, 15).unwrap(),
                    hours: 8.0,
                    category: WorkingHoursCategory::Shiftplan,
                },
                WorkingHoursDay {
                    date: time::Date::from_calendar_date(2024, Month::January, 16).unwrap(),
                    hours: 8.0,
                    category: WorkingHoursCategory::Shiftplan,
                },
                WorkingHoursDay {
                    date: time::Date::from_calendar_date(2024, Month::January, 17).unwrap(),
                    hours: 8.0,
                    category: WorkingHoursCategory::Shiftplan,
                },
                WorkingHoursDay {
                    date: time::Date::from_calendar_date(2024, Month::January, 18).unwrap(),
                    hours: 8.0,
                    category: WorkingHoursCategory::Shiftplan,
                },
                WorkingHoursDay {
                    date: time::Date::from_calendar_date(2024, Month::January, 19).unwrap(),
                    hours: 8.0,
                    category: WorkingHoursCategory::Shiftplan,
                },
                WorkingHoursDay {
                    date: time::Date::from_calendar_date(2024, Month::January, 22).unwrap(),
                    hours: 8.0,
                    category: WorkingHoursCategory::Shiftplan,
                },
                WorkingHoursDay {
                    date: time::Date::from_calendar_date(2024, Month::January, 23).unwrap(),
                    hours: 8.0,
                    category: WorkingHoursCategory::Shiftplan,
                },
                WorkingHoursDay {
                    date: time::Date::from_calendar_date(2024, Month::January, 24).unwrap(),
                    hours: 8.0,
                    category: WorkingHoursCategory::Shiftplan,
                },
                WorkingHoursDay {
                    date: time::Date::from_calendar_date(2024, Month::January, 25).unwrap(),
                    hours: 8.0,
                    category: WorkingHoursCategory::Shiftplan,
                },
                WorkingHoursDay {
                    date: time::Date::from_calendar_date(2024, Month::January, 26).unwrap(),
                    hours: 8.0,
                    category: WorkingHoursCategory::Shiftplan,
                },
                WorkingHoursDay {
                    date: time::Date::from_calendar_date(2024, Month::January, 29).unwrap(),
                    hours: 8.0,
                    category: WorkingHoursCategory::Shiftplan,
                },
                WorkingHoursDay {
                    date: time::Date::from_calendar_date(2024, Month::January, 30).unwrap(),
                    hours: 8.0,
                    category: WorkingHoursCategory::Shiftplan,
                },
                WorkingHoursDay {
                    date: time::Date::from_calendar_date(2024, Month::January, 31).unwrap(),
                    hours: 8.0,
                    category: WorkingHoursCategory::Shiftplan,
                },
            ]
            .into(),
        }]
        .into(),
        overall_working_hours: 1337.0,
        expected_working_hours: 1330.0,
        balance: 7.0,
        shiftplan_hours: 1250.0,
        extra_work_hours: 10.0,
        vacation_hours: 40.0,
        sick_leave_hours: 8.0,
        holiday_hours: 32.0,
    }
}
