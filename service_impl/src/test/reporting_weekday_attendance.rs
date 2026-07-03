//! Unit tests for the RPT-01 weekday-attendance-distribution pure fn.
//!
//! Tests the pure free function
//! `service::reporting::weekday_attendance_distribution` over a slice of
//! `WorkingHoursDay` plus a `counted_calendar_weeks: u32` denominator. No
//! mocks — purely data-driven.
//!
//! Semantics (locked in 47-CONTEXT, mirroring v2.1 D-AVG-02/03):
//! - Attendance day = category in {Shiftplan, ExtraWork, VolunteerWork} AND `hours > 0`.
//! - Distinct-date dedupe per weekday (BTreeSet<Date> per weekday bucket).
//! - `share = min(count / counted_calendar_weeks, 1.0)` rounded to two decimals.
//! - `counted_calendar_weeks == 0` → all shares 0.0 (no NaN, no +Inf).
//! - Result is ALWAYS length 7, ordered Monday..Sunday, every weekday present.

use service::reporting::{
    weekday_attendance_distribution, EmployeeAttendanceStatistics, ExtraHoursReportCategory,
    WeekdayAttendanceStat, WorkingHoursDay,
};
use shifty_utils::{DayOfWeek, LazyLoad};
use time::macros::date;
use uuid::Uuid;

fn day(date: time::Date, hours: f32, category: ExtraHoursReportCategory) -> WorkingHoursDay {
    WorkingHoursDay {
        date,
        hours,
        category,
    }
}

/// Test 1: empty input + zero weeks → all counts 0, all shares 0.0.
#[test]
fn empty_input_all_zero_and_zero_weeks() {
    let days: &[WorkingHoursDay] = &[];
    let stats: EmployeeAttendanceStatistics = weekday_attendance_distribution(days, 0);
    assert_eq!(stats.counted_calendar_weeks, 0);
    assert_eq!(stats.attendance_by_weekday.len(), 7);
    for stat in stats.attendance_by_weekday.iter() {
        assert_eq!(stat.count, 0);
        assert!(stat.share.is_finite(), "share must be finite, got {}", stat.share);
        assert!((stat.share - 0.0).abs() < 1e-6, "share must be 0.0, got {}", stat.share);
    }
}

/// Test 2: 5 Mondays across 5 weeks → Mo count=5 share=1.0, all others 0/0.
#[test]
fn even_distribution_over_5_weeks() {
    let days = [
        day(date!(2026 - 01 - 05), 8.0, ExtraHoursReportCategory::Shiftplan),
        day(date!(2026 - 01 - 12), 8.0, ExtraHoursReportCategory::Shiftplan),
        day(date!(2026 - 01 - 19), 8.0, ExtraHoursReportCategory::Shiftplan),
        day(date!(2026 - 01 - 26), 8.0, ExtraHoursReportCategory::Shiftplan),
        day(date!(2026 - 02 - 02), 8.0, ExtraHoursReportCategory::Shiftplan),
    ];
    let stats = weekday_attendance_distribution(&days, 5);
    assert_eq!(stats.counted_calendar_weeks, 5);
    let mon = &stats.attendance_by_weekday[0];
    assert_eq!(mon.weekday, DayOfWeek::Monday);
    assert_eq!(mon.count, 5);
    assert!((mon.share - 1.0).abs() < 1e-6, "Mo share should be 1.0, got {}", mon.share);
    for stat in stats.attendance_by_weekday.iter().skip(1) {
        assert_eq!(stat.count, 0);
        assert!((stat.share - 0.0).abs() < 1e-6);
    }
}

/// Test 3: partial share rounds to two decimals.
#[test]
fn partial_share_rounds_to_two_decimals() {
    // 8 Mondays across 10 weeks → share=0.80
    // 3 Tuesdays across 10 weeks → share=0.30
    let days = [
        // 8 Mondays
        day(date!(2026 - 01 - 05), 4.0, ExtraHoursReportCategory::Shiftplan),
        day(date!(2026 - 01 - 12), 4.0, ExtraHoursReportCategory::Shiftplan),
        day(date!(2026 - 01 - 19), 4.0, ExtraHoursReportCategory::Shiftplan),
        day(date!(2026 - 01 - 26), 4.0, ExtraHoursReportCategory::Shiftplan),
        day(date!(2026 - 02 - 02), 4.0, ExtraHoursReportCategory::Shiftplan),
        day(date!(2026 - 02 - 09), 4.0, ExtraHoursReportCategory::Shiftplan),
        day(date!(2026 - 02 - 16), 4.0, ExtraHoursReportCategory::Shiftplan),
        day(date!(2026 - 02 - 23), 4.0, ExtraHoursReportCategory::Shiftplan),
        // 3 Tuesdays
        day(date!(2026 - 01 - 06), 4.0, ExtraHoursReportCategory::ExtraWork),
        day(date!(2026 - 01 - 13), 4.0, ExtraHoursReportCategory::ExtraWork),
        day(date!(2026 - 01 - 20), 4.0, ExtraHoursReportCategory::ExtraWork),
    ];
    let stats = weekday_attendance_distribution(&days, 10);
    let mon = &stats.attendance_by_weekday[0];
    let tue = &stats.attendance_by_weekday[1];
    assert_eq!(mon.weekday, DayOfWeek::Monday);
    assert_eq!(mon.count, 8);
    assert!((mon.share - 0.80).abs() < 1e-4, "Mo share should be 0.80, got {}", mon.share);
    assert_eq!(tue.weekday, DayOfWeek::Tuesday);
    assert_eq!(tue.count, 3);
    assert!((tue.share - 0.30).abs() < 1e-4, "Tu share should be 0.30, got {}", tue.share);
}

/// Test 4: mixed categories on same date + absence excluded + Custom(_) excluded.
#[test]
fn mixed_categories_and_absence_excluded() {
    let days = [
        // 2026-03-02 is a Monday
        day(date!(2026 - 03 - 02), 4.0, ExtraHoursReportCategory::Shiftplan),
        day(date!(2026 - 03 - 02), 4.0, ExtraHoursReportCategory::Vacation),   // excluded
        day(date!(2026 - 03 - 02), 0.0, ExtraHoursReportCategory::ExtraWork),  // 0h excluded
        // 2026-03-03 is a Tuesday, Custom(_) excluded
        day(
            date!(2026 - 03 - 03),
            5.0,
            ExtraHoursReportCategory::Custom(LazyLoad::new(Uuid::nil())),
        ),
    ];
    let stats = weekday_attendance_distribution(&days, 4);
    let mon = &stats.attendance_by_weekday[0];
    let tue = &stats.attendance_by_weekday[1];
    assert_eq!(mon.count, 1, "Monday count should be 1 (only Shiftplan 4h)");
    assert_eq!(tue.count, 0, "Tuesday count should be 0 (Custom is excluded)");
}

/// Test 5: same date + multiple work entries → counted ONCE per weekday.
#[test]
fn same_date_multiple_work_entries_counts_once() {
    let days = [
        // 2026-03-09 is a Monday
        day(date!(2026 - 03 - 09), 4.0, ExtraHoursReportCategory::Shiftplan),
        day(date!(2026 - 03 - 09), 4.0, ExtraHoursReportCategory::ExtraWork),
        day(date!(2026 - 03 - 09), 1.0, ExtraHoursReportCategory::VolunteerWork),
    ];
    let stats = weekday_attendance_distribution(&days, 1);
    let mon = &stats.attendance_by_weekday[0];
    assert_eq!(mon.count, 1, "Monday count should be 1 (dedupe by date)");
    for stat in stats.attendance_by_weekday.iter().skip(1) {
        assert_eq!(stat.count, 0);
    }
}

/// Test 6: full week 2026-04-06..2026-04-12 all Shiftplan 4h → each weekday count=1 share=1.0.
/// Verifies the array is ordered Mon, Tue, Wed, Thu, Fri, Sat, Sun.
#[test]
fn all_seven_weekdays_present() {
    let days = [
        day(date!(2026 - 04 - 06), 4.0, ExtraHoursReportCategory::Shiftplan), // Mo
        day(date!(2026 - 04 - 07), 4.0, ExtraHoursReportCategory::Shiftplan), // Tu
        day(date!(2026 - 04 - 08), 4.0, ExtraHoursReportCategory::Shiftplan), // We
        day(date!(2026 - 04 - 09), 4.0, ExtraHoursReportCategory::Shiftplan), // Th
        day(date!(2026 - 04 - 10), 4.0, ExtraHoursReportCategory::Shiftplan), // Fr
        day(date!(2026 - 04 - 11), 4.0, ExtraHoursReportCategory::Shiftplan), // Sa
        day(date!(2026 - 04 - 12), 4.0, ExtraHoursReportCategory::Shiftplan), // Su
    ];
    let stats = weekday_attendance_distribution(&days, 1);
    let expected = [
        DayOfWeek::Monday,
        DayOfWeek::Tuesday,
        DayOfWeek::Wednesday,
        DayOfWeek::Thursday,
        DayOfWeek::Friday,
        DayOfWeek::Saturday,
        DayOfWeek::Sunday,
    ];
    for (i, stat) in stats.attendance_by_weekday.iter().enumerate() {
        assert_eq!(stat.weekday, expected[i], "weekday at index {} should be {:?}", i, expected[i]);
        assert_eq!(stat.count, 1, "count at index {} should be 1", i);
        assert!((stat.share - 1.0).abs() < 1e-6, "share at index {} should be 1.0, got {}", i, stat.share);
    }
}

/// Test 7: `counted_calendar_weeks == 0` → shares are 0.0 (finite), never NaN or +Inf.
#[test]
fn zero_weeks_yields_zero_shares_not_nan() {
    let days = [
        day(date!(2026 - 01 - 05), 4.0, ExtraHoursReportCategory::Shiftplan),
        day(date!(2026 - 01 - 12), 4.0, ExtraHoursReportCategory::Shiftplan),
        day(date!(2026 - 01 - 19), 4.0, ExtraHoursReportCategory::Shiftplan),
    ];
    let stats: EmployeeAttendanceStatistics = weekday_attendance_distribution(&days, 0);
    let mon: &WeekdayAttendanceStat = &stats.attendance_by_weekday[0];
    assert_eq!(mon.count, 3);
    assert!(mon.share.is_finite(), "share must be finite, got {}", mon.share);
    assert!((mon.share - 0.0).abs() < 1e-6, "share must be 0.0, got {}", mon.share);
}

/// Test 8: share is clamped to ≤ 1.0 even when count > counted_calendar_weeks.
#[test]
fn share_never_exceeds_one() {
    let days = [
        day(date!(2026 - 01 - 05), 4.0, ExtraHoursReportCategory::Shiftplan),
        day(date!(2026 - 01 - 12), 4.0, ExtraHoursReportCategory::Shiftplan),
        day(date!(2026 - 01 - 19), 4.0, ExtraHoursReportCategory::Shiftplan),
    ];
    let stats = weekday_attendance_distribution(&days, 2);
    let mon = &stats.attendance_by_weekday[0];
    assert_eq!(mon.count, 3);
    assert!(mon.share <= 1.0, "share must be ≤ 1.0, got {}", mon.share);
    assert!((mon.share - 1.0).abs() < 1e-6, "share should be clamped to 1.0, got {}", mon.share);
}
