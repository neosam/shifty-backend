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

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct ShiftplanTO {
    #[serde(default)]
    pub id: Uuid,
    pub name: Arc<str>,
    #[serde(default)]
    pub is_planning: bool,
    #[serde(default)]
    pub deleted: Option<PrimitiveDateTime>,
    #[serde(rename = "$version")]
    #[serde(default)]
    pub version: Uuid,
}
#[cfg(feature = "service-impl")]
impl From<&service::shiftplan_catalog::Shiftplan> for ShiftplanTO {
    fn from(shiftplan: &service::shiftplan_catalog::Shiftplan) -> Self {
        Self {
            id: shiftplan.id,
            name: shiftplan.name.clone(),
            is_planning: shiftplan.is_planning,
            deleted: shiftplan.deleted,
            version: shiftplan.version,
        }
    }
}
#[cfg(feature = "service-impl")]
impl From<&ShiftplanTO> for service::shiftplan_catalog::Shiftplan {
    fn from(to: &ShiftplanTO) -> Self {
        Self {
            id: to.id,
            name: to.name.clone(),
            is_planning: to.is_planning,
            deleted: to.deleted,
            version: to.version,
        }
    }
}

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

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
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
pub struct BookingLogTO {
    pub year: u32,
    pub calendar_week: u8,
    pub day_of_week: DayOfWeekTO,
    pub name: Arc<str>,
    #[schema(value_type = String, format = "time")]
    pub time_from: time::Time,
    #[schema(value_type = String, format = "time")]
    pub time_to: time::Time,
    #[schema(value_type = String, format = "date-time")]
    pub created: PrimitiveDateTime,
    #[serde(default)]
    #[schema(value_type = Option<String>, format = "date-time")]
    pub deleted: Option<PrimitiveDateTime>,
    pub created_by: Arc<str>,
    #[serde(default)]
    pub deleted_by: Option<Arc<str>>,
}

#[cfg(feature = "service-impl")]
impl From<&service::booking_log::BookingLog> for BookingLogTO {
    fn from(log: &service::booking_log::BookingLog) -> Self {
        Self {
            year: log.year,
            calendar_week: log.calendar_week,
            day_of_week: log.day_of_week.into(),
            name: log.name.clone(),
            time_from: log.time_from,
            time_to: log.time_to,
            created: log.created,
            deleted: log.deleted,
            created_by: log.created_by.clone(),
            deleted_by: log.deleted_by.clone(),
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
impl From<shifty_utils::DayOfWeek> for DayOfWeekTO {
    fn from(day_of_week: shifty_utils::DayOfWeek) -> Self {
        match day_of_week {
            shifty_utils::DayOfWeek::Monday => Self::Monday,
            shifty_utils::DayOfWeek::Tuesday => Self::Tuesday,
            shifty_utils::DayOfWeek::Wednesday => Self::Wednesday,
            shifty_utils::DayOfWeek::Thursday => Self::Thursday,
            shifty_utils::DayOfWeek::Friday => Self::Friday,
            shifty_utils::DayOfWeek::Saturday => Self::Saturday,
            shifty_utils::DayOfWeek::Sunday => Self::Sunday,
        }
    }
}
#[cfg(feature = "service-impl")]
impl From<DayOfWeekTO> for shifty_utils::DayOfWeek {
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct SlotTO {
    #[serde(default)]
    pub id: Uuid,
    pub day_of_week: DayOfWeekTO,
    #[schema(value_type = String, format = "time")]
    pub from: time::Time,
    #[schema(value_type = String, format = "time")]
    pub to: time::Time,
    pub min_resources: u8,
    /// Phase 5 (D-10): optionales Limit für die Anzahl bezahlter
    /// Mitarbeiter:innen pro Slot/Woche. `None` = kein Limit (D-15).
    /// `#[serde(default)]` hält Backward-Compat für API-Konsumenten,
    /// die das Feld weglassen — sie erhalten implizit „kein Limit".
    #[serde(default)]
    pub max_paid_employees: Option<u8>,
    pub valid_from: time::Date,
    pub valid_to: Option<time::Date>,
    #[serde(default)]
    pub deleted: Option<time::PrimitiveDateTime>,
    #[serde(rename = "$version")]
    #[serde(default)]
    pub version: Uuid,
    #[serde(default)]
    pub shiftplan_id: Option<Uuid>,
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
            max_paid_employees: slot.max_paid_employees,
            valid_from: slot.valid_from,
            valid_to: slot.valid_to,
            deleted: slot.deleted,
            version: slot.version,
            shiftplan_id: slot.shiftplan_id,
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
            max_paid_employees: slot.max_paid_employees,
            valid_from: slot.valid_from,
            valid_to: slot.valid_to,
            deleted: slot.deleted,
            version: slot.version,
            shiftplan_id: slot.shiftplan_id,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ShortEmployeeReportTO {
    pub sales_person: SalesPersonTO,
    pub balance_hours: f32,
    pub expected_hours: f32,
    pub dynamic_hours: f32,
    pub overall_hours: f32,
    #[serde(default)]
    pub volunteer_hours: f32,
}

#[cfg(feature = "service-impl")]
impl From<&service::reporting::ShortEmployeeReport> for ShortEmployeeReportTO {
    fn from(report: &service::reporting::ShortEmployeeReport) -> Self {
        Self {
            sales_person: SalesPersonTO::from(report.sales_person.as_ref()),
            balance_hours: report.balance_hours,
            expected_hours: report.expected_hours,
            dynamic_hours: report.dynamic_hours,
            overall_hours: report.overall_hours,
            volunteer_hours: report.volunteer_hours,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ReportingCustomExtraHoursTO {
    pub id: Uuid,
    pub name: Arc<str>,
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
    UnpaidLeave,
    VolunteerWork,
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
            service::reporting::ExtraHoursReportCategory::UnpaidLeave => Self::UnpaidLeave,
            service::reporting::ExtraHoursReportCategory::VolunteerWork => Self::VolunteerWork,
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
    pub dynamic_hours: f32,
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
    pub unpaid_leave_hours: f32,
    pub absence_days: f32,
    #[serde(default)]
    pub volunteer_hours: f32,

    pub custom_extra_hours: Arc<[ReportingCustomExtraHoursTO]>,

    pub days: Arc<[WorkingHoursDayTO]>,
}

#[cfg(feature = "service-impl")]
impl From<&service::reporting::GroupedReportHours> for WorkingHoursReportTO {
    fn from(hours: &service::reporting::GroupedReportHours) -> Self {
        Self {
            from: hours.from.to_date(),
            to: hours.to.to_date(),
            expected_hours: hours.expected_hours,
            dynamic_hours: hours.dynamic_hours,
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
            unpaid_leave_hours: hours.unpaid_leave_hours,
            absence_days: hours.absence_days(),
            volunteer_hours: hours.volunteer_hours,
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
    pub dynamic_hours: f32,

    pub shiftplan_hours: f32,
    pub extra_work_hours: f32,
    pub vacation_hours: f32,
    pub sick_leave_hours: f32,
    pub holiday_hours: f32,
    pub unpaid_leave_hours: f32,
    #[serde(default)]
    pub volunteer_hours: f32,

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
            dynamic_hours: report.dynamic_hours,
            shiftplan_hours: report.shiftplan_hours,
            extra_work_hours: report.extra_work_hours,
            vacation_hours: report.vacation_hours,
            sick_leave_hours: report.sick_leave_hours,
            holiday_hours: report.holiday_hours,
            unpaid_leave_hours: report.unpaid_leave_hours,
            volunteer_hours: report.volunteer_hours,
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
    pub is_dynamic: bool,
    #[serde(default)]
    pub cap_planned_hours_to_expected: bool,

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
            is_dynamic: working_hours.is_dynamic,
            cap_planned_hours_to_expected: working_hours.cap_planned_hours_to_expected,

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
            is_dynamic: working_hours.is_dynamic,
            cap_planned_hours_to_expected: working_hours.cap_planned_hours_to_expected,

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
    UnpaidLeave,
    VolunteerWork,
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
            service::extra_hours::ExtraHoursCategory::UnpaidLeave => Self::UnpaidLeave,
            service::extra_hours::ExtraHoursCategory::VolunteerWork => Self::VolunteerWork,
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
            ExtraHoursCategoryTO::UnpaidLeave => Self::UnpaidLeave,
            ExtraHoursCategoryTO::VolunteerWork => Self::VolunteerWork,
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

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct WorkingHoursPerSalesPersonTO {
    pub sales_person_id: Uuid,
    pub sales_person_name: Arc<str>,
    pub available_hours: f32,
    pub absence_hours: f32,
    pub vacation_hours: f32,
    pub sick_leave_hours: f32,
    pub holiday_hours: f32,
    pub unavailable_hours: f32,
    pub custom_absence_hours: Arc<[ReportingCustomExtraHoursTO]>,
}
#[cfg(feature = "service-impl")]
impl From<&WorkingHoursPerSalesPerson> for WorkingHoursPerSalesPersonTO {
    fn from(working_hours_per_sales_person: &WorkingHoursPerSalesPerson) -> Self {
        Self {
            sales_person_id: working_hours_per_sales_person.sales_person_id,
            sales_person_name: working_hours_per_sales_person.sales_person_name.clone(),
            available_hours: working_hours_per_sales_person.available_hours,
            absence_hours: working_hours_per_sales_person.absence_hours,
            vacation_hours: working_hours_per_sales_person.vacation_hours,
            sick_leave_hours: working_hours_per_sales_person.sick_leave_hours,
            holiday_hours: working_hours_per_sales_person.holiday_hours,
            unavailable_hours: working_hours_per_sales_person.unavailable_hours,
            custom_absence_hours: working_hours_per_sales_person
                .custom_absence_hours
                .iter()
                .map(ReportingCustomExtraHoursTO::from)
                .collect::<Vec<_>>()
                .into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WeeklySummaryTO {
    pub year: u32,
    pub week: u8,
    pub overall_available_hours: f32,
    pub required_hours: f32,
    pub paid_hours: f32,
    pub volunteer_hours: f32,
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
            paid_hours: weekly_summary.paid_hours,
            volunteer_hours: weekly_summary.volunteer_hours,
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

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ShiftplanBookingTO {
    pub booking: BookingTO,
    pub sales_person: Arc<SalesPersonTO>,
    pub self_added: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ShiftplanSlotTO {
    pub slot: SlotTO,
    pub bookings: Vec<ShiftplanBookingTO>,
    /// Phase 5 (D-09): Wire-Mirror von
    /// `service::shiftplan::ShiftplanSlot.current_paid_count`. Live-Count
    /// der Bookings im Slot (für die View-Woche), deren Sales Person
    /// `is_paid == true` hat (D-04). Soft-deleted Bookings/Sales Persons
    /// sind upstream gefiltert; Absence-Status zählt nicht (D-05). Immer
    /// populated, unabhängig von `slot.max_paid_employees`.
    pub current_paid_count: u8,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ShiftplanDayTO {
    pub day_of_week: DayOfWeekTO,
    pub slots: Vec<ShiftplanSlotTO>,
    /// Phase-3-Marker — gesetzt nur durch die per-sales-person-Sicht
    /// (`get_shiftplan_*_for_sales_person`). Globale Schichtplan-Endpunkte
    /// lassen das Feld immer `None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unavailable: Option<UnavailabilityMarkerTO>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
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
            self_added: booking.self_added,
        }
    }
}

#[cfg(feature = "service-impl")]
impl From<&service::shiftplan::ShiftplanSlot> for ShiftplanSlotTO {
    fn from(slot: &service::shiftplan::ShiftplanSlot) -> Self {
        Self {
            slot: (&slot.slot).into(),
            bookings: slot.bookings.iter().map(Into::into).collect(),
            current_paid_count: slot.current_paid_count,
        }
    }
}

#[cfg(feature = "service-impl")]
impl From<&service::shiftplan::ShiftplanDay> for ShiftplanDayTO {
    fn from(day: &service::shiftplan::ShiftplanDay) -> Self {
        Self {
            day_of_week: day.day_of_week.into(),
            slots: day.slots.iter().map(Into::into).collect(),
            unavailable: day.unavailable.as_ref().map(UnavailabilityMarkerTO::from),
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

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PlanDayViewTO {
    pub shiftplan: ShiftplanTO,
    pub slots: Vec<ShiftplanSlotTO>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ShiftplanDayAggregateTO {
    pub year: u32,
    pub calendar_week: u8,
    pub day_of_week: DayOfWeekTO,
    pub plans: Vec<PlanDayViewTO>,
}

#[cfg(feature = "service-impl")]
impl From<&service::shiftplan::PlanDayView> for PlanDayViewTO {
    fn from(plan: &service::shiftplan::PlanDayView) -> Self {
        Self {
            shiftplan: (&plan.shiftplan).into(),
            slots: plan.slots.iter().map(Into::into).collect(),
        }
    }
}

#[cfg(feature = "service-impl")]
impl From<&service::shiftplan::ShiftplanDayAggregate> for ShiftplanDayAggregateTO {
    fn from(agg: &service::shiftplan::ShiftplanDayAggregate) -> Self {
        Self {
            year: agg.year,
            calendar_week: agg.calendar_week,
            day_of_week: agg.day_of_week.into(),
            plans: agg.plans.iter().map(Into::into).collect(),
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

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct BillingPeriodValueTO {
    pub value_delta: f32,
    pub value_ytd_from: f32,
    pub value_ytd_to: f32,
    pub value_full_year: f32,
}

#[cfg(feature = "service-impl")]
impl From<&service::billing_period::BillingPeriodValue> for BillingPeriodValueTO {
    fn from(value: &service::billing_period::BillingPeriodValue) -> Self {
        Self {
            value_delta: value.value_delta,
            value_ytd_from: value.value_ytd_from,
            value_ytd_to: value.value_ytd_to,
            value_full_year: value.value_full_year,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct BillingPeriodSalesPersonTO {
    pub id: Uuid,
    pub sales_person_id: Uuid,
    pub values: std::collections::BTreeMap<String, BillingPeriodValueTO>,
    #[schema(value_type = String, format = "date-time")]
    pub created_at: time::PrimitiveDateTime,
    pub created_by: Arc<str>,
    #[schema(value_type = Option<String>, format = "date-time")]
    pub deleted_at: Option<time::PrimitiveDateTime>,
    pub deleted_by: Option<Arc<str>>,
}

#[cfg(feature = "service-impl")]
impl From<&service::billing_period::BillingPeriodSalesPerson> for BillingPeriodSalesPersonTO {
    fn from(sp: &service::billing_period::BillingPeriodSalesPerson) -> Self {
        let values = sp
            .values
            .iter()
            .map(|(k, v)| (k.as_str().to_string(), BillingPeriodValueTO::from(v)))
            .collect();

        Self {
            id: sp.id,
            sales_person_id: sp.sales_person_id,
            values,
            created_at: sp.created_at,
            created_by: sp.created_by.clone(),
            deleted_at: sp.deleted_at,
            deleted_by: sp.deleted_by.clone(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct BillingPeriodTO {
    pub id: Uuid,
    pub start_date: time::Date,
    pub end_date: time::Date,
    pub snapshot_schema_version: u32,
    pub sales_persons: Arc<[BillingPeriodSalesPersonTO]>,
    #[schema(value_type = String, format = "date-time")]
    pub created_at: time::PrimitiveDateTime,
    pub created_by: Arc<str>,
    #[schema(value_type = Option<String>, format = "date-time")]
    pub deleted_at: Option<time::PrimitiveDateTime>,
    pub deleted_by: Option<Arc<str>>,
}

#[cfg(feature = "service-impl")]
impl From<&service::billing_period::BillingPeriod> for BillingPeriodTO {
    fn from(bp: &service::billing_period::BillingPeriod) -> Self {
        Self {
            id: bp.id,
            start_date: bp.start_date.to_date(),
            end_date: bp.end_date.to_date(),
            snapshot_schema_version: bp.snapshot_schema_version,
            sales_persons: bp
                .sales_persons
                .iter()
                .map(BillingPeriodSalesPersonTO::from)
                .collect(),
            created_at: bp.created_at,
            created_by: bp.created_by.clone(),
            deleted_at: bp.deleted_at,
            deleted_by: bp.deleted_by.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateBillingPeriodRequestTO {
    pub end_date: time::Date,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, ToSchema)]
pub enum TemplateEngineTO {
    #[serde(rename = "tera")]
    #[default]
    Tera,
    #[serde(rename = "minijinja")]
    MiniJinja,
}

#[cfg(feature = "service-impl")]
impl From<&service::text_template::TemplateEngine> for TemplateEngineTO {
    fn from(engine: &service::text_template::TemplateEngine) -> Self {
        match engine {
            service::text_template::TemplateEngine::Tera => Self::Tera,
            service::text_template::TemplateEngine::MiniJinja => Self::MiniJinja,
        }
    }
}

#[cfg(feature = "service-impl")]
impl From<&TemplateEngineTO> for service::text_template::TemplateEngine {
    fn from(engine: &TemplateEngineTO) -> Self {
        match engine {
            TemplateEngineTO::Tera => Self::Tera,
            TemplateEngineTO::MiniJinja => Self::MiniJinja,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct TextTemplateTO {
    #[serde(default)]
    pub id: Uuid,
    #[serde(default)]
    pub name: Option<Arc<str>>,
    pub template_type: Arc<str>,
    pub template_text: Arc<str>,
    #[serde(default)]
    pub template_engine: TemplateEngineTO,
    #[serde(default)]
    pub created_at: Option<time::PrimitiveDateTime>,
    #[serde(default)]
    pub created_by: Option<Arc<str>>,
    #[serde(default)]
    pub deleted: Option<time::PrimitiveDateTime>,
    #[serde(default)]
    pub deleted_by: Option<Arc<str>>,
    #[serde(rename = "$version")]
    #[serde(default)]
    pub version: Uuid,
}

#[cfg(feature = "service-impl")]
impl From<&service::text_template::TextTemplate> for TextTemplateTO {
    fn from(text_template: &service::text_template::TextTemplate) -> Self {
        Self {
            id: text_template.id,
            name: text_template.name.clone(),
            template_type: text_template.template_type.clone(),
            template_text: text_template.template_text.clone(),
            template_engine: TemplateEngineTO::from(&text_template.template_engine),
            created_at: text_template.created_at,
            created_by: text_template.created_by.clone(),
            deleted: text_template.deleted,
            deleted_by: text_template.deleted_by.clone(),
            version: text_template.version,
        }
    }
}

#[cfg(feature = "service-impl")]
impl From<&TextTemplateTO> for service::text_template::TextTemplate {
    fn from(text_template: &TextTemplateTO) -> Self {
        Self {
            id: text_template.id,
            name: text_template.name.clone(),
            template_type: text_template.template_type.clone(),
            template_text: text_template.template_text.clone(),
            template_engine: service::text_template::TemplateEngine::from(&text_template.template_engine),
            created_at: text_template.created_at,
            created_by: text_template.created_by.clone(),
            deleted: text_template.deleted,
            deleted_by: text_template.deleted_by.clone(),
            version: text_template.version,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct CreateTextTemplateRequestTO {
    pub name: Option<Arc<str>>,
    pub template_type: Arc<str>,
    pub template_text: Arc<str>,
    #[serde(default)]
    pub template_engine: TemplateEngineTO,
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct UpdateTextTemplateRequestTO {
    pub name: Option<Arc<str>>,
    pub template_type: Arc<str>,
    pub template_text: Arc<str>,
    #[serde(default)]
    pub template_engine: TemplateEngineTO,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct BlockTO {
    pub year: u32,
    pub week: u8,
    pub sales_person: Option<SalesPersonTO>,
    pub day_of_week: DayOfWeekTO,
    #[schema(value_type = String, format = "time")]
    pub from: time::Time,
    #[schema(value_type = String, format = "time")]
    pub to: time::Time,
    pub bookings: Vec<BookingTO>,
    pub slots: Vec<SlotTO>,
}

#[cfg(feature = "service-impl")]
impl From<&service::block::Block> for BlockTO {
    fn from(block: &service::block::Block) -> Self {
        Self {
            year: block.year,
            week: block.week,
            sales_person: block
                .sales_person
                .as_ref()
                .map(|sp| SalesPersonTO::from(sp.as_ref())),
            day_of_week: block.day_of_week.into(),
            from: block.from,
            to: block.to,
            bookings: block.bookings.iter().map(BookingTO::from).collect(),
            slots: block.slots.iter().map(SlotTO::from).collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct ShiftplanAssignmentTO {
    pub shiftplan_id: Uuid,
    #[serde(default = "default_permission_level")]
    pub permission_level: String,
}

fn default_permission_level() -> String {
    "available".to_string()
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct ToggleTO {
    pub name: Arc<str>,
    pub enabled: bool,
    #[serde(default)]
    pub description: Option<Arc<str>>,
}

#[cfg(feature = "service-impl")]
impl From<&service::toggle::Toggle> for ToggleTO {
    fn from(toggle: &service::toggle::Toggle) -> Self {
        Self {
            name: toggle.name.clone(),
            enabled: toggle.enabled,
            description: toggle.description.clone(),
        }
    }
}

#[cfg(feature = "service-impl")]
impl From<&ToggleTO> for service::toggle::Toggle {
    fn from(toggle: &ToggleTO) -> Self {
        Self {
            name: toggle.name.clone(),
            enabled: toggle.enabled,
            description: toggle.description.clone(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct ToggleGroupTO {
    pub name: Arc<str>,
    #[serde(default)]
    pub description: Option<Arc<str>>,
}

#[cfg(feature = "service-impl")]
impl From<&service::toggle::ToggleGroup> for ToggleGroupTO {
    fn from(group: &service::toggle::ToggleGroup) -> Self {
        Self {
            name: group.name.clone(),
            description: group.description.clone(),
        }
    }
}

#[cfg(feature = "service-impl")]
impl From<&ToggleGroupTO> for service::toggle::ToggleGroup {
    fn from(group: &ToggleGroupTO) -> Self {
        Self {
            name: group.name.clone(),
            description: group.description.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct ImpersonateTO {
    pub impersonating: bool,
    #[serde(default)]
    pub user_id: Option<Arc<str>>,
}

// ─────────────────────────────────────────────────────────────────────────
// AbsencePeriod (Phase 1 — Range-based absence domain)
// ─────────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum AbsenceCategoryTO {
    Vacation,
    SickLeave,
    UnpaidLeave,
}

#[cfg(feature = "service-impl")]
impl From<&service::absence::AbsenceCategory> for AbsenceCategoryTO {
    fn from(c: &service::absence::AbsenceCategory) -> Self {
        match c {
            service::absence::AbsenceCategory::Vacation => Self::Vacation,
            service::absence::AbsenceCategory::SickLeave => Self::SickLeave,
            service::absence::AbsenceCategory::UnpaidLeave => Self::UnpaidLeave,
        }
    }
}
#[cfg(feature = "service-impl")]
impl From<&AbsenceCategoryTO> for service::absence::AbsenceCategory {
    fn from(c: &AbsenceCategoryTO) -> Self {
        match c {
            AbsenceCategoryTO::Vacation => Self::Vacation,
            AbsenceCategoryTO::SickLeave => Self::SickLeave,
            AbsenceCategoryTO::UnpaidLeave => Self::UnpaidLeave,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct AbsencePeriodTO {
    #[serde(default)]
    pub id: Uuid,
    pub sales_person_id: Uuid,
    pub category: AbsenceCategoryTO,
    #[schema(value_type = String, format = "date")]
    pub from_date: time::Date,
    #[schema(value_type = String, format = "date")]
    pub to_date: time::Date,
    #[serde(default)]
    pub description: Arc<str>,
    #[serde(default)]
    pub created: Option<time::PrimitiveDateTime>,
    #[serde(default)]
    pub deleted: Option<time::PrimitiveDateTime>,
    #[serde(rename = "$version")]
    #[serde(default)]
    pub version: Uuid,
}

#[cfg(feature = "service-impl")]
impl From<&service::absence::AbsencePeriod> for AbsencePeriodTO {
    fn from(a: &service::absence::AbsencePeriod) -> Self {
        Self {
            id: a.id,
            sales_person_id: a.sales_person_id,
            category: (&a.category).into(),
            from_date: a.from_date,
            to_date: a.to_date,
            description: a.description.clone(),
            created: a.created,
            deleted: a.deleted,
            version: a.version,
        }
    }
}
#[cfg(feature = "service-impl")]
impl From<&AbsencePeriodTO> for service::absence::AbsencePeriod {
    fn from(a: &AbsencePeriodTO) -> Self {
        Self {
            id: a.id,
            sales_person_id: a.sales_person_id,
            category: (&a.category).into(),
            from_date: a.from_date,
            to_date: a.to_date,
            description: a.description.clone(),
            created: a.created,
            deleted: a.deleted,
            version: a.version,
        }
    }
}

// ──────────────────────────────────────────────────────────────────────
// Phase 3 — Cross-Source-Warning + Wrapper-Result-DTOs
// ──────────────────────────────────────────────────────────────────────
//
// 5 inline DTOs für die Phase-3 REST-Surface:
// * `WarningTO`             — Tag-Enum (5 Varianten), JSON-Form
//                             `{ "kind": ..., "data": { ... } }`.
// * `UnavailabilityMarkerTO`— Tag-Enum (3 Varianten) für die per-sales-
//                             person-Sicht (`ShiftplanDayTO.unavailable`).
// * `BookingCreateResultTO` — Wrapper für `POST /shiftplan-edit/booking`.
// * `CopyWeekResultTO`      — Wrapper für `POST /shiftplan-edit/copy-week`.
// * `AbsencePeriodCreateResultTO` — Wrapper für `POST /absence-period`
//                             und `PATCH /absence-period/{id}`.
//
// Tag-Enums nutzen `#[serde(tag = "kind", content = "data",
// rename_all = "snake_case")]` (utoipa-5-Support für `#[serde(tag,
// content)]`, RESEARCH.md Pattern 4).

/// Cross-Source-Konflikt-Warning für REST. Eine Warning pro betroffenem
/// Booking-Tag (D-Phase3-15: KEINE De-Dup zwischen Quellen).
///
/// JSON-Form: `{ "kind": "booking_on_absence_day", "data": { ... } }`.
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum WarningTO {
    /// Beim Anlegen eines Bookings auf einem Tag, der durch eine
    /// AbsencePeriod abgedeckt ist (Reverse-Warning, BOOK-02).
    BookingOnAbsenceDay {
        booking_id: Uuid,
        #[schema(value_type = String, format = "date")]
        date: time::Date,
        absence_id: Uuid,
        category: AbsenceCategoryTO,
    },
    /// Beim Anlegen eines Bookings auf einem Tag, der durch
    /// `sales_person_unavailable` abgedeckt ist (Reverse-Warning,
    /// BOOK-02).
    BookingOnUnavailableDay {
        booking_id: Uuid,
        year: u32,
        week: u8,
        day_of_week: DayOfWeekTO,
    },
    /// Beim Anlegen einer AbsencePeriod, die ein bestehendes Booking
    /// überlappt (Forward-Warning, BOOK-01).
    AbsenceOverlapsBooking {
        absence_id: Uuid,
        booking_id: Uuid,
        #[schema(value_type = String, format = "date")]
        date: time::Date,
    },
    /// Beim Anlegen einer AbsencePeriod, die einen bestehenden manuellen
    /// `sales_person_unavailable`-Eintrag überdeckt (Forward-Warning,
    /// BOOK-01, D-Phase3-16: KEIN Auto-Cleanup).
    AbsenceOverlapsManualUnavailable {
        absence_id: Uuid,
        unavailable_id: Uuid,
    },
    /// Phase 5 (D-08): Wire-Mirror von `service::warning::Warning::PaidEmployeeLimitExceeded`.
    /// Emittiert wenn der Live-Count an bezahlten Mitarbeiter:innen in
    /// (year, week, slot) das konfigurierte `max_paid_employees`-Limit
    /// strikt übersteigt (`current > max`, D-06). Buchung wird trotzdem
    /// persistiert (D-07). JSON-Tag (auto via `rename_all = "snake_case"`):
    /// `paid_employee_limit_exceeded`.
    PaidEmployeeLimitExceeded {
        slot_id: Uuid,
        booking_id: Uuid,
        year: u32,
        week: u8,
        current_paid_count: u8,
        max_paid_employees: u8,
    },
}

#[cfg(feature = "service-impl")]
impl From<&service::warning::Warning> for WarningTO {
    fn from(w: &service::warning::Warning) -> Self {
        match w {
            service::warning::Warning::BookingOnAbsenceDay {
                booking_id,
                date,
                absence_id,
                category,
            } => Self::BookingOnAbsenceDay {
                booking_id: *booking_id,
                date: *date,
                absence_id: *absence_id,
                category: category.into(),
            },
            service::warning::Warning::BookingOnUnavailableDay {
                booking_id,
                year,
                week,
                day_of_week,
            } => Self::BookingOnUnavailableDay {
                booking_id: *booking_id,
                year: *year,
                week: *week,
                day_of_week: (*day_of_week).into(),
            },
            service::warning::Warning::AbsenceOverlapsBooking {
                absence_id,
                booking_id,
                date,
            } => Self::AbsenceOverlapsBooking {
                absence_id: *absence_id,
                booking_id: *booking_id,
                date: *date,
            },
            service::warning::Warning::AbsenceOverlapsManualUnavailable {
                absence_id,
                unavailable_id,
            } => Self::AbsenceOverlapsManualUnavailable {
                absence_id: *absence_id,
                unavailable_id: *unavailable_id,
            },
            service::warning::Warning::PaidEmployeeLimitExceeded {
                slot_id,
                booking_id,
                year,
                week,
                current_paid_count,
                max_paid_employees,
            } => Self::PaidEmployeeLimitExceeded {
                slot_id: *slot_id,
                booking_id: *booking_id,
                year: *year,
                week: *week,
                current_paid_count: *current_paid_count,
                max_paid_employees: *max_paid_employees,
            },
        }
    }
}

/// Per-Tag-Marker für die per-sales-person-Sicht (D-Phase3-10). Tag-Enum
/// analog [`WarningTO`].
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum UnavailabilityMarkerTO {
    AbsencePeriod {
        absence_id: Uuid,
        category: AbsenceCategoryTO,
    },
    ManualUnavailable,
    /// Doppel-Quelle: AbsencePeriod UND ManualUnavailable am selben Tag.
    /// `absence_id`/`category` der AbsencePeriod werden mitgeführt
    /// (semantisch reicher).
    Both {
        absence_id: Uuid,
        category: AbsenceCategoryTO,
    },
}

#[cfg(feature = "service-impl")]
impl From<&service::shiftplan::UnavailabilityMarker> for UnavailabilityMarkerTO {
    fn from(m: &service::shiftplan::UnavailabilityMarker) -> Self {
        match m {
            service::shiftplan::UnavailabilityMarker::AbsencePeriod {
                absence_id,
                category,
            } => Self::AbsencePeriod {
                absence_id: *absence_id,
                category: category.into(),
            },
            service::shiftplan::UnavailabilityMarker::ManualUnavailable => Self::ManualUnavailable,
            service::shiftplan::UnavailabilityMarker::Both {
                absence_id,
                category,
            } => Self::Both {
                absence_id: *absence_id,
                category: category.into(),
            },
        }
    }
}

/// Wrapper für `POST /shiftplan-edit/booking` (BOOK-02 Reverse-Warning).
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct BookingCreateResultTO {
    pub booking: BookingTO,
    pub warnings: Vec<WarningTO>,
}

#[cfg(feature = "service-impl")]
impl From<&service::shiftplan_edit::BookingCreateResult> for BookingCreateResultTO {
    fn from(r: &service::shiftplan_edit::BookingCreateResult) -> Self {
        Self {
            booking: BookingTO::from(&r.booking),
            warnings: r.warnings.iter().map(WarningTO::from).collect(),
        }
    }
}

/// Wrapper für `POST /shiftplan-edit/copy-week` (BOOK-02 / D-Phase3-02).
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct CopyWeekResultTO {
    pub copied_bookings: Vec<BookingTO>,
    pub warnings: Vec<WarningTO>,
}

#[cfg(feature = "service-impl")]
impl From<&service::shiftplan_edit::CopyWeekResult> for CopyWeekResultTO {
    fn from(r: &service::shiftplan_edit::CopyWeekResult) -> Self {
        Self {
            copied_bookings: r.copied_bookings.iter().map(BookingTO::from).collect(),
            warnings: r.warnings.iter().map(WarningTO::from).collect(),
        }
    }
}

/// Wrapper für `POST /absence-period` und `PATCH /absence-period/{id}`
/// (BOOK-01 Forward-Warning).
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct AbsencePeriodCreateResultTO {
    pub absence: AbsencePeriodTO,
    pub warnings: Vec<WarningTO>,
}

#[cfg(feature = "service-impl")]
impl From<&service::absence::AbsencePeriodCreateResult> for AbsencePeriodCreateResultTO {
    fn from(r: &service::absence::AbsencePeriodCreateResult) -> Self {
        Self {
            absence: AbsencePeriodTO::from(&r.absence),
            warnings: r.warnings.iter().map(WarningTO::from).collect(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Phase 4 — Cutover DTOs (Plan 04-06)
//
// Inline per Phase-3 wrapper-DTO precedent (BookingCreateResultTO above).
// `From` impls are gated behind `feature = "service-impl"` so the wasm /
// frontend build doesn't drag the service crate in.
// ─────────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct CutoverGateDriftRowTO {
    pub sales_person_id: Uuid,
    pub sales_person_name: String,
    pub category: AbsenceCategoryTO,
    pub year: u32,
    pub legacy_sum: f32,
    pub derived_sum: f32,
    pub drift: f32,
    pub quarantined_extra_hours_count: u32,
    pub quarantine_reasons: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct CutoverGateDriftReportTO {
    pub gate_run_id: Uuid,
    /// ISO-8601 UTC timestamp string (OpenAPI-portable).
    pub run_at: String,
    pub dry_run: bool,
    pub drift_threshold: f32,
    pub total_drift_rows: u32,
    pub drift: Vec<CutoverGateDriftRowTO>,
    pub passed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct CutoverRunResultTO {
    pub run_id: Uuid,
    /// ISO-8601 UTC timestamp string (OpenAPI-portable).
    pub ran_at: String,
    pub dry_run: bool,
    pub gate_passed: bool,
    pub total_clusters: u32,
    pub migrated_clusters: u32,
    pub quarantined_rows: u32,
    pub gate_drift_rows: u32,
    pub diff_report_path: Option<String>,
}

/// Body shape for the HTTP-403 returned when a deprecated ExtraHoursCategory
/// is POST'd after the cutover flag is on (D-Phase4-09).
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct ExtraHoursCategoryDeprecatedErrorTO {
    /// Always `"extra_hours_category_deprecated"`.
    pub error: String,
    /// Lowercase variant name (e.g. `"vacation"`).
    pub category: String,
    /// User-facing migration hint.
    pub message: String,
}

/// Per-(sales_person, category, year) profile bucket per C-Phase4-05.
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct CutoverProfileBucketTO {
    pub sales_person_id: Uuid,
    pub sales_person_name: String,
    pub category: AbsenceCategoryTO,
    pub year: u32,
    pub row_count: u32,
    /// Equivalent to `sum_amount` in the service-layer struct.
    pub sum_hours: f32,
    pub fractional_count: u32,
    pub weekend_on_workday_only_count: u32,
    pub iso_53_indicator: bool,
}

/// Production-data profile envelope — wraps every bucket plus run metadata.
/// Persisted to `.planning/migration-backup/profile-{ts}.json` and returned
/// verbatim from `POST /admin/cutover/profile`.
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct CutoverProfileTO {
    pub profile_run_id: Uuid,
    /// ISO-8601 UTC timestamp string.
    pub run_at: String,
    pub total_buckets: u32,
    pub buckets: Vec<CutoverProfileBucketTO>,
    /// Server-controlled file path under `.planning/migration-backup/`.
    pub output_path: String,
}

#[cfg(feature = "service-impl")]
impl From<&service::cutover::DriftRow> for CutoverGateDriftRowTO {
    fn from(r: &service::cutover::DriftRow) -> Self {
        Self {
            sales_person_id: r.sales_person_id,
            sales_person_name: r.sales_person_name.to_string(),
            category: AbsenceCategoryTO::from(&r.category),
            year: r.year,
            legacy_sum: r.legacy_sum,
            derived_sum: r.derived_sum,
            drift: r.drift,
            quarantined_extra_hours_count: r.quarantined_extra_hours_count,
            quarantine_reasons: r
                .quarantine_reasons
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }
}

#[cfg(feature = "service-impl")]
impl From<&service::cutover::CutoverRunResult> for CutoverRunResultTO {
    fn from(r: &service::cutover::CutoverRunResult) -> Self {
        let ran_at = r
            .ran_at
            .assume_utc()
            .format(&time::format_description::well_known::Iso8601::DEFAULT)
            .unwrap_or_default();
        Self {
            run_id: r.run_id,
            ran_at,
            dry_run: r.dry_run,
            gate_passed: r.gate_passed,
            total_clusters: r.total_clusters,
            migrated_clusters: r.migrated_clusters,
            quarantined_rows: r.quarantined_rows,
            gate_drift_rows: r.gate_drift_rows,
            diff_report_path: r.diff_report_path.as_ref().map(|s| s.to_string()),
        }
    }
}

#[cfg(feature = "service-impl")]
impl From<&service::cutover::CutoverProfileBucket> for CutoverProfileBucketTO {
    fn from(b: &service::cutover::CutoverProfileBucket) -> Self {
        Self {
            sales_person_id: b.sales_person_id,
            sales_person_name: b.sales_person_name.to_string(),
            category: AbsenceCategoryTO::from(&b.category),
            year: b.year,
            row_count: b.row_count,
            sum_hours: b.sum_amount,
            fractional_count: b.fractional_count,
            weekend_on_workday_only_count: b.weekend_on_workday_only_contract_count,
            iso_53_indicator: b.iso_53_indicator,
        }
    }
}

#[cfg(feature = "service-impl")]
impl From<&service::cutover::CutoverProfile> for CutoverProfileTO {
    fn from(p: &service::cutover::CutoverProfile) -> Self {
        let run_at = p
            .generated_at
            .assume_utc()
            .format(&time::format_description::well_known::Iso8601::DEFAULT)
            .unwrap_or_default();
        Self {
            profile_run_id: p.run_id,
            run_at,
            total_buckets: p.buckets.len() as u32,
            buckets: p.buckets.iter().map(CutoverProfileBucketTO::from).collect(),
            output_path: p.profile_path.to_string(),
        }
    }
}
