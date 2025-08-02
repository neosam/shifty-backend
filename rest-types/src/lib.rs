use std::sync::Arc;

use serde::{Deserialize, Serialize};
#[cfg(feature = "service-impl")]
use service::booking_information::{BookingInformation, WeeklySummary, WorkingHoursPerSalesPerson};
#[cfg(feature = "service-impl")]
use service::{booking::Booking, sales_person::SalesPerson};
use shifty_utils::{derive_from_reference, LazyLoad};
use time::{PrimitiveDateTime, Weekday};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
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

#[derive(Debug, Serialize, Deserialize, ToSchema)]
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

#[derive(Debug, Serialize, Deserialize, ToSchema)]
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

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserRole {
    pub user: String,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
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
    #[serde(default)]
    pub created_by: Option<Arc<str>>,
    #[serde(default)]
    pub deleted_by: Option<Arc<str>>,
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
            created_by: booking.created_by.clone(),
            deleted_by: booking.deleted_by.clone(),
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
            created_by: booking.created_by.clone(),
            deleted_by: booking.deleted_by.clone(),
            version: booking.version,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
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

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, ToSchema)]
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
impl From<Weekday> for DayOfWeekTO {
    fn from(weekday: Weekday) -> Self {
        match weekday {
            Weekday::Monday => DayOfWeekTO::Monday,
            Weekday::Tuesday => DayOfWeekTO::Tuesday,
            Weekday::Wednesday => DayOfWeekTO::Wednesday,
            Weekday::Thursday => DayOfWeekTO::Thursday,
            Weekday::Friday => DayOfWeekTO::Friday,
            Weekday::Saturday => DayOfWeekTO::Saturday,
            Weekday::Sunday => DayOfWeekTO::Sunday,
        }
    }
}
impl From<DayOfWeekTO> for Weekday {
    fn from(day_of_week: DayOfWeekTO) -> Self {
        match day_of_week {
            DayOfWeekTO::Monday => Weekday::Monday,
            DayOfWeekTO::Tuesday => Weekday::Tuesday,
            DayOfWeekTO::Wednesday => Weekday::Wednesday,
            DayOfWeekTO::Thursday => Weekday::Thursday,
            DayOfWeekTO::Friday => Weekday::Friday,
            DayOfWeekTO::Saturday => Weekday::Saturday,
            DayOfWeekTO::Sunday => Weekday::Sunday,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlotTO {
    #[serde(default)]
    pub id: Uuid,
    pub day_of_week: DayOfWeekTO,
    pub from: time::Time,
    pub to: time::Time,
    pub min_resources: u8,
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
            min_resources: slot.min_resources,
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
            min_resources: slot.min_resources,
            valid_from: slot.valid_from,
            valid_to: slot.valid_to,
            deleted: slot.deleted,
            version: slot.version,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ShortEmployeeReportTO {
    pub sales_person: SalesPersonTO,
    pub balance_hours: f32,
    pub expected_hours: f32,
    pub overall_hours: f32,
}

#[cfg(feature = "service-impl")]
impl From<&service::reporting::ShortEmployeeReport> for ShortEmployeeReportTO {
    fn from(report: &service::reporting::ShortEmployeeReport) -> Self {
        Self {
            sales_person: SalesPersonTO::from(report.sales_person.as_ref()),
            balance_hours: report.balance_hours,
            expected_hours: report.expected_hours,
            overall_hours: report.overall_hours,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ReportingCustomExtraHoursTO {
    pub id: Uuid,
    pub name: String,
    pub hours: f32,
}

#[cfg(feature = "service-impl")]
impl From<&service::reporting::CustomExtraHours> for ReportingCustomExtraHoursTO {
    fn from(custom_hours: &service::reporting::CustomExtraHours) -> Self {
        Self {
            id: custom_hours.id,
            name: custom_hours.name.clone(),
            hours: custom_hours.hours,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub enum ExtraHoursReportCategoryTO {
    Shiftplan,
    ExtraWork,
    Vacation,
    SickLeave,
    Holiday,
    Unavailable,
    Custom(Uuid),
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
            service::reporting::ExtraHoursReportCategory::Unavailable => Self::Unavailable,
            service::reporting::ExtraHoursReportCategory::Custom(lazy) => Self::Custom(*lazy.key()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
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

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct WorkingHoursReportTO {
    pub from: time::Date,
    pub to: time::Date,
    pub expected_hours: f32,
    pub overall_hours: f32,
    pub balance: f32,

    pub days_per_week: u8,
    pub workdays_per_week: f32,

    pub shiftplan_hours: f32,
    pub extra_work_hours: f32,
    pub vacation_hours: f32,
    pub vacation_days: f32,
    pub sick_leave_hours: f32,
    pub sick_leave_days: f32,
    pub holiday_hours: f32,
    pub holiday_days: f32,
    pub absence_days: f32,

    pub custom_extra_hours: Arc<[ReportingCustomExtraHoursTO]>,

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
            custom_extra_hours: hours
                .custom_extra_hours
                .iter()
                .map(ReportingCustomExtraHoursTO::from)
                .collect(),
            days: hours.days.iter().map(WorkingHoursDayTO::from).collect(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
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

    pub vacation_carryover: i32,
    pub vacation_days: f32,
    pub vacation_entitlement: f32,
    pub sick_leave_days: f32,
    pub holiday_days: f32,
    pub absence_days: f32,

    pub carryover_hours: f32,

    pub custom_extra_hours: Arc<[ReportingCustomExtraHoursTO]>,

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
            holiday_hours: report.holiday_hours,
            vacation_carryover: report.vacation_carryover,
            vacation_days: report.vacation_days,
            vacation_entitlement: report.vacation_entitlement,
            sick_leave_days: report.sick_leave_days,
            holiday_days: report.holiday_days,
            absence_days: report.absence_days,
            carryover_hours: report.carryover_hours,
            custom_extra_hours: report
                .custom_extra_hours
                .iter()
                .map(ReportingCustomExtraHoursTO::from)
                .collect(),
            by_week: report
                .by_week
                .iter()
                .map(WorkingHoursReportTO::from)
                .collect(),
            by_month: report
                .by_month
                .iter()
                .map(WorkingHoursReportTO::from)
                .collect(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmployeeWorkDetailsTO {
    #[serde(default)]
    pub id: Uuid,
    pub sales_person_id: Uuid,
    pub expected_hours: f32,
    pub from_day_of_week: DayOfWeekTO,
    pub from_calendar_week: u8,
    pub from_year: u32,
    pub to_day_of_week: DayOfWeekTO,
    pub to_calendar_week: u8,
    pub to_year: u32,
    pub workdays_per_week: u8,

    pub monday: bool,
    pub tuesday: bool,
    pub wednesday: bool,
    pub thursday: bool,
    pub friday: bool,
    pub saturday: bool,
    pub sunday: bool,

    pub vacation_days: u8,

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
impl From<&service::employee_work_details::EmployeeWorkDetails> for EmployeeWorkDetailsTO {
    fn from(working_hours: &service::employee_work_details::EmployeeWorkDetails) -> Self {
        Self {
            id: working_hours.id,
            sales_person_id: working_hours.sales_person_id,
            expected_hours: working_hours.expected_hours,
            from_day_of_week: working_hours.from_day_of_week.into(),
            from_calendar_week: working_hours.from_calendar_week,
            from_year: working_hours.from_year,
            to_day_of_week: working_hours.to_day_of_week.into(),
            to_calendar_week: working_hours.to_calendar_week,
            to_year: working_hours.to_year,
            workdays_per_week: working_hours.workdays_per_week,

            monday: working_hours.monday,
            tuesday: working_hours.tuesday,
            wednesday: working_hours.wednesday,
            thursday: working_hours.thursday,
            friday: working_hours.friday,
            saturday: working_hours.saturday,
            sunday: working_hours.sunday,

            vacation_days: working_hours.vacation_days,

            days_per_week: working_hours.potential_days_per_week(),
            hours_per_day: working_hours.hours_per_day(),
            hours_per_holiday: working_hours.holiday_hours(),

            created: working_hours.created,
            deleted: working_hours.deleted,
            version: working_hours.version,
        }
    }
}

#[cfg(feature = "service-impl")]
impl From<&EmployeeWorkDetailsTO> for service::employee_work_details::EmployeeWorkDetails {
    fn from(working_hours: &EmployeeWorkDetailsTO) -> Self {
        Self {
            id: working_hours.id,
            sales_person_id: working_hours.sales_person_id,
            expected_hours: working_hours.expected_hours,
            from_day_of_week: working_hours.from_day_of_week.into(),
            from_calendar_week: working_hours.from_calendar_week,
            from_year: working_hours.from_year,
            to_day_of_week: working_hours.to_day_of_week.into(),
            to_calendar_week: working_hours.to_calendar_week,
            to_year: working_hours.to_year,
            workdays_per_week: working_hours.workdays_per_week,

            monday: working_hours.monday,
            tuesday: working_hours.tuesday,
            wednesday: working_hours.wednesday,
            thursday: working_hours.thursday,
            friday: working_hours.friday,
            saturday: working_hours.saturday,
            sunday: working_hours.sunday,

            vacation_days: working_hours.vacation_days,

            created: working_hours.created,
            deleted: working_hours.deleted,
            version: working_hours.version,
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, ToSchema)]
pub enum ExtraHoursCategoryTO {
    ExtraWork,
    Vacation,
    SickLeave,
    Holiday,
    Unavailable,
    Custom(Uuid),
}
#[cfg(feature = "service-impl")]
impl From<&service::extra_hours::ExtraHoursCategory> for ExtraHoursCategoryTO {
    fn from(category: &service::extra_hours::ExtraHoursCategory) -> Self {
        match category {
            service::extra_hours::ExtraHoursCategory::ExtraWork => Self::ExtraWork,
            service::extra_hours::ExtraHoursCategory::Vacation => Self::Vacation,
            service::extra_hours::ExtraHoursCategory::SickLeave => Self::SickLeave,
            service::extra_hours::ExtraHoursCategory::Holiday => Self::Holiday,
            service::extra_hours::ExtraHoursCategory::Unavailable => Self::Unavailable,
            service::extra_hours::ExtraHoursCategory::CustomExtraHours(lazy) => {
                Self::Custom(*lazy.key())
            }
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
            ExtraHoursCategoryTO::Unavailable => Self::Unavailable,
            ExtraHoursCategoryTO::Custom(id) => Self::CustomExtraHours(LazyLoad::new(*id)),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
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

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct SalesPersonUnavailableTO {
    #[serde(default)]
    pub id: Uuid,
    pub sales_person_id: Uuid,
    pub year: u32,
    pub calendar_week: u8,
    pub day_of_week: DayOfWeekTO,
    #[serde(default)]
    pub created: Option<time::PrimitiveDateTime>,
    #[serde(default)]
    pub deleted: Option<time::PrimitiveDateTime>,
    #[serde(rename = "$version")]
    #[serde(default)]
    pub version: Uuid,
}
#[cfg(feature = "service-impl")]
impl From<&service::sales_person_unavailable::SalesPersonUnavailable> for SalesPersonUnavailableTO {
    fn from(unavailable: &service::sales_person_unavailable::SalesPersonUnavailable) -> Self {
        Self {
            id: unavailable.id,
            sales_person_id: unavailable.sales_person_id,
            year: unavailable.year,
            calendar_week: unavailable.calendar_week,
            day_of_week: unavailable.day_of_week.into(),
            created: unavailable.created,
            deleted: unavailable.deleted,
            version: unavailable.version,
        }
    }
}
#[cfg(feature = "service-impl")]
impl From<&SalesPersonUnavailableTO> for service::sales_person_unavailable::SalesPersonUnavailable {
    fn from(unavailable: &SalesPersonUnavailableTO) -> Self {
        Self {
            id: unavailable.id,
            sales_person_id: unavailable.sales_person_id,
            year: unavailable.year,
            calendar_week: unavailable.calendar_week,
            day_of_week: unavailable.day_of_week.into(),
            created: unavailable.created,
            deleted: unavailable.deleted,
            version: unavailable.version,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BookingConflictTO {
    pub booking: BookingTO,
    pub slot: Arc<SlotTO>,
    pub sales_person: Arc<SalesPersonTO>,
}

#[cfg(feature = "service-impl")]
impl From<&BookingInformation> for BookingConflictTO {
    fn from(booking_conflict: &BookingInformation) -> BookingConflictTO {
        BookingConflictTO {
            booking: (&booking_conflict.booking).into(),
            slot: Arc::new(SlotTO::from(&*booking_conflict.slot)),
            sales_person: Arc::new(SalesPersonTO::from(&*booking_conflict.sales_person)),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkingHoursPerSalesPersonTO {
    pub sales_person_id: Uuid,
    pub sales_person_name: Arc<str>,
    pub available_hours: f32,
}
#[cfg(feature = "service-impl")]
impl From<&WorkingHoursPerSalesPerson> for WorkingHoursPerSalesPersonTO {
    fn from(working_hours_per_sales_person: &WorkingHoursPerSalesPerson) -> Self {
        Self {
            sales_person_id: working_hours_per_sales_person.sales_person_id,
            sales_person_name: working_hours_per_sales_person.sales_person_name.clone(),
            available_hours: working_hours_per_sales_person.available_hours,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WeeklySummaryTO {
    pub year: u32,
    pub week: u8,
    pub overall_available_hours: f32,
    pub required_hours: f32,
    pub monday_available_hours: f32,
    pub tuesday_available_hours: f32,
    pub wednesday_available_hours: f32,
    pub thursday_available_hours: f32,
    pub friday_available_hours: f32,
    pub saturday_available_hours: f32,
    pub sunday_available_hours: f32,
    pub working_hours_per_sales_person: Arc<[WorkingHoursPerSalesPersonTO]>,
}
#[cfg(feature = "service-impl")]
impl From<&WeeklySummary> for WeeklySummaryTO {
    fn from(weekly_summary: &WeeklySummary) -> Self {
        Self {
            year: weekly_summary.year,
            week: weekly_summary.week,
            overall_available_hours: weekly_summary.overall_available_hours,
            required_hours: weekly_summary.required_hours,
            monday_available_hours: weekly_summary.monday_available_hours,
            tuesday_available_hours: weekly_summary.tuesday_available_hours,
            wednesday_available_hours: weekly_summary.wednesday_available_hours,
            thursday_available_hours: weekly_summary.thursday_available_hours,
            friday_available_hours: weekly_summary.friday_available_hours,
            saturday_available_hours: weekly_summary.saturday_available_hours,
            sunday_available_hours: weekly_summary.sunday_available_hours,
            working_hours_per_sales_person: weekly_summary
                .working_hours_per_sales_person
                .iter()
                .map(|working_hours_per_sales_person| {
                    WorkingHoursPerSalesPersonTO::from(working_hours_per_sales_person)
                })
                .collect::<Vec<_>>()
                .into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum SpecialDayTypeTO {
    Holiday,
    ShortDay,
}
#[cfg(feature = "service-impl")]
impl From<&service::special_days::SpecialDayType> for SpecialDayTypeTO {
    fn from(day_type: &service::special_days::SpecialDayType) -> Self {
        match day_type {
            service::special_days::SpecialDayType::Holiday => Self::Holiday,
            service::special_days::SpecialDayType::ShortDay => Self::ShortDay,
        }
    }
}
#[cfg(feature = "service-impl")]
impl From<&SpecialDayTypeTO> for service::special_days::SpecialDayType {
    fn from(day_type: &SpecialDayTypeTO) -> Self {
        match day_type {
            SpecialDayTypeTO::Holiday => Self::Holiday,
            SpecialDayTypeTO::ShortDay => Self::ShortDay,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShiftplanBookingTO {
    pub booking: BookingTO,
    pub sales_person: Arc<SalesPersonTO>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShiftplanSlotTO {
    pub slot: SlotTO,
    pub bookings: Vec<ShiftplanBookingTO>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShiftplanDayTO {
    pub day_of_week: DayOfWeekTO,
    pub slots: Vec<ShiftplanSlotTO>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShiftplanWeekTO {
    pub year: u32,
    pub calendar_week: u8,
    pub days: Vec<ShiftplanDayTO>,
}

#[cfg(feature = "service-impl")]
impl From<&service::shiftplan::ShiftplanBooking> for ShiftplanBookingTO {
    fn from(booking: &service::shiftplan::ShiftplanBooking) -> Self {
        Self {
            booking: (&booking.booking).into(),
            sales_person: Arc::new((&booking.sales_person).into()),
        }
    }
}

#[cfg(feature = "service-impl")]
impl From<&service::shiftplan::ShiftplanSlot> for ShiftplanSlotTO {
    fn from(slot: &service::shiftplan::ShiftplanSlot) -> Self {
        Self {
            slot: (&slot.slot).into(),
            bookings: slot.bookings.iter().map(Into::into).collect(),
        }
    }
}

#[cfg(feature = "service-impl")]
impl From<&service::shiftplan::ShiftplanDay> for ShiftplanDayTO {
    fn from(day: &service::shiftplan::ShiftplanDay) -> Self {
        Self {
            day_of_week: day.day_of_week.into(),
            slots: day.slots.iter().map(Into::into).collect(),
        }
    }
}

#[cfg(feature = "service-impl")]
impl From<&service::shiftplan::ShiftplanWeek> for ShiftplanWeekTO {
    fn from(week: &service::shiftplan::ShiftplanWeek) -> Self {
        Self {
            year: week.year,
            calendar_week: week.calendar_week,
            days: week.days.iter().map(Into::into).collect(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct SpecialDayTO {
    #[serde(default)]
    pub id: Uuid,
    pub year: u32,
    pub calendar_week: u8,
    pub day_of_week: DayOfWeekTO,
    pub day_type: SpecialDayTypeTO,
    #[schema(value_type = Option<String>, format = "time")]
    pub time_of_day: Option<time::Time>,
    #[serde(default)]
    #[schema(value_type = Option<String>, format = "date-time")]
    pub created: Option<time::PrimitiveDateTime>,
    #[serde(default)]
    #[schema(value_type = Option<String>, format = "date-time")]
    pub deleted: Option<time::PrimitiveDateTime>,
    #[serde(rename = "$version")]
    #[serde(default)]
    pub version: Uuid,
}
#[cfg(feature = "service-impl")]
impl From<&service::special_days::SpecialDay> for SpecialDayTO {
    fn from(special_day: &service::special_days::SpecialDay) -> Self {
        Self {
            id: special_day.id,
            year: special_day.year,
            calendar_week: special_day.calendar_week,
            day_of_week: special_day.day_of_week.into(),
            day_type: (&special_day.day_type).into(),
            time_of_day: special_day.time_of_day,
            created: special_day.created,
            deleted: special_day.deleted,
            version: special_day.version,
        }
    }
}
#[cfg(feature = "service-impl")]
impl From<&SpecialDayTO> for service::special_days::SpecialDay {
    fn from(special_day: &SpecialDayTO) -> Self {
        Self {
            id: special_day.id,
            year: special_day.year,
            calendar_week: special_day.calendar_week,
            day_of_week: special_day.day_of_week.into(),
            day_type: (&special_day.day_type).into(),
            time_of_day: special_day.time_of_day,
            created: special_day.created,
            deleted: special_day.deleted,
            version: special_day.version,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VacationPayloadTO {
    pub sales_person_id: Uuid,
    pub from: time::Date,
    pub to: time::Date,
    pub description: Arc<str>,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct CustomExtraHoursTO {
    #[serde(default)]
    pub id: Uuid,
    pub name: Arc<str>,
    pub description: Option<Arc<str>>,
    pub modifies_balance: bool,
    pub assigned_sales_person_ids: Arc<[Uuid]>,
    #[serde(default)]
    pub created: Option<PrimitiveDateTime>,
    #[serde(default)]
    pub deleted: Option<PrimitiveDateTime>,
    #[serde(rename = "$version")]
    #[serde(default)]
    pub version: Uuid,
}

#[cfg(feature = "service-impl")]
impl From<&service::custom_extra_hours::CustomExtraHours> for CustomExtraHoursTO {
    fn from(entity: &service::custom_extra_hours::CustomExtraHours) -> Self {
        Self {
            id: entity.id,
            name: entity.name.clone(),
            description: entity.description.clone(),
            modifies_balance: entity.modifies_balance,
            assigned_sales_person_ids: entity.assigned_sales_person_ids.clone(),
            created: entity.created,
            deleted: entity.deleted,
            version: entity.version,
        }
    }
}
#[cfg(feature = "service-impl")]
derive_from_reference!(
    service::custom_extra_hours::CustomExtraHours,
    CustomExtraHoursTO
);

#[cfg(feature = "service-impl")]
impl From<&CustomExtraHoursTO> for service::custom_extra_hours::CustomExtraHours {
    fn from(entity: &CustomExtraHoursTO) -> Self {
        Self {
            id: entity.id,
            name: entity.name.clone(),
            description: entity.description.clone(),
            modifies_balance: entity.modifies_balance,
            assigned_sales_person_ids: entity.assigned_sales_person_ids.clone(),
            created: entity.created,
            deleted: entity.deleted,
            version: entity.version,
        }
    }
}
#[cfg(feature = "service-impl")]
derive_from_reference!(
    CustomExtraHoursTO,
    service::custom_extra_hours::CustomExtraHours
);

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct WeekMessageTO {
    #[serde(default)]
    pub id: Uuid,
    pub year: u32,
    pub calendar_week: u8,
    pub message: Arc<str>,
    #[serde(default)]
    pub created: Option<PrimitiveDateTime>,
    #[serde(default)]
    pub deleted: Option<PrimitiveDateTime>,
    #[serde(rename = "$version")]
    #[serde(default)]
    pub version: Uuid,
}

#[cfg(feature = "service-impl")]
impl From<&service::week_message::WeekMessage> for WeekMessageTO {
    fn from(entity: &service::week_message::WeekMessage) -> Self {
        Self {
            id: entity.id,
            year: entity.year,
            calendar_week: entity.calendar_week,
            message: entity.message.clone(),
            created: entity.created,
            deleted: entity.deleted,
            version: entity.version,
        }
    }
}

#[cfg(feature = "service-impl")]
impl From<&WeekMessageTO> for service::week_message::WeekMessage {
    fn from(entity: &WeekMessageTO) -> Self {
        Self {
            id: entity.id,
            year: entity.year,
            calendar_week: entity.calendar_week,
            message: entity.message.clone(),
            created: entity.created,
            deleted: entity.deleted,
            version: entity.version,
        }
    }
}
