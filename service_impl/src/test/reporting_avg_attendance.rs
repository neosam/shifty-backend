//! Unit tests for the AVG-01 average-hours-per-attendance-day formula.
//!
//! Tests the pure free function `service::reporting::average_hours_per_attendance_day`
//! over a slice of `WorkingHoursDay`. No mocks needed — purely data-driven.
//!
//! Covered cases (Phase 41, D-AVG-01..08):
//! - user_example: 12 distinct attendance days, 54h total → Some(4.5).
//! - absence_day_not_counted: Vacation day is NOT an attendance day (D-AVG-03).
//! - mixed_day_counts_work_only: same date work + absence → 1 day, numerator work only.
//! - custom_category_not_attendance: Custom(_) is NOT an attendance category (D-AVG-02).
//! - empty_slice_returns_none: empty input → attendance_days 0, average None (D-AVG-06).
//! - one_day_returns_none: exactly 1 attendance day → None (< 2 threshold).
//! - two_days_returns_some: exactly 2 attendance days → Some (threshold met).

use service::reporting::{
    average_hours_per_attendance_day, ExtraHoursReportCategory, WorkingHoursDay,
};
use shifty_utils::LazyLoad;
use time::macros::date;
use uuid::Uuid;

/// Build a `WorkingHoursDay` with the given date, hours and category.
fn day(date: time::Date, hours: f32, category: ExtraHoursReportCategory) -> WorkingHoursDay {
    WorkingHoursDay {
        date,
        hours,
        category,
    }
}

/// D-AVG-01/02: 12 distinct attendance days summing to 54 worked hours → 4.5 h/day.
/// 6 days of 4h + 6 days of 5h = 24 + 30 = 54h over 12 distinct dates → Some(4.5).
#[test]
fn user_example() {
    let days = [
        day(date!(2026 - 01 - 01), 4.0, ExtraHoursReportCategory::Shiftplan),
        day(date!(2026 - 01 - 02), 4.0, ExtraHoursReportCategory::Shiftplan),
        day(date!(2026 - 01 - 03), 4.0, ExtraHoursReportCategory::Shiftplan),
        day(date!(2026 - 01 - 04), 4.0, ExtraHoursReportCategory::Shiftplan),
        day(date!(2026 - 01 - 05), 4.0, ExtraHoursReportCategory::Shiftplan),
        day(date!(2026 - 01 - 06), 4.0, ExtraHoursReportCategory::Shiftplan),
        day(date!(2026 - 01 - 07), 5.0, ExtraHoursReportCategory::ExtraWork),
        day(date!(2026 - 01 - 08), 5.0, ExtraHoursReportCategory::ExtraWork),
        day(date!(2026 - 01 - 09), 5.0, ExtraHoursReportCategory::ExtraWork),
        day(date!(2026 - 01 - 10), 5.0, ExtraHoursReportCategory::VolunteerWork),
        day(date!(2026 - 01 - 11), 5.0, ExtraHoursReportCategory::VolunteerWork),
        day(date!(2026 - 01 - 12), 5.0, ExtraHoursReportCategory::VolunteerWork),
    ];
    let stats = average_hours_per_attendance_day(&days);
    assert_eq!(stats.attendance_days, 12);
    assert!((stats.total_worked_hours - 54.0).abs() < 0.001, "total_worked_hours={}", stats.total_worked_hours);
    let avg = stats.average_hours_per_attendance_day.expect("average should be Some for 12 attendance days");
    assert!((avg - 4.5).abs() < 0.001, "avg={avg}");
}

/// D-AVG-03: a Vacation day (hours>0, no work) is NOT an attendance day.
/// With one work day + one vacation day, only the work day counts → 1 attendance day.
#[test]
fn absence_day_not_counted() {
    let days = [
        day(date!(2026 - 02 - 01), 8.0, ExtraHoursReportCategory::Shiftplan),
        day(date!(2026 - 02 - 02), 8.0, ExtraHoursReportCategory::Vacation),
    ];
    let stats = average_hours_per_attendance_day(&days);
    assert_eq!(stats.attendance_days, 1);
    assert!((stats.total_worked_hours - 8.0).abs() < 0.001);
    assert_eq!(stats.average_hours_per_attendance_day, None);
}

/// D-AVG-02/03: same calendar date with Shiftplan=4h + Vacation=4h → 1 attendance day,
/// numerator counts work hours only (4h, not 8h).
#[test]
fn mixed_day_counts_work_only() {
    let days = [
        day(date!(2026 - 03 - 01), 4.0, ExtraHoursReportCategory::Shiftplan),
        day(date!(2026 - 03 - 01), 4.0, ExtraHoursReportCategory::Vacation),
        day(date!(2026 - 03 - 02), 4.0, ExtraHoursReportCategory::Shiftplan),
    ];
    let stats = average_hours_per_attendance_day(&days);
    assert_eq!(stats.attendance_days, 2);
    assert!((stats.total_worked_hours - 8.0).abs() < 0.001);
    assert_eq!(stats.average_hours_per_attendance_day, Some(4.0));
}

/// D-AVG-02: Custom(_) is NOT an attendance category, even with hours>0.
#[test]
fn custom_category_not_attendance() {
    let days = [day(
        date!(2026 - 04 - 01),
        6.0,
        ExtraHoursReportCategory::Custom(LazyLoad::new(Uuid::nil())),
    )];
    let stats = average_hours_per_attendance_day(&days);
    assert_eq!(stats.attendance_days, 0);
    assert!((stats.total_worked_hours - 0.0).abs() < 0.001);
    assert_eq!(stats.average_hours_per_attendance_day, None);
}

/// D-AVG-06: empty slice → attendance_days 0, average None (no division by zero).
#[test]
fn empty_slice_returns_none() {
    let days: &[WorkingHoursDay] = &[];
    let stats = average_hours_per_attendance_day(days);
    assert_eq!(stats.attendance_days, 0);
    assert!((stats.total_worked_hours - 0.0).abs() < 0.001);
    assert_eq!(stats.average_hours_per_attendance_day, None);
}

/// D-AVG-06: exactly 1 attendance day → average None (below 2-day threshold).
#[test]
fn one_day_returns_none() {
    let days = [day(
        date!(2026 - 05 - 01),
        7.0,
        ExtraHoursReportCategory::Shiftplan,
    )];
    let stats = average_hours_per_attendance_day(&days);
    assert_eq!(stats.attendance_days, 1);
    assert!((stats.total_worked_hours - 7.0).abs() < 0.001);
    assert_eq!(stats.average_hours_per_attendance_day, None);
}

/// D-AVG-06: exactly 2 attendance days → average Some (threshold met).
#[test]
fn two_days_returns_some() {
    let days = [
        day(date!(2026 - 06 - 01), 6.0, ExtraHoursReportCategory::Shiftplan),
        day(date!(2026 - 06 - 02), 4.0, ExtraHoursReportCategory::ExtraWork),
    ];
    let stats = average_hours_per_attendance_day(&days);
    assert_eq!(stats.attendance_days, 2);
    assert!((stats.total_worked_hours - 10.0).abs() < 0.001);
    assert_eq!(stats.average_hours_per_attendance_day, Some(5.0));
}
