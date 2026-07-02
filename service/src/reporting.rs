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
    UnpaidLeave,
    VolunteerWork,
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
            crate::extra_hours::ExtraHoursCategory::UnpaidLeave => Self::UnpaidLeave,
            crate::extra_hours::ExtraHoursCategory::VolunteerWork => Self::VolunteerWork,
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
    pub dynamic_hours: f32,
    pub overall_hours: f32,
    pub balance: f32,

    pub days_per_week: u8,
    pub workdays_per_week: f32,

    pub shiftplan_hours: f32,
    pub extra_work_hours: f32,
    pub vacation_hours: f32,
    pub sick_leave_hours: f32,
    pub holiday_hours: f32,
    pub unpaid_leave_hours: f32,
    pub volunteer_hours: f32,

    pub custom_extra_hours: Arc<[CustomExtraHours]>,

    pub days: Arc<[WorkingHoursDay]>,
}
impl GroupedReportHours {
    pub fn hours_per_day(&self) -> f32 {
        if self.workdays_per_week == 0.0 {
            return 0.0;
        }
        self.contract_weekly_hours / self.workdays_per_week
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
        (self.vacation_hours + self.sick_leave_hours + self.holiday_hours + self.unpaid_leave_hours) / self.hours_per_day()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ShortEmployeeReport {
    pub sales_person: Arc<SalesPerson>,
    pub balance_hours: f32,
    pub dynamic_hours: f32,
    pub expected_hours: f32,
    pub overall_hours: f32,
    pub vacation_hours: f32,
    pub sick_leave_hours: f32,
    pub holiday_hours: f32,
    pub unavailable_hours: f32,
    pub unpaid_leave_hours: f32,
    pub volunteer_hours: f32,
    pub custom_absence_hours: Arc<[CustomExtraHours]>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EmployeeReport {
    pub sales_person: Arc<SalesPerson>,

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
    pub volunteer_hours: f32,

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

/// Result of the A-22-1 average-worked-hours-per-week computation.
#[derive(Clone, Debug, PartialEq)]
pub struct EmployeeWeeklyStatistics {
    /// Average worked hours per week (overall_hours + volunteer_hours), excluding
    /// fully-absent weeks from the denominator (A-22-1).
    pub average_worked_hours_per_week: f32,
    /// Number of weeks included in the average (denominator).
    pub included_weeks: u32,
    /// Sum of worked hours across all included weeks (numerator).
    pub total_worked_hours: f32,
}

/// Pure A-22-1 formula: compute average worked hours per week from a slice of
/// per-week data.
///
/// - Worked for a week = `overall_hours + volunteer_hours`.
/// - A week is "fully absent" iff `worked == 0.0 && absence > 0.0`, where
///   `absence = vacation_hours + sick_leave_hours + unpaid_leave_hours + holiday_hours`.
/// - Fully-absent weeks are EXCLUDED from the denominator.
/// - Weeks with `worked == 0.0 && absence == 0.0` are INCLUDED as 0.
/// - Empty included set → average 0.0, included_weeks 0.
/// - Formula does NOT reference expected_hours or contract_weekly_hours (flexible-contract safe).
pub fn average_worked_hours_per_week(weeks: &[GroupedReportHours]) -> EmployeeWeeklyStatistics {
    let mut total_worked_hours: f32 = 0.0;
    let mut included_weeks: u32 = 0;

    for w in weeks {
        let worked = w.overall_hours + w.volunteer_hours;
        let absence =
            w.vacation_hours + w.sick_leave_hours + w.unpaid_leave_hours + w.holiday_hours;
        // Exclude fully-absent weeks (worked == 0 and some absence recorded).
        if worked == 0.0 && absence > 0.0 {
            continue;
        }
        total_worked_hours += worked;
        included_weeks += 1;
    }

    let average_worked_hours_per_week = if included_weeks > 0 {
        total_worked_hours / included_weeks as f32
    } else {
        0.0
    };

    EmployeeWeeklyStatistics {
        average_worked_hours_per_week,
        included_weeks,
        total_worked_hours,
    }
}

/// Result of the AVG-01 attendance-day metric (Phase 41).
///
/// Deliberately distinct from A-22-1 (`EmployeeWeeklyStatistics`): this is a
/// day-based average, not a week-based one (D-AVG-01).
#[derive(Clone, Debug, PartialEq)]
pub struct EmployeeAttendanceStatistics {
    /// Average worked hours per attendance day, or None if fewer than 2
    /// attendance days (D-AVG-06 — not meaningful below the threshold).
    pub average_hours_per_attendance_day: Option<f32>,
    /// Number of distinct calendar dates counted as attendance days (denominator).
    pub attendance_days: u32,
    /// Sum of worked hours across all attendance days (numerator).
    pub total_worked_hours: f32,
}

/// Pure AVG-01 formula: average worked hours per attendance day.
///
/// A day counts as an attendance day iff it has at least one entry with
/// category in {Shiftplan, ExtraWork, VolunteerWork} and `hours > 0` (D-AVG-02).
/// Absence categories (Vacation, SickLeave, Holiday, UnpaidLeave, Unavailable)
/// and `Custom(_)` are NOT attendance categories (D-AVG-03) — they drop out of
/// both numerator and denominator by construction of the filter.
///
/// - Denominator = number of DISTINCT dates among work-category entries
///   (deduplicated via `BTreeSet<time::Date>`, so a date with several work
///   entries counts once).
/// - Numerator = sum of `hours` over all work-category entries.
/// - Denominator < 2 → `average_hours_per_attendance_day` is None (D-AVG-06).
///
/// This is a separate function from A-22-1 (`average_worked_hours_per_week`):
/// different input type (`&[WorkingHoursDay]`) and different result struct.
pub fn average_hours_per_attendance_day(days: &[WorkingHoursDay]) -> EmployeeAttendanceStatistics {
    use std::collections::BTreeSet;
    use ExtraHoursReportCategory::{ExtraWork, Shiftplan, VolunteerWork};

    // Work-category entries with positive hours only (D-AVG-02/03).
    let work_entries = days.iter().filter(|d| {
        d.hours > 0.0 && matches!(d.category, Shiftplan | ExtraWork | VolunteerWork)
    });

    // Distinct attendance dates → denominator.
    let attendance_date_set: BTreeSet<time::Date> =
        work_entries.clone().map(|d| d.date).collect();
    let attendance_days = attendance_date_set.len() as u32;

    // Sum all worked hours → numerator.
    let total_worked_hours: f32 = work_entries.map(|d| d.hours).sum();

    let average_hours_per_attendance_day = if attendance_days >= 2 {
        Some(total_worked_hours / attendance_days as f32)
    } else {
        None // D-AVG-06: not meaningful below the 2-day threshold
    };

    EmployeeAttendanceStatistics {
        average_hours_per_attendance_day,
        attendance_days,
        total_worked_hours,
    }
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

    /// Returns average worked hours per week (A-22-1) for the current year up
    /// to today, for the given sales person. HR-gated (STAT-01/D-22-05).
    async fn get_employee_weekly_statistics(
        &self,
        sales_person_id: &Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<EmployeeWeeklyStatistics, ServiceError>;
}
