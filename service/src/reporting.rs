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

/// RPT-01: per-weekday attendance-day statistic (Phase 47).
///
/// One entry per weekday (Mon..Sun). `count` is the number of DISTINCT
/// attendance dates that fall on this weekday within the report range;
/// `share` is `count / counted_calendar_weeks`, clamped to `0.0..=1.0` and
/// rounded to two decimals. When `counted_calendar_weeks == 0`, `share = 0.0`
/// (no NaN, no +Inf).
///
/// v2.2.1: additionally `hours` (sum of worked hours on this weekday within
/// the report range, same category filter as `count`) and `share_of_hours`
/// (hours / total_hours across all seven days, so the sum over Mon..Sun is
/// exactly 1.0 when total_hours > 0, else 0.0).
#[derive(Clone, Debug, PartialEq)]
pub struct WeekdayAttendanceStat {
    pub weekday: shifty_utils::DayOfWeek,
    pub count: u32,
    pub share: f32,
    pub hours: f32,
    pub share_of_hours: f32,
}

/// RPT-01: v2.2 attendance statistic — per-weekday distribution.
///
/// Replaces the v2.1 scalar Ø-hours metric with a count + share breakdown per
/// weekday over the displayed report range.
#[derive(Clone, Debug, PartialEq)]
pub struct EmployeeAttendanceStatistics {
    /// Always length 7, ordered Monday..Sunday. Every weekday present, even when count=0.
    pub attendance_by_weekday: [WeekdayAttendanceStat; 7],
    /// Number of calendar weeks counted in the denominator (= length of `report.by_week`).
    pub counted_calendar_weeks: u32,
}

/// Pure RPT-01 formula: per-weekday attendance-day distribution.
///
/// - Attendance-day filter (D-AVG-02/03, byte-identical to v2.1):
///   `d.hours > 0.0 && matches!(d.category, Shiftplan | ExtraWork | VolunteerWork)`.
/// - Distinct-date dedupe per weekday bucket (`BTreeSet<time::Date>`).
/// - `share = min(count / counted_calendar_weeks, 1.0)` rounded to two decimals
///   via `((x * 100.0).round() / 100.0)`.
/// - `counted_calendar_weeks == 0` → all shares 0.0 (no NaN, no +Inf).
/// - Result array is ALWAYS length 7, ordered Mon..Sun.
pub fn weekday_attendance_distribution(
    days: &[WorkingHoursDay],
    counted_calendar_weeks: u32,
) -> EmployeeAttendanceStatistics {
    use std::collections::BTreeSet;
    use ExtraHoursReportCategory::{ExtraWork, Shiftplan, VolunteerWork};

    // Seven distinct-date buckets, indexed by Monday=0..Sunday=6.
    let mut buckets: [BTreeSet<time::Date>; 7] = Default::default();
    // v2.2.1: parallel hour-sum buckets, same filter + weekday indexing.
    let mut hour_buckets: [f32; 7] = [0.0; 7];

    for d in days.iter().filter(|d| {
        d.hours > 0.0 && matches!(d.category, Shiftplan | ExtraWork | VolunteerWork)
    }) {
        let idx = weekday_index_mon0(d.date.weekday());
        buckets[idx].insert(d.date);
        hour_buckets[idx] += d.hours;
    }

    let total_hours: f32 = hour_buckets.iter().sum();

    let weekday_order = [
        shifty_utils::DayOfWeek::Monday,
        shifty_utils::DayOfWeek::Tuesday,
        shifty_utils::DayOfWeek::Wednesday,
        shifty_utils::DayOfWeek::Thursday,
        shifty_utils::DayOfWeek::Friday,
        shifty_utils::DayOfWeek::Saturday,
        shifty_utils::DayOfWeek::Sunday,
    ];

    let attendance_by_weekday: [WeekdayAttendanceStat; 7] = std::array::from_fn(|i| {
        let count = buckets[i].len() as u32;
        let share = if counted_calendar_weeks == 0 {
            0.0
        } else {
            let raw = (count as f32) / (counted_calendar_weeks as f32);
            let clamped = raw.min(1.0);
            (clamped * 100.0).round() / 100.0
        };
        let hours = hour_buckets[i];
        let share_of_hours = if total_hours <= 0.0 {
            0.0
        } else {
            let raw = hours / total_hours;
            (raw * 100.0).round() / 100.0
        };
        WeekdayAttendanceStat {
            weekday: weekday_order[i],
            count,
            share,
            hours,
            share_of_hours,
        }
    });

    EmployeeAttendanceStatistics {
        attendance_by_weekday,
        counted_calendar_weeks,
    }
}

/// Maps `time::Weekday` to an index in [0..7] with Monday=0..Sunday=6.
fn weekday_index_mon0(weekday: time::Weekday) -> usize {
    match weekday {
        time::Weekday::Monday => 0,
        time::Weekday::Tuesday => 1,
        time::Weekday::Wednesday => 2,
        time::Weekday::Thursday => 3,
        time::Weekday::Friday => 4,
        time::Weekday::Saturday => 5,
        time::Weekday::Sunday => 6,
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

    /// Returns the RPT-01 per-weekday attendance-day distribution for the given
    /// sales person over the displayed report range.
    ///
    /// - D-47-BE: same endpoint as v2.1, response shape swapped to `attendance_by_weekday`
    ///   (7 entries, Mon..Sun; each with count of attendance days + share of counted
    ///   calendar weeks) plus `counted_calendar_weeks`.
    /// - "Attendance day" definition (D-AVG-02/03, unchanged from v2.1): category in
    ///   {Shiftplan, ExtraWork, VolunteerWork} with `hours > 0`.
    /// - D-AVG-04: aggregates over the shown report window (`year` / `until_week`
    ///   via `get_report_for_employee`) — no separate date picker.
    /// - D-AVG-05: HR-gated (`HR_PRIVILEGE` is the FIRST operation, no data is
    ///   fetched before authorization) and server-side filtered on `is_dynamic`:
    ///   non-flexible employees yield `Ok(None)` (the metric is not computed nor
    ///   returned for them).
    /// - RPT-03: pure read aggregate — no new `BillingPeriodValueType`, no persistence,
    ///   snapshot version stays 12.
    async fn get_employee_attendance_statistics(
        &self,
        sales_person_id: &Uuid,
        year: u32,
        until_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Option<EmployeeAttendanceStatistics>, ServiceError>;
}
