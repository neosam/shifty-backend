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

use service::{
    ServiceError,
    permission::Authentication,
    slot::Slot,
    special_days::{SpecialDay, SpecialDayType},
    toggle::ToggleService,
};
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

/// Liest den Stichtag-Toggle einmal und toleriert `Unauthorized` als
/// "Gate inaktiv / Legacy-Verhalten" — analog HCFG-02 in `reporting.rs:164-172`.
///
/// **Warum die Toleranz nötig ist:** Die vier Aggregat-Konsumenten (Chain A'
/// Block, Chain B Shiftplan, Chain C BookingInformation, Chain D
/// ShiftplanReport) werden aus unterschiedlichen Auth-Kontexten aufgerufen:
///
/// - Vom REST-Layer mit vollem User-Kontext (Standardfall) → Toggle-DB-Lookup
///   ergibt `Ok(raw)`.
/// - Aus anderen Services mit `Authentication::Full` (z. B. `BookingInformation`
///   ruft `ShiftplanReport` mit `Full` auf). `ToggleService::get_toggle_value`
///   verlangt aber eine echte User-ID; `Authentication::Full` liefert
///   `current_user_id → Ok(None)` → `Err(Unauthorized)`.
/// - Aus mock-auth Development-Setups + Integration-Tests, ebenfalls ohne
///   echte User-ID.
///
/// In allen drei Fällen ist "kein Stichtag konfiguriert" semantisch identisch
/// mit "kein Stichtag gesetzt" → `Ok(None)` → Legacy-Verhalten (kein Clip).
/// Ohne diese Toleranz schlagen ansonsten funktionierende Endpoints (z. B.
/// `GET /report/week/{year}/{week}`, `GET /booking-information/weekly-resource-report/{year}`)
/// mit HTTP 401 fehl.
///
/// # Ablauf im Konsumenten (Wave 2 / Gap-Closure)
///
/// ```ignore
/// let active_from = shortday_gate::read_active_from(
///     self.toggle_service.as_ref(),
///     context.clone(),
/// ).await?;
/// // active_from == None → Gate inaktiv → Legacy
/// // active_from == Some(date) → Gate greift ab date (inklusiv)
/// ```
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

/// Ergebnis des pro-Slot-Clips (Phase 51, D-51-06 Chain A' / Chain C).
///
/// - `Keep(slot)` — Slot bleibt (roh oder geclippt).
/// - `Drop` — Slot fällt ganz weg (Cutoff ≤ `slot.from`; D-04 Zeile 3).
pub(crate) enum ClipOutcome {
    Keep(Slot),
    Drop,
}

/// Behandlung des Slots, wenn das Stichtag-Gate **inaktiv** ist
/// (`active_from == None` oder `booking_date < active_from`).
///
/// - `Modern` — Slot bleibt roh und ungefiltert (aktueller Status Chain A'/D).
///   Verwendung: `block.rs` (Chain A'), `shiftplan_report.rs` (Chain D).
/// - `Legacy` — Legacy-Filter-Semantik (Pre-Phase-51): existiert ein `ShortDay`
///   für den Wochentag, wird der Slot **verworfen**, sobald `slot.to > cutoff`.
///   Endet er spätestens am Cutoff (`slot.to <= cutoff`), bleibt er roh drin.
///   Verwendung: `shiftplan.rs` (Chain B), `booking_information.rs` (Chain C).
///
/// Historische Herleitung siehe Gap-Closure Phase 51:
/// - Chain B alt (Commit `8d12645^`, `shiftplan.rs:62-66`):
///   `if slot.to > cutoff { continue; }`
/// - Chain C alt (Commit `62a2f35^`, `booking_information.rs:394-401`):
///   `.filter(|slot| !special_days.iter().any(|day| day.day_of_week == slot.day_of_week
///                   && (Holiday || (ShortDay && cutoff && slot.to > cutoff))))`
///
/// Wenn das Gate **aktiv** ist (`booking_date >= active_from`), ist der Modus
/// irrelevant — es wird immer via `Slot::clip_to` geclippt (D-04, D-51-09).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ShortdayMode {
    /// Chain A' / Chain D: Gate aus + ShortDay → Slot bleibt trotzdem roh.
    Modern,
    /// Chain B / Chain C: Gate aus + ShortDay → Slot droppen wenn
    /// `slot.to > cutoff` (Pre-Phase-51-Verhalten).
    Legacy,
}

/// Wendet den ShortDay-Cutoff pro Wochentag + Stichtag-Gate auf einen Slot an.
///
/// Zentraler Helper für alle Aggregat-Konsumenten (Chain A' Block, Chain B
/// Shiftplan, Chain C BookingInformation, Chain D ShiftplanReport). Keine
/// DB-Zugriffe — reine In-Memory-Kombi aus `special_days`-Snapshot pro Woche,
/// ISO-Datum-Konstruktion und [`Slot::clip_to`] (P01).
///
/// # Semantik
///
/// - **Gate aktiv** (`booking_date >= active_from`): Slot wird via `clip_to`
///   geclippt (D-04, D-51-09). Modus egal.
/// - **Gate inaktiv** (`active_from == None` oder `booking_date < active_from`):
///   - Kein `ShortDay` für den Wochentag → `Keep(raw)`.
///   - `ShortDay` mit cutoff für den Wochentag:
///     - `mode == Modern` → `Keep(raw)` (aktuelles Verhalten Chain A'/D).
///     - `mode == Legacy` → Wenn `slot.to > cutoff` → `Drop`; sonst `Keep(raw)`
///       (historisches Filter-Verhalten Chain B/C, Gap-Closure Phase 51).
pub(crate) fn clip_slot_for_week(
    slot: &Slot,
    special_days: &[SpecialDay],
    year: u32,
    week: u8,
    active_from: Option<Date>,
    mode: ShortdayMode,
) -> ClipOutcome {
    // Cutoff aus SpecialDay (nur ShortDay mit `time_of_day`) für diesen dow —
    // wird sowohl im aktiven als auch im Legacy-Zweig gebraucht.
    let cutoff = special_days.iter().find_map(|sd| {
        if sd.day_of_week == slot.day_of_week && sd.day_type == SpecialDayType::ShortDay {
            sd.time_of_day
        } else {
            None
        }
    });

    // Stichtag-Gate: greift das Gate für diesen Wochentag überhaupt?
    let gate_active = resolve_active_from_for_week(year, week, slot.day_of_week, active_from);
    if !gate_active {
        // Gate aus. Modus entscheidet über Legacy-Filter.
        return match (mode, cutoff) {
            // Modern: immer Keep(raw) — unabhängig davon, ob ShortDay existiert.
            (ShortdayMode::Modern, _) => ClipOutcome::Keep(slot.clone()),
            // Legacy ohne ShortDay: Keep(raw).
            (ShortdayMode::Legacy, None) => ClipOutcome::Keep(slot.clone()),
            // Legacy mit ShortDay: Pre-Phase-51-Filter — Drop wenn slot.to > cutoff.
            (ShortdayMode::Legacy, Some(cutoff)) => {
                if slot.to > cutoff {
                    ClipOutcome::Drop
                } else {
                    ClipOutcome::Keep(slot.clone())
                }
            }
        };
    }

    // Gate aktiv → Standard-Clip (D-04, D-51-09).
    let Some(cutoff) = cutoff else {
        return ClipOutcome::Keep(slot.clone());
    };

    match slot.clip_to(cutoff) {
        Some(clipped) => ClipOutcome::Keep(clipped),
        None => ClipOutcome::Drop,
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

    // ─── Gap-Closure Phase 51: Legacy-Filter-Semantik (Chain B/C) ───────────
    //
    // Sanity-Tests direkt am Helper — die Chain-B/C-Integration-Tests
    // (test/shiftplan.rs, test/booking_information_chain_c.rs) prüfen den
    // gleichen Contract oben auf der Aggregat-Ebene.

    use service::slot::Slot;
    use service::special_days::{SpecialDay, SpecialDayType};
    use time::Time;
    use uuid::Uuid;

    fn t(h: u8, m: u8) -> Time {
        Time::from_hms(h, m, 0).expect("valid test time")
    }

    fn slot_for(dow: DayOfWeek, from: Time, to: Time) -> Slot {
        Slot {
            id: Uuid::new_v4(),
            day_of_week: dow,
            from,
            to,
            min_resources: 1,
            max_paid_employees: None,
            valid_from: Date::from_calendar_date(2020, Month::January, 1).unwrap(),
            valid_to: None,
            deleted: None,
            version: Uuid::new_v4(),
            shiftplan_id: None,
        }
    }

    fn shortday_for(dow: DayOfWeek, cutoff: Time, year: u32, week: u8) -> SpecialDay {
        SpecialDay {
            id: Uuid::new_v4(),
            year,
            calendar_week: week,
            day_of_week: dow,
            day_type: SpecialDayType::ShortDay,
            time_of_day: Some(cutoff),
            created: None,
            deleted: None,
            version: Uuid::new_v4(),
        }
    }

    /// Modern-Mode + Gate aus: Slot bleibt roh (aktuelles Chain A'/D-Verhalten).
    #[test]
    fn clip_slot_modern_gate_off_keeps_raw_even_with_shortday() {
        let slot = slot_for(DayOfWeek::Monday, t(14, 0), t(15, 0));
        let sd = shortday_for(DayOfWeek::Monday, t(14, 30), 2026, 31);
        let outcome = clip_slot_for_week(&slot, &[sd], 2026, 31, None, ShortdayMode::Modern);
        match outcome {
            ClipOutcome::Keep(s) => {
                assert_eq!(s.from, t(14, 0));
                assert_eq!(s.to, t(15, 0), "Modern + Gate off → raw slot.to");
            }
            ClipOutcome::Drop => panic!("Modern + Gate off must never drop"),
        }
    }

    /// Legacy-Mode + Gate aus + ShortDay + `slot.to > cutoff` → Slot droppen
    /// (Pre-Phase-51-Semantik: `if slot.to > cutoff { continue }`).
    #[test]
    fn clip_slot_legacy_gate_off_drops_when_slot_to_after_cutoff() {
        // Slot 14:00–15:00, cutoff 14:30 → slot.to (15:00) > 14:30 → Drop.
        let slot = slot_for(DayOfWeek::Monday, t(14, 0), t(15, 0));
        let sd = shortday_for(DayOfWeek::Monday, t(14, 30), 2026, 31);
        let outcome = clip_slot_for_week(&slot, &[sd], 2026, 31, None, ShortdayMode::Legacy);
        assert!(
            matches!(outcome, ClipOutcome::Drop),
            "Legacy + Gate off + slot.to > cutoff must Drop"
        );
    }

    /// Legacy-Mode + Gate aus + ShortDay + `slot.to == cutoff` → Slot bleibt
    /// (endet genau am Cutoff → historisch in slot_hours / slots enthalten).
    #[test]
    fn clip_slot_legacy_gate_off_keeps_when_slot_to_equals_cutoff() {
        // Slot 12:00–14:30, cutoff 14:30 → slot.to == cutoff → Keep(raw).
        let slot = slot_for(DayOfWeek::Monday, t(12, 0), t(14, 30));
        let sd = shortday_for(DayOfWeek::Monday, t(14, 30), 2026, 31);
        let outcome = clip_slot_for_week(&slot, &[sd], 2026, 31, None, ShortdayMode::Legacy);
        match outcome {
            ClipOutcome::Keep(s) => {
                assert_eq!(s.to, t(14, 30), "Legacy + Gate off + slot.to==cutoff → raw");
            }
            ClipOutcome::Drop => {
                panic!("Legacy + Gate off + slot.to == cutoff must Keep, not Drop")
            }
        }
    }

    /// Legacy-Mode + Gate aus + ShortDay + `slot.to < cutoff` → Slot bleibt roh.
    #[test]
    fn clip_slot_legacy_gate_off_keeps_when_slot_ends_before_cutoff() {
        let slot = slot_for(DayOfWeek::Monday, t(9, 0), t(12, 0));
        let sd = shortday_for(DayOfWeek::Monday, t(14, 30), 2026, 31);
        let outcome = clip_slot_for_week(&slot, &[sd], 2026, 31, None, ShortdayMode::Legacy);
        assert!(matches!(outcome, ClipOutcome::Keep(_)));
    }

    /// Legacy-Mode + Gate aus + **kein** ShortDay → Slot bleibt roh
    /// (Legacy-Filter greift nicht ohne ShortDay-Zeile).
    #[test]
    fn clip_slot_legacy_gate_off_keeps_when_no_shortday() {
        let slot = slot_for(DayOfWeek::Monday, t(14, 0), t(15, 0));
        let outcome = clip_slot_for_week(&slot, &[], 2026, 31, None, ShortdayMode::Legacy);
        assert!(matches!(outcome, ClipOutcome::Keep(_)));
    }

    /// Legacy-Mode + Gate **aktiv** + ShortDay-Overlap: verhält sich identisch
    /// zum Modern-Mode (Clip via `Slot::clip_to`, kein Legacy-Drop-Path). Beweis
    /// dass der Modus-Parameter nur den Gate-off-Fall verändert.
    #[test]
    fn clip_slot_legacy_gate_on_behaves_like_modern_clip() {
        let slot = slot_for(DayOfWeek::Monday, t(12, 0), t(15, 0));
        let sd = shortday_for(DayOfWeek::Monday, t(14, 0), 2026, 31);
        // Gate aktiv: active_from lange vor 2026-W31.
        let active = Some(d(2020, Month::January, 1));
        let legacy = clip_slot_for_week(
            &slot,
            std::slice::from_ref(&sd),
            2026,
            31,
            active,
            ShortdayMode::Legacy,
        );
        let modern = clip_slot_for_week(&slot, &[sd], 2026, 31, active, ShortdayMode::Modern);
        match (legacy, modern) {
            (ClipOutcome::Keep(l), ClipOutcome::Keep(m)) => {
                assert_eq!(l.to, t(14, 0));
                assert_eq!(m.to, t(14, 0));
            }
            other => panic!("expected both Keep(clipped), got {:?}", match other {
                (ClipOutcome::Keep(_), ClipOutcome::Drop) => "(Keep, Drop)",
                (ClipOutcome::Drop, ClipOutcome::Keep(_)) => "(Drop, Keep)",
                (ClipOutcome::Drop, ClipOutcome::Drop) => "(Drop, Drop)",
                _ => "unreachable",
            }),
        }
    }
}
