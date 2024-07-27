use std::sync::Arc;

use serde::{Deserialize, Serialize};
#[cfg(feature = "service-impl")]
use service::{booking::Booking, sales_person::SalesPerson};
use time::PrimitiveDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserTO {
    pub name: String,
}
#[cfg(feature = "service-impl")]
impl From<&service::User> for UserTO {
    fn from(user: &service::User) -> Self {
        Self {
            name: user.name.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoleTO {
    pub name: String,
}
#[cfg(feature = "service-impl")]
impl From<&service::Role> for RoleTO {
    fn from(role: &service::Role) -> Self {
        Self {
            name: role.name.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrivilegeTO {
    pub name: String,
}
#[cfg(feature = "service-impl")]
impl From<&service::Privilege> for PrivilegeTO {
    fn from(privilege: &service::Privilege) -> Self {
        Self {
            name: privilege.name.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserRole {
    pub user: String,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RolePrivilege {
    pub role: String,
    pub privilege: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BookingTO {
    #[serde(default)]
    pub id: Uuid,
    pub sales_person_id: Uuid,
    pub slot_id: Uuid,
    pub calendar_week: i32,
    pub year: u32,
    #[serde(default)]
    pub created: Option<PrimitiveDateTime>,
    #[serde(default)]
    pub deleted: Option<PrimitiveDateTime>,
    #[serde(rename = "$version")]
    #[serde(default)]
    pub version: Uuid,
}
#[cfg(feature = "service-impl")]
impl From<&Booking> for BookingTO {
    fn from(booking: &Booking) -> Self {
        Self {
            id: booking.id,
            sales_person_id: booking.sales_person_id,
            slot_id: booking.slot_id,
            calendar_week: booking.calendar_week,
            year: booking.year,
            created: booking.created,
            deleted: booking.deleted,
            version: booking.version,
        }
    }
}
#[cfg(feature = "service-impl")]
impl From<&BookingTO> for Booking {
    fn from(booking: &BookingTO) -> Self {
        Self {
            id: booking.id,
            sales_person_id: booking.sales_person_id,
            slot_id: booking.slot_id,
            calendar_week: booking.calendar_week,
            year: booking.year,
            created: booking.created,
            deleted: booking.deleted,
            version: booking.version,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SalesPersonTO {
    #[serde(default)]
    pub id: Uuid,
    pub name: Arc<str>,
    pub background_color: Arc<str>,
    #[serde(default)]
    pub is_paid: Option<bool>,
    #[serde(default)]
    pub inactive: bool,
    #[serde(default)]
    pub deleted: Option<time::PrimitiveDateTime>,
    #[serde(rename = "$version")]
    #[serde(default)]
    pub version: Uuid,
}
#[cfg(feature = "service-impl")]
impl From<&SalesPerson> for SalesPersonTO {
    fn from(sales_person: &SalesPerson) -> Self {
        Self {
            id: sales_person.id,
            name: sales_person.name.clone(),
            background_color: sales_person.background_color.clone(),
            is_paid: sales_person.is_paid,
            inactive: sales_person.inactive,
            deleted: sales_person.deleted,
            version: sales_person.version,
        }
    }
}
#[cfg(feature = "service-impl")]
impl From<&SalesPersonTO> for SalesPerson {
    fn from(sales_person: &SalesPersonTO) -> Self {
        Self {
            id: sales_person.id,
            name: sales_person.name.clone(),
            background_color: sales_person.background_color.clone(),
            is_paid: sales_person.is_paid,
            inactive: sales_person.inactive,
            deleted: sales_person.deleted,
            version: sales_person.version,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum DayOfWeekTO {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}
#[cfg(feature = "service-impl")]
impl From<service::slot::DayOfWeek> for DayOfWeekTO {
    fn from(day_of_week: service::slot::DayOfWeek) -> Self {
        match day_of_week {
            service::slot::DayOfWeek::Monday => Self::Monday,
            service::slot::DayOfWeek::Tuesday => Self::Tuesday,
            service::slot::DayOfWeek::Wednesday => Self::Wednesday,
            service::slot::DayOfWeek::Thursday => Self::Thursday,
            service::slot::DayOfWeek::Friday => Self::Friday,
            service::slot::DayOfWeek::Saturday => Self::Saturday,
            service::slot::DayOfWeek::Sunday => Self::Sunday,
        }
    }
}
#[cfg(feature = "service-impl")]
impl From<DayOfWeekTO> for service::slot::DayOfWeek {
    fn from(day_of_week: DayOfWeekTO) -> Self {
        match day_of_week {
            DayOfWeekTO::Monday => Self::Monday,
            DayOfWeekTO::Tuesday => Self::Tuesday,
            DayOfWeekTO::Wednesday => Self::Wednesday,
            DayOfWeekTO::Thursday => Self::Thursday,
            DayOfWeekTO::Friday => Self::Friday,
            DayOfWeekTO::Saturday => Self::Saturday,
            DayOfWeekTO::Sunday => Self::Sunday,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlotTO {
    #[serde(default)]
    pub id: Uuid,
    pub day_of_week: DayOfWeekTO,
    pub from: time::Time,
    pub to: time::Time,
    pub valid_from: time::Date,
    pub valid_to: Option<time::Date>,
    #[serde(default)]
    pub deleted: Option<time::PrimitiveDateTime>,
    #[serde(rename = "$version")]
    #[serde(default)]
    pub version: Uuid,
}
#[cfg(feature = "service-impl")]
impl From<&service::slot::Slot> for SlotTO {
    fn from(slot: &service::slot::Slot) -> Self {
        Self {
            id: slot.id,
            day_of_week: slot.day_of_week.into(),
            from: slot.from,
            to: slot.to,
            valid_from: slot.valid_from,
            valid_to: slot.valid_to,
            deleted: slot.deleted,
            version: slot.version,
        }
    }
}
#[cfg(feature = "service-impl")]
impl From<&SlotTO> for service::slot::Slot {
    fn from(slot: &SlotTO) -> Self {
        Self {
            id: slot.id,
            day_of_week: slot.day_of_week.into(),
            from: slot.from,
            to: slot.to,
            valid_from: slot.valid_from,
            valid_to: slot.valid_to,
            deleted: slot.deleted,
            version: slot.version,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShortEmployeeReportTO {
    pub sales_person: SalesPersonTO,
    pub balance_hours: f32,
}

#[cfg(feature = "service-impl")]
impl From<&service::reporting::ShortEmployeeReport> for ShortEmployeeReportTO {
    fn from(report: &service::reporting::ShortEmployeeReport) -> Self {
        Self {
            sales_person: SalesPersonTO::from(report.sales_person.as_ref()),
            balance_hours: report.balance_hours,
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub enum ExtraHoursReportCategoryTO {
    Shiftplan,
    ExtraWork,
    Vacation,
    SickLeave,
    Holiday,
}
#[cfg(feature = "service-impl")]
impl From<&service::reporting::ExtraHoursReportCategory> for ExtraHoursReportCategoryTO {
    fn from(category: &service::reporting::ExtraHoursReportCategory) -> Self {
        match category {
            service::reporting::ExtraHoursReportCategory::Shiftplan => Self::Shiftplan,
            service::reporting::ExtraHoursReportCategory::ExtraWork => Self::ExtraWork,
            service::reporting::ExtraHoursReportCategory::Vacation => Self::Vacation,
            service::reporting::ExtraHoursReportCategory::SickLeave => Self::SickLeave,
            service::reporting::ExtraHoursReportCategory::Holiday => Self::Holiday,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkingHoursDayTO {
    pub date: time::Date,
    pub hours: f32,
    pub category: ExtraHoursReportCategoryTO,
}
#[cfg(feature = "service-impl")]
impl From<&service::reporting::WorkingHoursDay> for WorkingHoursDayTO {
    fn from(day: &service::reporting::WorkingHoursDay) -> Self {
        Self {
            date: day.date,
            hours: day.hours,
            category: (&day.category).into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkingHoursReportTO {
    pub from: time::Date,
    pub to: time::Date,
    pub expected_hours: f32,
    pub overall_hours: f32,
    pub balance: f32,

    pub days_per_week: u8,
    pub workdays_per_week: u8,

    pub shiftplan_hours: f32,
    pub extra_work_hours: f32,
    pub vacation_hours: f32,
    pub vacation_days: f32,
    pub sick_leave_hours: f32,
    pub sick_leave_days: f32,
    pub holiday_hours: f32,
    pub holiday_days: f32,
    pub absence_days: f32,

    pub days: Arc<[WorkingHoursDayTO]>,
}

#[cfg(feature = "service-impl")]
impl From<&service::reporting::GroupedReportHours> for WorkingHoursReportTO {
    fn from(hours: &service::reporting::GroupedReportHours) -> Self {
        Self {
            from: hours.from,
            to: hours.to,
            expected_hours: hours.expected_hours,
            overall_hours: hours.overall_hours,
            balance: hours.balance,

            days_per_week: hours.days_per_week,
            workdays_per_week: hours.workdays_per_week,

            shiftplan_hours: hours.shiftplan_hours,
            extra_work_hours: hours.extra_work_hours,
            vacation_hours: hours.vacation_hours,
            vacation_days: hours.vacation_days(),
            sick_leave_hours: hours.sick_leave_hours,
            sick_leave_days: hours.sick_leave_days(),
            holiday_hours: hours.holiday_hours,
            holiday_days: hours.holiday_days(),
            absence_days: hours.absence_days(),
            days: hours
                .days
                .iter()
                .map(|day| WorkingHoursDayTO::from(day))
                .collect::<Vec<_>>()
                .into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmployeeReportTO {
    pub sales_person: Arc<SalesPersonTO>,

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

    pub by_week: Arc<[WorkingHoursReportTO]>,
    pub by_month: Arc<[WorkingHoursReportTO]>,
}
#[cfg(feature = "service-impl")]

impl From<&service::reporting::EmployeeReport> for EmployeeReportTO {
    fn from(report: &service::reporting::EmployeeReport) -> Self {
        Self {
            sales_person: Arc::new(SalesPersonTO::from(report.sales_person.as_ref())),
            balance_hours: report.balance_hours,
            overall_hours: report.overall_hours,
            expected_hours: report.expected_hours,
            shiftplan_hours: report.shiftplan_hours,
            extra_work_hours: report.extra_work_hours,
            vacation_hours: report.vacation_hours,
            sick_leave_hours: report.sick_leave_hours,
            vacation_days: report.vacation_days,
            sick_leave_days: report.sick_leave_days,
            holiday_days: report.holiday_days,
            holiday_hours: report.holiday_hours,
            absence_days: report.absence_days,
            by_week: report
                .by_week
                .iter()
                .map(|hours| WorkingHoursReportTO::from(hours))
                .collect::<Vec<_>>()
                .into(),
            by_month: report
                .by_month
                .iter()
                .map(|hours| WorkingHoursReportTO::from(hours))
                .collect::<Vec<_>>()
                .into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkingHoursTO {
    #[serde(default)]
    pub id: Uuid,
    pub sales_person_id: Uuid,
    pub expected_hours: f32,
    pub from_calendar_week: u8,
    pub from_year: u32,
    pub to_calendar_week: u8,
    pub to_year: u32,
    pub workdays_per_week: u8,
    pub days_per_week: u8,
    pub hours_per_day: f32,
    pub hours_per_holiday: f32,
    #[serde(default)]
    pub created: Option<time::PrimitiveDateTime>,
    #[serde(default)]
    pub deleted: Option<time::PrimitiveDateTime>,
    #[serde(rename = "$version")]
    #[serde(default)]
    pub version: Uuid,
}
#[cfg(feature = "service-impl")]
impl From<&service::working_hours::WorkingHours> for WorkingHoursTO {
    fn from(working_hours: &service::working_hours::WorkingHours) -> Self {
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
            hours_per_day: working_hours.hours_per_day(),
            hours_per_holiday: working_hours.holiday_hours(),
            created: working_hours.created,
            deleted: working_hours.deleted,
            version: working_hours.version,
        }
    }
}

#[cfg(feature = "service-impl")]
impl From<&WorkingHoursTO> for service::working_hours::WorkingHours {
    fn from(working_hours: &WorkingHoursTO) -> Self {
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
            created: working_hours.created,
            deleted: working_hours.deleted,
            version: working_hours.version,
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ExtraHoursCategoryTO {
    ExtraWork,
    Vacation,
    SickLeave,
    Holiday,
}
#[cfg(feature = "service-impl")]
impl From<&service::extra_hours::ExtraHoursCategory> for ExtraHoursCategoryTO {
    fn from(category: &service::extra_hours::ExtraHoursCategory) -> Self {
        match category {
            service::extra_hours::ExtraHoursCategory::ExtraWork => Self::ExtraWork,
            service::extra_hours::ExtraHoursCategory::Vacation => Self::Vacation,
            service::extra_hours::ExtraHoursCategory::SickLeave => Self::SickLeave,
            service::extra_hours::ExtraHoursCategory::Holiday => Self::Holiday,
        }
    }
}
#[cfg(feature = "service-impl")]
impl From<&ExtraHoursCategoryTO> for service::extra_hours::ExtraHoursCategory {
    fn from(category: &ExtraHoursCategoryTO) -> Self {
        match category {
            ExtraHoursCategoryTO::ExtraWork => Self::ExtraWork,
            ExtraHoursCategoryTO::Vacation => Self::Vacation,
            ExtraHoursCategoryTO::SickLeave => Self::SickLeave,
            ExtraHoursCategoryTO::Holiday => Self::Holiday,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExtraHoursTO {
    #[serde(default)]
    pub id: Uuid,
    pub sales_person_id: Uuid,
    pub amount: f32,
    pub category: ExtraHoursCategoryTO,
    pub description: Arc<str>,
    pub date_time: time::PrimitiveDateTime,
    #[serde(default)]
    pub created: Option<time::PrimitiveDateTime>,
    #[serde(default)]
    pub deleted: Option<time::PrimitiveDateTime>,
    #[serde(rename = "$version")]
    #[serde(default)]
    pub version: Uuid,
}
#[cfg(feature = "service-impl")]
impl From<&service::extra_hours::ExtraHours> for ExtraHoursTO {
    fn from(extra_hours: &service::extra_hours::ExtraHours) -> Self {
        Self {
            id: extra_hours.id,
            sales_person_id: extra_hours.sales_person_id,
            amount: extra_hours.amount,
            category: (&extra_hours.category).into(),
            description: extra_hours.description.clone(),
            date_time: extra_hours.date_time,
            created: extra_hours.created,
            deleted: extra_hours.deleted,
            version: extra_hours.version,
        }
    }
}
#[cfg(feature = "service-impl")]
impl From<&ExtraHoursTO> for service::extra_hours::ExtraHours {
    fn from(extra_hours: &ExtraHoursTO) -> Self {
        Self {
            id: extra_hours.id,
            sales_person_id: extra_hours.sales_person_id,
            amount: extra_hours.amount,
            category: (&extra_hours.category).into(),
            description: extra_hours.description.clone(),
            date_time: extra_hours.date_time,
            created: extra_hours.created,
            deleted: extra_hours.deleted,
            version: extra_hours.version,
        }
    }
}
