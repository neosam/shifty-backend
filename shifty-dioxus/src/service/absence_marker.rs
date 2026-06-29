//! Pure helper: maps a person's absence periods to the weekday column indices
//! that should receive the existing "Nicht Verfügbar" (discourage) marker in the
//! shift-plan grid (Phase 31, D-31-01 / D-31-04).
//!
//! This module is **intentionally free of Dioxus / browser dependencies** so that
//! `cargo test` can exercise the business logic without a WASM runtime.

use crate::state::absence_period::{AbsenceCategory, AbsencePeriod, DayFraction};
use crate::state::shiftplan::Weekday;

/// Returns `true` for every `AbsenceCategory` variant that triggers the discourage
/// marker.  Written as an **exhaustive match** (no wildcard) so that adding a
/// fourth variant forces a deliberate review here — zero drift vs.
/// `shiftplan_edit.rs:538` (D-31-01 / SC2).
fn category_triggers_marker(c: AbsenceCategory) -> bool {
    match c {
        AbsenceCategory::Vacation => true,
        AbsenceCategory::SickLeave => true,
        AbsenceCategory::UnpaidLeave => true,
    }
}

/// Maps a slice of `AbsencePeriod`s to the `Weekday`s within the displayed week
/// that should be shown as discouraged ("Nicht Verfügbar").
///
/// For each of the seven days in the week (Monday … Sunday), the day is included
/// in the result iff **at least one** absence satisfies **all** of:
/// 1. `day_fraction == Full` — half-day absences are silently tolerated, mirroring
///    `shiftplan_edit.rs:538` (D-31-01 / SC2).
/// 2. `category_triggers_marker(category)` — all three current variants trigger it.
/// 3. The concrete calendar date falls within `[from_date, to_date]` inclusive.
///
/// `week_monday` must be the Monday of the displayed ISO week.  The function is
/// pure and has no side-effects, making it trivially `cargo test`-able (D-31-04).
pub fn absence_periods_to_discourage_days(
    absences: &[AbsencePeriod],
    week_monday: time::Date,
) -> Vec<Weekday> {
    (0u8..=6)
        .filter(|&offset| {
            let day = week_monday + time::Duration::days(offset as i64);
            absences.iter().any(|ap| {
                ap.day_fraction == DayFraction::Full
                    && category_triggers_marker(ap.category)
                    && ap.from_date <= day
                    && day <= ap.to_date
            })
        })
        .map(Weekday::from_num_from_monday)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use time::macros::date;
    use uuid::Uuid;

    /// Minimal builder so the `AbsencePeriod` field list lives in one place.
    fn make_absence(
        category: AbsenceCategory,
        from_date: time::Date,
        to_date: time::Date,
        day_fraction: DayFraction,
    ) -> AbsencePeriod {
        AbsencePeriod {
            id: Uuid::from_u128(1),
            sales_person_id: Uuid::from_u128(2),
            category,
            from_date,
            to_date,
            description: Arc::<str>::from(""),
            version: Uuid::from_u128(3),
            day_fraction,
            derived_days: 0.0,
            person_name: Arc::<str>::from(""),
            background_color: Arc::<str>::from(""),
        }
    }

    /// The Monday of the fixed test week (ISO week 27, 2026-06-29).
    const MONDAY: time::Date = date!(2026 - 06 - 29);

    /// Vacation, Full, Tue–Thu → exactly Tuesday, Wednesday, Thursday (D-31-04).
    #[test]
    fn vacation_full_tue_to_thu_yields_three_weekdays() {
        let absences = [make_absence(
            AbsenceCategory::Vacation,
            date!(2026 - 06 - 30), // Tuesday
            date!(2026 - 07 - 02), // Thursday
            DayFraction::Full,
        )];
        let result = absence_periods_to_discourage_days(&absences, MONDAY);
        assert_eq!(
            result,
            vec![Weekday::Tuesday, Weekday::Wednesday, Weekday::Thursday],
            "Vacation Full Tue–Thu should yield exactly Tue, Wed, Thu"
        );
    }

    /// SickLeave, Full, single in-week day (Wednesday) → Wednesday present.
    #[test]
    fn sick_leave_full_single_day_yields_that_weekday() {
        let absences = [make_absence(
            AbsenceCategory::SickLeave,
            date!(2026 - 07 - 01), // Wednesday
            date!(2026 - 07 - 01),
            DayFraction::Full,
        )];
        let result = absence_periods_to_discourage_days(&absences, MONDAY);
        assert!(
            result.contains(&Weekday::Wednesday),
            "Wednesday must be present for SickLeave Full"
        );
        assert_eq!(result.len(), 1, "Only Wednesday should be returned");
    }

    /// UnpaidLeave, Full, single in-week day (Friday) → Friday present.
    #[test]
    fn unpaid_leave_full_single_day_yields_that_weekday() {
        let absences = [make_absence(
            AbsenceCategory::UnpaidLeave,
            date!(2026 - 07 - 03), // Friday
            date!(2026 - 07 - 03),
            DayFraction::Full,
        )];
        let result = absence_periods_to_discourage_days(&absences, MONDAY);
        assert!(
            result.contains(&Weekday::Friday),
            "Friday must be present for UnpaidLeave Full"
        );
        assert_eq!(result.len(), 1, "Only Friday should be returned");
    }

    /// Vacation, Half, in-week → result is empty (half-day never marks; SC2).
    #[test]
    fn vacation_half_day_in_week_yields_empty() {
        let absences = [make_absence(
            AbsenceCategory::Vacation,
            date!(2026 - 06 - 30), // Tuesday
            date!(2026 - 07 - 02), // Thursday
            DayFraction::Half,
        )];
        let result = absence_periods_to_discourage_days(&absences, MONDAY);
        assert!(
            result.is_empty(),
            "Half-day absence must not mark any weekday (SC2 / D-31-01)"
        );
    }

    /// Vacation, Full, entirely outside the displayed week → result is empty.
    #[test]
    fn vacation_full_outside_week_yields_empty() {
        let absences = [make_absence(
            AbsenceCategory::Vacation,
            date!(2026 - 07 - 06), // Next Monday
            date!(2026 - 07 - 10), // Next Friday
            DayFraction::Full,
        )];
        let result = absence_periods_to_discourage_days(&absences, MONDAY);
        assert!(result.is_empty(), "Absence entirely outside the week must yield empty");
    }

    /// Vacation, Full, spanning the entire week (Mon–Sun) → all 7 weekdays.
    #[test]
    fn vacation_full_whole_week_yields_all_seven_weekdays() {
        let absences = [make_absence(
            AbsenceCategory::Vacation,
            date!(2026 - 06 - 29), // Monday
            date!(2026 - 07 - 05), // Sunday
            DayFraction::Full,
        )];
        let result = absence_periods_to_discourage_days(&absences, MONDAY);
        assert_eq!(result.len(), 7, "Full-week absence must mark all 7 weekdays");
    }

    /// Vacation, Full, starting before Monday and ending Wednesday →
    /// Monday, Tuesday, Wednesday present (partial overlap at week start).
    #[test]
    fn vacation_full_partial_overlap_start_yields_mon_tue_wed() {
        let absences = [make_absence(
            AbsenceCategory::Vacation,
            date!(2026 - 06 - 22), // Previous Monday
            date!(2026 - 07 - 01), // Wednesday of the test week
            DayFraction::Full,
        )];
        let result = absence_periods_to_discourage_days(&absences, MONDAY);
        assert_eq!(
            result,
            vec![Weekday::Monday, Weekday::Tuesday, Weekday::Wednesday],
            "Partial overlap at start should yield Mon, Tue, Wed"
        );
    }

    /// Vacation, Full, starting Friday of the test week and ending after Sunday →
    /// Friday, Saturday, Sunday present (partial overlap at week end). Symmetric
    /// to the start-overlap case (code-review IN-02).
    #[test]
    fn vacation_full_partial_overlap_end_yields_fri_sat_sun() {
        let absences = [make_absence(
            AbsenceCategory::Vacation,
            date!(2026 - 07 - 03), // Friday of the test week
            date!(2026 - 07 - 10), // next week — extends past Sunday 2026-07-05
            DayFraction::Full,
        )];
        let result = absence_periods_to_discourage_days(&absences, MONDAY);
        assert_eq!(
            result,
            vec![Weekday::Friday, Weekday::Saturday, Weekday::Sunday],
            "Partial overlap at end should yield Fri, Sat, Sun"
        );
    }
}
