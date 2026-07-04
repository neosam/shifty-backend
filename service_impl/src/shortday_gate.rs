//! Stichtag-Gate für die ShortDay-Slot-Kürzung (Phase 51, D-51-07).
//!
//! Zentraler Ort für die Semantik "ab welchem Datum greift die Slot-Kürzung".
//! Alle vier Aggregat-Ketten aus D-51-06 (Chain A' Block, Chain B Shiftplan,
//! Chain C BookingInformation, Chain D ShiftplanReport) rufen den Toggle
//! selbst via [`ToggleService::get_toggle_value`] auf und übergeben das
//! Ergebnis an [`parse_active_from`] und dann an [`should_clip`], bevor sie
//! die eigentliche `Slot::clip_to`-Fn (aus P01) benutzen.
//!
//! Ablauf im Konsumenten (Wave 2):
//!
//! ```ignore
//! let raw = toggle_service
//!     .get_toggle_value(shortday_gate::TOGGLE_NAME, ctx, None)
//!     .await?;
//! let active_from = shortday_gate::parse_active_from(raw.as_deref());
//! if shortday_gate::should_clip(booking_date, active_from) {
//!     slot.clip_to(cutoff)
//! } else {
//!     Some(slot) // Legacy: ungeclippt
//! }
//! ```
//!
//! Muster übernommen aus `service_impl/src/reporting.rs:164-180`
//! (HCFG-02 `holiday_auto_credit` in v1.7).

use shifty_utils::DayOfWeek;
use time::{Date, format_description::well_known::Iso8601};

/// Name des ToggleService-Eintrags (D-51-07).
///
/// Konsumenten (Wave 2) übergeben diesen an
/// `ToggleService::get_toggle_value(TOGGLE_NAME, ...)` statt einer
/// Magic-String.
pub const TOGGLE_NAME: &str = "shortday_slot_clipping_active_from";

/// Parst den ISO-8601-Stichtag aus dem Toggle-Wert.
///
/// - `None` (Toggle-Wert nicht gesetzt) → `None` (Legacy / Rollout-Default).
/// - `Some("")` → `None`.
/// - `Some(bad)` → `None` (defensiv; Konsumenten fallen in Legacy statt zu
///   crashen — analog `reporting.rs:173-179`).
/// - `Some("2026-08-01")` → `Some(Date{2026-08-01})`.
pub fn parse_active_from(raw: Option<&str>) -> Option<Date> {
    let s = raw?;
    if s.is_empty() {
        return None;
    }
    Date::parse(s, &Iso8601::DEFAULT).ok()
}

/// Entscheidet, ob für ein gegebenes Buchungsdatum die Slot-Kürzung greift.
///
/// Semantik (D-51-07):
/// - `active_from == None` → immer `false` (Kürzung deaktiviert).
/// - `booking_date < active_from` → `false` (historisch, ungeclippt).
/// - `booking_date == active_from` → `true` (inklusiv am Stichtag).
/// - `booking_date > active_from` → `true`.
pub fn should_clip(booking_date: Date, active_from: Option<Date>) -> bool {
    match active_from {
        None => false,
        Some(cutoff) => booking_date >= cutoff,
    }
}

/// Convenience für Konsumenten, die nur ISO-Wochen-Koordinaten haben
/// (Chain A' / Chain B iterieren über ISO-Woche + `DayOfWeek`, nicht über
/// `time::Date`).
///
/// Konstruiert das Buchungsdatum aus `(year, week, day_of_week)` und
/// delegiert an [`should_clip`]. Bei Parse-Fehler (z. B. Woche 53 in einem
/// Nicht-53-Wochen-Jahr) gibt es `false` zurück (defensiver Skip statt
/// Panic).
pub fn resolve_active_from_for_week(
    year: u32,
    week: u8,
    day_of_week: DayOfWeek,
    active_from: Option<Date>,
) -> bool {
    let Ok(booking_date) = Date::from_iso_week_date(year as i32, week, day_of_week.into()) else {
        return false;
    };
    should_clip(booking_date, active_from)
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
        // Rollout-Default: kein Stichtag gesetzt → nie clippen.
        assert!(!should_clip(d(2026, Month::August, 1), None));
        assert!(!should_clip(d(2030, Month::December, 31), None));
    }

    #[test]
    fn should_clip_before_stichtag_returns_false() {
        // Vortag: booking_date = 2026-07-31, active_from = 2026-08-01 → false.
        let active_from = Some(d(2026, Month::August, 1));
        assert!(!should_clip(d(2026, Month::July, 31), active_from));
        assert!(!should_clip(d(2025, Month::January, 1), active_from));
    }

    #[test]
    fn should_clip_at_or_after_stichtag_returns_true() {
        // Grenzfall inklusiv am Stichtag + eindeutig danach.
        let active_from = Some(d(2026, Month::August, 1));
        assert!(should_clip(d(2026, Month::August, 1), active_from));
        assert!(should_clip(d(2026, Month::August, 2), active_from));
        assert!(should_clip(d(2027, Month::January, 1), active_from));
    }

    #[test]
    fn resolve_active_from_for_week_delegates_to_should_clip() {
        // Sanity: die Wochen-Convenience-Fn respektiert dasselbe Gate.
        // 2026-08-01 ist ein Samstag; ISO-Woche 31 / Sa.
        let active_from = Some(d(2026, Month::August, 1));
        assert!(resolve_active_from_for_week(
            2026,
            31,
            DayOfWeek::Saturday,
            active_from
        ));
        // Vortag (2026-07-31, Freitag Wo 31) → false.
        assert!(!resolve_active_from_for_week(
            2026,
            31,
            DayOfWeek::Friday,
            active_from
        ));
        // active_from == None → immer false.
        assert!(!resolve_active_from_for_week(
            2026,
            31,
            DayOfWeek::Saturday,
            None
        ));
    }

    #[test]
    fn resolve_active_from_for_week_returns_false_on_invalid_week() {
        // 2025 hat keine ISO-Woche 53 → Konstruktion schlägt fehl → false
        // (defensiver Skip statt Panic).
        let active_from = Some(d(2025, Month::January, 1));
        assert!(!resolve_active_from_for_week(
            2025,
            53,
            DayOfWeek::Monday,
            active_from
        ));
    }
}
