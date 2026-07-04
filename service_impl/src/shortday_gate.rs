//! Stichtag-Gate für die ShortDay-Slot-Kürzung (Phase 51, D-51-07).
//!
//! TDD RED phase — Skeleton mit `unimplemented!()` bodies; die Tests im
//! `#[cfg(test)] mod tests` müssen jetzt failen.

use shifty_utils::DayOfWeek;
use time::Date;

pub const TOGGLE_NAME: &str = "shortday_slot_clipping_active_from";

pub fn parse_active_from(_raw: Option<&str>) -> Option<Date> {
    unimplemented!("RED phase")
}

pub fn should_clip(_booking_date: Date, _active_from: Option<Date>) -> bool {
    unimplemented!("RED phase")
}

pub fn resolve_active_from_for_week(
    _year: u32,
    _week: u8,
    _day_of_week: DayOfWeek,
    _active_from: Option<Date>,
) -> bool {
    unimplemented!("RED phase")
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::Month;

    fn d(year: i32, month: Month, day: u8) -> Date {
        Date::from_calendar_date(year, month, day).expect("valid test date")
    }

    #[test]
    fn parse_none_returns_none() {
        assert_eq!(parse_active_from(None), None);
    }

    #[test]
    fn parse_empty_returns_none() {
        assert_eq!(parse_active_from(Some("")), None);
    }

    #[test]
    fn parse_malformed_returns_none() {
        assert_eq!(parse_active_from(Some("not-a-date")), None);
        assert_eq!(parse_active_from(Some("2026-13-40")), None);
    }

    #[test]
    fn parse_iso_valid() {
        assert_eq!(
            parse_active_from(Some("2026-08-01")),
            Some(d(2026, Month::August, 1))
        );
    }

    #[test]
    fn should_clip_none_active_from_returns_false() {
        assert!(!should_clip(d(2026, Month::August, 1), None));
        assert!(!should_clip(d(2030, Month::December, 31), None));
    }

    #[test]
    fn should_clip_before_stichtag_returns_false() {
        let active_from = Some(d(2026, Month::August, 1));
        assert!(!should_clip(d(2026, Month::July, 31), active_from));
        assert!(!should_clip(d(2025, Month::January, 1), active_from));
    }

    #[test]
    fn should_clip_at_or_after_stichtag_returns_true() {
        let active_from = Some(d(2026, Month::August, 1));
        assert!(should_clip(d(2026, Month::August, 1), active_from));
        assert!(should_clip(d(2026, Month::August, 2), active_from));
        assert!(should_clip(d(2027, Month::January, 1), active_from));
    }

    #[test]
    fn resolve_active_from_for_week_delegates_to_should_clip() {
        let active_from = Some(d(2026, Month::August, 1));
        assert!(resolve_active_from_for_week(
            2026,
            31,
            DayOfWeek::Saturday,
            active_from
        ));
        assert!(!resolve_active_from_for_week(
            2026,
            31,
            DayOfWeek::Friday,
            active_from
        ));
        assert!(!resolve_active_from_for_week(
            2026,
            31,
            DayOfWeek::Saturday,
            None
        ));
    }

    #[test]
    fn resolve_active_from_for_week_returns_false_on_invalid_week() {
        let active_from = Some(d(2025, Month::January, 1));
        assert!(!resolve_active_from_for_week(
            2025,
            53,
            DayOfWeek::Monday,
            active_from
        ));
    }
}
