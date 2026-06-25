//! Unit tests for the A-22-1 average-worked-hours-per-week formula.
//!
//! Tests the pure free function `service::reporting::average_worked_hours_per_week`
//! over a slice of `GroupedReportHours`. No mocks needed — purely data-driven.
//!
//! Covered cases (D-22-08):
//! - Fully-absent week excluded from denominator.
//! - Partial-absence week included with actual worked hours (not pro-rated).
//! - Zero-work-no-absence week counted as 0 (drags average down).
//! - Flexible/dynamic contract (contract_weekly_hours==0, expected_hours==0) works.
//! - Volunteer hours (overall_hours + volunteer_hours) counted toward worked.
//! - Empty input → average 0.0, included_weeks 0.

use std::sync::Arc;

use service::reporting::{
    average_worked_hours_per_week, GroupedReportHours,
};
use shifty_utils::{DayOfWeek, ShiftyDate};

/// Build a minimal `GroupedReportHours` with only the fields that matter for
/// the A-22-1 formula. All other fields get benign defaults (0.0 / empty).
fn week(
    overall: f32,
    volunteer: f32,
    vacation: f32,
    sick: f32,
    unpaid: f32,
    holiday: f32,
) -> GroupedReportHours {
    let from = ShiftyDate::new(2026, 1, DayOfWeek::Monday).unwrap();
    let to = ShiftyDate::new(2026, 1, DayOfWeek::Sunday).unwrap();
    GroupedReportHours {
        from,
        to,
        year: 2026,
        week: 1,
        contract_weekly_hours: 0.0,
        expected_hours: 0.0,
        dynamic_hours: 0.0,
        overall_hours: overall,
        balance: 0.0,
        days_per_week: 0,
        workdays_per_week: 0.0,
        shiftplan_hours: 0.0,
        extra_work_hours: 0.0,
        vacation_hours: vacation,
        sick_leave_hours: sick,
        holiday_hours: holiday,
        unpaid_leave_hours: unpaid,
        volunteer_hours: volunteer,
        custom_extra_hours: Arc::new([]),
        days: Arc::new([]),
    }
}

/// Fully-absent week (worked==0, some absence) must be excluded from denominator.
/// Setup: 3 weeks; week 2 is fully absent (vacation_hours=40).
/// Expected: average over weeks 1 and 3 only → (30.0 + 20.0) / 2 = 25.0.
#[test]
fn fully_absent_week_excluded() {
    let weeks = [
        week(30.0, 0.0, 0.0, 0.0, 0.0, 0.0), // included: worked=30
        week(0.0, 0.0, 40.0, 0.0, 0.0, 0.0), // excluded: worked=0, vacation=40
        week(20.0, 0.0, 0.0, 0.0, 0.0, 0.0), // included: worked=20
    ];
    let stats = average_worked_hours_per_week(&weeks);
    assert_eq!(stats.included_weeks, 2);
    assert!((stats.total_worked_hours - 50.0).abs() < 0.001);
    assert!((stats.average_worked_hours_per_week - 25.0).abs() < 0.001);
}

/// Partial-absence week (worked>0 AND some absence) must be INCLUDED with actual
/// worked hours — not excluded and not pro-rated.
/// Setup: 1 week with worked=12, vacation=8.
/// Expected: average = 12.0, included_weeks=1.
#[test]
fn partial_absence_week_included_with_actual_worked() {
    let weeks = [week(12.0, 0.0, 8.0, 0.0, 0.0, 0.0)];
    let stats = average_worked_hours_per_week(&weeks);
    assert_eq!(stats.included_weeks, 1);
    assert!((stats.total_worked_hours - 12.0).abs() < 0.001);
    assert!((stats.average_worked_hours_per_week - 12.0).abs() < 0.001);
}

/// A week with worked=0 AND absence=0 must count as 0 (included, drags average down).
/// Setup: 2 weeks; week 1 worked=20, week 2 worked=0 no absence.
/// Expected: average = (20 + 0) / 2 = 10.0, included_weeks=2.
#[test]
fn zero_work_no_absence_counts_as_zero() {
    let weeks = [
        week(20.0, 0.0, 0.0, 0.0, 0.0, 0.0), // included: worked=20
        week(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),   // included: worked=0, absence=0
    ];
    let stats = average_worked_hours_per_week(&weeks);
    assert_eq!(stats.included_weeks, 2);
    assert!((stats.total_worked_hours - 20.0).abs() < 0.001);
    assert!((stats.average_worked_hours_per_week - 10.0).abs() < 0.001);
}

/// Flexible/dynamic contract: contract_weekly_hours==0, expected_hours==0.
/// The formula MUST NOT reference those fields (A-22-1 flexible-contract safety).
/// Setup: 2 weeks with zero contract hours but actual worked hours.
/// Expected: simple sum / count with no panic or NaN.
#[test]
fn flexible_contract_no_expected_hours() {
    // week() already sets contract_weekly_hours=0.0 and expected_hours=0.0.
    let weeks = [
        week(15.0, 0.0, 0.0, 0.0, 0.0, 0.0),
        week(25.0, 0.0, 0.0, 0.0, 0.0, 0.0),
    ];
    let stats = average_worked_hours_per_week(&weeks);
    assert_eq!(stats.included_weeks, 2);
    assert!((stats.total_worked_hours - 40.0).abs() < 0.001);
    assert!((stats.average_worked_hours_per_week - 20.0).abs() < 0.001);
}

/// Volunteer hours (volunteer_hours) must be counted toward worked.
/// D-22-02: worked = overall_hours + volunteer_hours.
/// Setup: 1 week with overall=10, volunteer=5 → worked=15.
#[test]
fn volunteer_counts_toward_worked() {
    let weeks = [week(10.0, 5.0, 0.0, 0.0, 0.0, 0.0)];
    let stats = average_worked_hours_per_week(&weeks);
    assert_eq!(stats.included_weeks, 1);
    assert!((stats.total_worked_hours - 15.0).abs() < 0.001);
    assert!((stats.average_worked_hours_per_week - 15.0).abs() < 0.001);
}

/// Fully-absent via sick leave (not just vacation).
/// Setup: 1 week with overall=0, sick=32 → must be excluded.
#[test]
fn fully_absent_sick_leave_excluded() {
    let weeks = [week(0.0, 0.0, 0.0, 32.0, 0.0, 0.0)];
    let stats = average_worked_hours_per_week(&weeks);
    assert_eq!(stats.included_weeks, 0);
    assert!((stats.total_worked_hours - 0.0).abs() < 0.001);
    assert!((stats.average_worked_hours_per_week - 0.0).abs() < 0.001);
}

/// Empty input slice → average 0.0, included_weeks 0 (no division by zero).
#[test]
fn empty_input_returns_zero() {
    let weeks: &[GroupedReportHours] = &[];
    let stats = average_worked_hours_per_week(weeks);
    assert_eq!(stats.included_weeks, 0);
    assert!((stats.total_worked_hours - 0.0).abs() < 0.001);
    assert!((stats.average_worked_hours_per_week - 0.0).abs() < 0.001);
}

/// All four absence categories (vacation, sick, unpaid, holiday) each cause
/// full-absence exclusion when worked==0.
#[test]
fn all_absence_categories_cause_exclusion() {
    let weeks = [
        week(0.0, 0.0, 8.0, 0.0, 0.0, 0.0),  // vacation only → excluded
        week(0.0, 0.0, 0.0, 8.0, 0.0, 0.0),  // sick only → excluded
        week(0.0, 0.0, 0.0, 0.0, 8.0, 0.0),  // unpaid only → excluded
        week(0.0, 0.0, 0.0, 0.0, 0.0, 8.0),  // holiday only → excluded
    ];
    let stats = average_worked_hours_per_week(&weeks);
    assert_eq!(stats.included_weeks, 0);
    assert!((stats.average_worked_hours_per_week - 0.0).abs() < 0.001);
}

/// Volunteer-only week (overall=0, volunteer>0, no absence) must be INCLUDED
/// because worked = overall + volunteer > 0 (not fully absent).
#[test]
fn volunteer_only_week_included() {
    let weeks = [week(0.0, 8.0, 0.0, 0.0, 0.0, 0.0)];
    let stats = average_worked_hours_per_week(&weeks);
    assert_eq!(stats.included_weeks, 1);
    assert!((stats.total_worked_hours - 8.0).abs() < 0.001);
    assert!((stats.average_worked_hours_per_week - 8.0).abs() < 0.001);
}
