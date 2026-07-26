//! Stichtag-Gate für die Freiwilligen-Rebooking-Auto-Cron (Phase 56 F4,
//! REB-AUTO-02 / REB-AUTO-05).
//!
//! Zentraler Ort für die Semantik "ab welcher ISO-Woche läuft der
//! automatische Wochen-Cron und bucht Überschüsse als Ausgleich". Der
//! Scheduler-Write-Guard (Plan 02/03) ruft den Toggle selbst via
//! [`ToggleService::get_toggle_value`] auf und übergibt das Ergebnis an
//! [`parse_active_from`] und dann an [`should_run_for_week`], bevor er für
//! eine gegebene ISO-Woche einen Rebooking-Batch (`kind=auto_cron`) erzeugt.
//!
//! Ablauf im Konsumenten (Wave 1 / Plan 02+03):
//!
//! ```ignore
//! let active_from = voluntary_rebooking_gate::read_active_from(
//!     self.toggle_service.as_ref(),
//!     context.clone(),
//! ).await?;
//! if voluntary_rebooking_gate::should_run_for_week(iso_year, iso_week, active_from) {
//!     // Woche verarbeiten (Überschuss als Ausgleich buchen)
//! } else {
//!     // Feature dormant / vor Stichtag -> Skip
//! }
//! ```
//!
//! Muster übernommen aus `service_impl/src/shortday_gate.rs` (Phase 51,
//! D-51-07), adaptiert von Buchungsdatum-Semantik auf ISO-Wochen-Koordinaten:
//! der Cutoff greift, sobald der **Montag** der ISO-Woche auf oder nach dem
//! Stichtag liegt (inklusive untere Grenze, spiegelt
//! `shortday_gate::should_clip`).

use service::{ServiceError, permission::Authentication, toggle::ToggleService};
use time::{Date, Weekday, format_description::well_known::Iso8601};

/// Name des ToggleService-Eintrags (REB-AUTO-05).
///
/// Muss byte-genau dem Toggle-Key in der Migration
/// `20260707000002_seed-voluntary-rebooking-toggle.sql` entsprechen.
/// Konsumenten (Plan 02/03) übergeben diesen an
/// `ToggleService::get_toggle_value(TOGGLE_NAME, ...)` statt einer Magic-String.
pub const TOGGLE_NAME: &str = "voluntary_rebooking_auto_active_from";

/// Parst den ISO-8601-Stichtag aus dem Toggle-Wert.
///
/// - `None` (Toggle-Wert nicht gesetzt) → `None` (Rollout-Default / Feature aus).
/// - `Some("")` → `None`.
/// - `Some(bad)` → `None` (defensiv; Konsumenten fallen in "Feature aus" statt
///   zu crashen — analog `shortday_gate::parse_active_from`).
/// - `Some("2026-08-01")` → `Some(Date{2026-08-01})`.
pub fn parse_active_from(raw: Option<&str>) -> Option<Date> {
    let s = raw?;
    if s.is_empty() {
        return None;
    }
    Date::parse(s, &Iso8601::DEFAULT).ok()
}

/// Entscheidet, ob der Auto-Cron für eine gegebene ISO-Woche laufen soll.
///
/// Semantik (REB-AUTO-05, spiegelt `shortday_gate::should_clip`):
/// - `active_from == None` → immer `false` (Feature dormant, kein Rebooking).
/// - ISO-Woche, deren **Montag** strikt vor `active_from` liegt → `false`
///   (historisch, kein rückwirkendes Auto-Rebooking).
/// - ISO-Woche, deren Montag auf oder nach `active_from` liegt → `true`
///   (inklusive untere Grenze am Stichtag).
///
/// Der Montag der ISO-Woche wird über [`Date::from_iso_week_date`] aufgelöst
/// (nie über eine naive `year()+week()`-Kombination), damit der
/// ISO-Jahresgrenzfall (z. B. Woche 53) korrekt behandelt wird. Schlägt die
/// Auflösung fehl (z. B. Woche 53 in einem Nicht-53-Wochen-Jahr), gibt die fn
/// `false` zurück (defensiver Skip statt Panic).
pub fn should_run_for_week(iso_year: u32, iso_week: u8, active_from: Option<Date>) -> bool {
    let Some(cutoff) = active_from else {
        return false;
    };
    let Ok(monday) = Date::from_iso_week_date(iso_year as i32, iso_week, Weekday::Monday) else {
        return false;
    };
    monday >= cutoff
}

/// Liest den Stichtag-Toggle einmal und toleriert `Unauthorized` als "Feature
/// inaktiv" — analog `shortday_gate::read_active_from`.
///
/// **Warum die Toleranz nötig ist:** Der Scheduler-Write-Guard (Plan 02/03)
/// läuft aus einem Cron-Kontext ohne echte User-ID (`Authentication::Full` →
/// `current_user_id → Ok(None)` → `ToggleService` liefert
/// `Err(Unauthorized)`). Für den Cron ist "kein Stichtag lesbar" semantisch
/// identisch mit "kein Stichtag gesetzt" → `Ok(None)` → Feature aus.
// Wird erst vom Scheduler-Write-Guard (Plan 02/03) konsumiert; in Plan 01
// landet das Gate-Modul vorab (Wave-0-Validierungs-Backbone), daher hier noch
// ungenutzt.
#[allow(dead_code)]
pub(crate) async fn read_active_from<S: ToggleService + ?Sized>(
    toggle_service: &S,
    context: Authentication<S::Context>,
) -> Result<Option<Date>, ServiceError> {
    match toggle_service
        .get_toggle_value(TOGGLE_NAME, context, None)
        .await
    {
        Ok(raw) => Ok(parse_active_from(raw.as_deref())),
        Err(ServiceError::Unauthorized) => Ok(None),
        Err(e) => Err(e),
    }
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
        // Defensiv: schlechter Input crasht nie, fällt in "Feature aus".
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
    fn should_run_none_active_from_is_always_false() {
        // Feature dormant: kein Stichtag gesetzt → nie laufen, für jede Woche.
        assert!(!should_run_for_week(2026, 1, None));
        assert!(!should_run_for_week(2026, 31, None));
        assert!(!should_run_for_week(2030, 52, None));
    }

    #[test]
    fn should_run_week_before_stichtag_is_false() {
        // active_from = 2026-08-03 (Montag KW 32). KW 31/2026 (Mo 2026-07-27)
        // liegt strikt davor → false.
        let active_from = Some(d(2026, Month::August, 3));
        assert!(!should_run_for_week(2026, 31, active_from));
        // Weit vorher.
        assert!(!should_run_for_week(2025, 1, active_from));
    }

    #[test]
    fn should_run_week_at_or_after_stichtag_is_true() {
        // active_from = 2026-08-03 (Montag KW 32). KW 32/2026 (Mo 2026-08-03)
        // liegt inklusiv am Stichtag → true; KW 33 danach → true.
        let active_from = Some(d(2026, Month::August, 3));
        assert!(should_run_for_week(2026, 32, active_from));
        assert!(should_run_for_week(2026, 33, active_from));
        assert!(should_run_for_week(2027, 1, active_from));
    }

    #[test]
    fn should_run_resolves_monday_via_iso_week_date_at_year_boundary() {
        // ISO-Jahresgrenze: 2026 hat eine Woche 53 (Mo 2026-12-28).
        // Mit active_from vor diesem Montag → true; die Auflösung geht über
        // Date::from_iso_week_date, nicht über eine naive year()+week()-Kombi.
        let iso_year = 2026;
        let iso_week = 53;
        // Sanity: KW 53/2026 Montag ist 2026-12-28.
        let monday = Date::from_iso_week_date(iso_year as i32, iso_week, Weekday::Monday)
            .expect("2026 has ISO week 53");
        assert_eq!(monday, d(2026, Month::December, 28));

        // Stichtag genau am Montag der Grenzwoche → true (inklusiv).
        assert!(should_run_for_week(
            iso_year,
            iso_week,
            Some(d(2026, Month::December, 28))
        ));
        // Stichtag einen Tag später → Montag liegt davor → false.
        assert!(!should_run_for_week(
            iso_year,
            iso_week,
            Some(d(2026, Month::December, 29))
        ));

        // Defensiver Skip: 2025 hat keine ISO-Woche 53 → Auflösung schlägt fehl
        // → false statt Panic.
        assert!(!should_run_for_week(2025, 53, Some(d(2025, Month::January, 1))));
    }
}
