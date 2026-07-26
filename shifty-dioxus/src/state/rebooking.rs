//! Phase 55 (REB-MANUAL / HR-ALERT): thin Frontend-State-Mapper fuer die
//! Rebooking-Reconciliation-TOs (F3 manuelle Umbuchung + F5 Suggest/Approve/Reject).
//!
//! **Fat Backend, Thin Client (bindend, 55-UI-SPEC.md):** Alle IST/DANN-Werte
//! (`balance`, `balance_after`, `voluntary_ist`, `voluntary_ist_after`,
//! `proposed_hours`) kommen backend-fertig aus `RebookingSuggestionTO`
//! (55-03). Dieser Mapper kopiert nur Felder — **keine Arithmetik**. Die
//! DANN-Faerbungsregeln im Modal lesen die Werte, rechnen sie aber nicht neu.

use uuid::Uuid;

use rest_types::rebooking::{RebookingBatchTO, RebookingDirectionTO, RebookingSuggestionTO};

/// Richtung einer manuellen Umbuchung (F3, D-55-06). Spiegelt
/// `rest_types::rebooking::RebookingDirectionTO` 1:1 fuer das Manual-Modal-Select.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RebookingDirection {
    /// Standard-Ausgleich fuer gedeckelte Mitarbeiter im Minus (Default).
    #[default]
    VolunteerToExtra,
    /// Korrektur ueberzahlter Zusagen.
    ExtraToVolunteer,
}

impl From<RebookingDirection> for RebookingDirectionTO {
    fn from(direction: RebookingDirection) -> Self {
        match direction {
            RebookingDirection::VolunteerToExtra => RebookingDirectionTO::VolunteerToExtra,
            RebookingDirection::ExtraToVolunteer => RebookingDirectionTO::ExtraToVolunteer,
        }
    }
}

impl From<&RebookingDirectionTO> for RebookingDirection {
    fn from(direction: &RebookingDirectionTO) -> Self {
        match direction {
            RebookingDirectionTO::VolunteerToExtra => RebookingDirection::VolunteerToExtra,
            RebookingDirectionTO::ExtraToVolunteer => RebookingDirection::ExtraToVolunteer,
        }
    }
}

/// Frontend-Spiegel der backend-berechneten IST/DANN-Vorschau (HR-ALERT-02).
///
/// Reines Feld-Kopieren aus `RebookingSuggestionTO` — die DANN-Felder
/// (`balance_after`, `voluntary_ist_after`) sind bereits vom Backend gerechnet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RebookingSuggestion {
    /// Die pending-Batch-id (fuer Approve/Reject-Routen).
    pub batch_id: Uuid,
    // reason: field is populated from the TO for completeness/roundtrip parity, but
    // approve/reject key off `batch_id`; no read site consumes the sales-person id yet.
    #[allow(dead_code)]
    pub sales_person_id: Uuid,
    /// Vorgeschlagene Umbuchungs-Stunden (backend: `min(|balance|, voluntary_ist)`).
    pub proposed_hours: f32,
    /// IST: Stundenkonto-Balance vor Umbuchung.
    pub balance: f32,
    /// DANN: Balance nach Umbuchung (backend-computed).
    pub balance_after: f32,
    /// IST: freiwillige Ist-Stunden vor Umbuchung.
    pub voluntary_ist: f32,
    /// DANN: freiwillige Ist-Stunden nach Umbuchung (backend-computed).
    pub voluntary_ist_after: f32,
}

impl From<&RebookingSuggestionTO> for RebookingSuggestion {
    fn from(to: &RebookingSuggestionTO) -> Self {
        // Pure field copy — no arithmetic (Fat Backend, Thin Client).
        Self {
            batch_id: to.batch_id,
            sales_person_id: to.sales_person_id,
            proposed_hours: to.proposed_hours,
            balance: to.balance,
            balance_after: to.balance_after,
            voluntary_ist: to.voluntary_ist,
            voluntary_ist_after: to.voluntary_ist_after,
        }
    }
}

/// Frontend-Spiegel eines Rebooking-Batches (State-Zeile). Reines Feld-Kopieren.
/// Reserviert fuer eine spaetere Batch-Listen-Ansicht (analog `RebookingBatchTO`,
/// das list-only ist) — noch kein Konsument in Phase 55.
#[allow(dead_code)] // reason: reserved for a future rebooking-batch list view (no consumer in Phase 55; mirrors list-only RebookingBatchTO)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RebookingBatch {
    pub id: Uuid,
    pub sales_person_id: Uuid,
    pub iso_year: u32,
    pub iso_week: u8,
}

impl From<&RebookingBatchTO> for RebookingBatch {
    fn from(to: &RebookingBatchTO) -> Self {
        Self {
            id: to.id,
            sales_person_id: to.sales_person_id,
            iso_year: to.iso_year,
            iso_week: to.iso_week,
        }
    }
}

// ─── F4 Backfill (Phase 56-05, REB-AUTO-04) ──────────────────────────────────
//
// Der Backfill-Card sendet eine ISO-Woche `YYYY-Www` (z.B. `2026-W20`) und
// rendert die backend-berechnete `BackfillPreviewTO` verbatim (Fat Backend, Thin
// Client). Die einzige FE-Logik ist die Eingabe-Validitaet des `from`-Feldes —
// bewusst als reine, testbare Funktion extrahiert (analog
// `settings::is_valid_shortday_date_input`), damit die Button-Freigabe-Regel
// unabhaengig vom Browser geprueft werden kann.

/// Validiert eine ISO-Wochen-Eingabe im Format `YYYY-Www` (z.B. `2026-W20`).
///
/// Contract (rein syntaktisch — die endgueltige Bereichs-/53-Wochen-Pruefung
/// macht das Backend, T-56-11):
/// - Leerer String → `false` (der Backfill-Button bleibt gesperrt, bis eine
///   Woche eingegeben ist; anders als beim optionalen Stichtag-Datum).
/// - `YYYY-Www` mit 4-stelligem Jahr, Literal `-W` und 2-stelliger Woche
///   `01..=53` → `true`.
/// - Alles andere (deutsche Notation, Klartext, fehlendes `W`, Woche `00`/`>53`)
///   → `false`.
///
/// Bewusst KEINE Arithmetik/Umrechnung — nur Format-Gate fuer die Button-Freigabe.
pub fn is_valid_iso_week_input(s: &str) -> bool {
    let bytes = s.as_bytes();
    // Exact shape "YYYY-Www": 8 chars, digits + "-W" + 2 digits.
    if bytes.len() != 8 {
        return false;
    }
    if !bytes[0..4].iter().all(|b| b.is_ascii_digit()) {
        return false;
    }
    if bytes[4] != b'-' || bytes[5] != b'W' {
        return false;
    }
    if !bytes[6..8].iter().all(|b| b.is_ascii_digit()) {
        return false;
    }
    // Week 01..=53 (upper bound stays permissive; the backend clamps to
    // weeks_in_year, T-56-11).
    let week: u8 = s[6..8].parse().unwrap_or(0);
    (1..=53).contains(&week)
}

/// Frontend-Render-Spiegel eines `BackfillPreviewItemTO` (REB-AUTO-04). Reines
/// Feld-Kopieren — KEINE Arithmetik (Fat Backend, Thin Client). Existiert, damit
/// das Card-Rendering ueber einen stabilen Typ iteriert statt direkt ueber die
/// Wire-TO; alle Werte kommen backend-fertig.
#[derive(Debug, Clone, PartialEq)]
pub struct BackfillPreviewItem {
    pub sales_person_id: Uuid,
    pub iso_year: u32,
    pub iso_week: u8,
    pub voluntary_ist: f32,
    pub voluntary_soll: f32,
    pub committed_voluntary: f32,
    pub excess: f32,
    pub would_book: bool,
    pub skipped: bool,
    pub skip_reason: Option<String>,
}

impl From<&rest_types::rebooking::BackfillPreviewItemTO> for BackfillPreviewItem {
    fn from(to: &rest_types::rebooking::BackfillPreviewItemTO) -> Self {
        // Pure field copy — no arithmetic (Fat Backend, Thin Client).
        Self {
            sales_person_id: to.sales_person_id,
            iso_year: to.iso_year,
            iso_week: to.iso_week,
            voluntary_ist: to.voluntary_ist,
            voluntary_soll: to.voluntary_soll,
            committed_voluntary: to.committed_voluntary,
            excess: to.excess,
            would_book: to.would_book,
            skipped: to.skipped,
            skip_reason: to.skip_reason.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rest_types::rebooking::RebookingBatchStateTO;

    #[test]
    fn suggestion_mapper_is_pure_field_copy() {
        let to = RebookingSuggestionTO {
            batch_id: Uuid::from_u128(7),
            sales_person_id: Uuid::from_u128(3),
            proposed_hours: 2.0,
            balance: -2.0,
            balance_after: 0.0,
            voluntary_ist: 5.0,
            voluntary_ist_after: 3.0,
        };
        let s = RebookingSuggestion::from(&to);
        // Jedes Feld 1:1 uebernommen — keine Ableitung/Rechnung.
        assert_eq!(s.batch_id, to.batch_id);
        assert_eq!(s.sales_person_id, to.sales_person_id);
        assert_eq!(s.proposed_hours, to.proposed_hours);
        assert_eq!(s.balance, to.balance);
        assert_eq!(s.balance_after, to.balance_after);
        assert_eq!(s.voluntary_ist, to.voluntary_ist);
        assert_eq!(s.voluntary_ist_after, to.voluntary_ist_after);
    }

    #[test]
    fn direction_roundtrips_through_to() {
        for dir in [
            RebookingDirection::VolunteerToExtra,
            RebookingDirection::ExtraToVolunteer,
        ] {
            let to: RebookingDirectionTO = dir.into();
            let back: RebookingDirection = (&to).into();
            assert_eq!(dir, back);
        }
    }

    #[test]
    fn direction_default_is_volunteer_to_extra() {
        assert_eq!(
            RebookingDirection::default(),
            RebookingDirection::VolunteerToExtra
        );
    }

    #[test]
    fn batch_mapper_copies_fields() {
        let to = RebookingBatchTO {
            id: Uuid::from_u128(1),
            sales_person_id: Uuid::from_u128(2),
            iso_year: 2026,
            iso_week: 30,
            state: RebookingBatchStateTO::Pending,
        };
        let b = RebookingBatch::from(&to);
        assert_eq!(b.id, to.id);
        assert_eq!(b.sales_person_id, to.sales_person_id);
        assert_eq!(b.iso_year, 2026);
        assert_eq!(b.iso_week, 30);
    }

    #[test]
    fn is_valid_iso_week_input_accepts_wellformed() {
        assert!(is_valid_iso_week_input("2026-W20"));
        assert!(is_valid_iso_week_input("2026-W01"));
        // Permissive upper bound (backend clamps to weeks_in_year).
        assert!(is_valid_iso_week_input("2026-W53"));
    }

    #[test]
    fn is_valid_iso_week_input_rejects_malformed() {
        assert!(!is_valid_iso_week_input(""));
        assert!(!is_valid_iso_week_input("2026-20")); // missing W
        assert!(!is_valid_iso_week_input("2026-W0")); // week not 2 digits
        assert!(!is_valid_iso_week_input("2026-W00")); // week 0 out of range
        assert!(!is_valid_iso_week_input("2026-W54")); // week > 53
        assert!(!is_valid_iso_week_input("26-W20")); // year not 4 digits
        assert!(!is_valid_iso_week_input("2026-W020")); // too long
        assert!(!is_valid_iso_week_input("08.01.2026")); // German date notation
        assert!(!is_valid_iso_week_input("next week")); // natural language
    }

    #[test]
    fn backfill_item_mapper_is_pure_field_copy() {
        let to = rest_types::rebooking::BackfillPreviewItemTO {
            sales_person_id: Uuid::from_u128(9),
            iso_year: 2026,
            iso_week: 20,
            voluntary_ist: 12.0,
            voluntary_soll: 4.0,
            committed_voluntary: 4.0,
            excess: 8.0,
            would_book: true,
            skipped: false,
            skip_reason: None,
        };
        let item = BackfillPreviewItem::from(&to);
        // Every field copied 1:1 — no derivation/arithmetic.
        assert_eq!(item.sales_person_id, to.sales_person_id);
        assert_eq!(item.iso_year, to.iso_year);
        assert_eq!(item.iso_week, to.iso_week);
        assert_eq!(item.voluntary_ist, to.voluntary_ist);
        assert_eq!(item.voluntary_soll, to.voluntary_soll);
        assert_eq!(item.committed_voluntary, to.committed_voluntary);
        assert_eq!(item.excess, to.excess);
        assert_eq!(item.would_book, to.would_book);
        assert_eq!(item.skipped, to.skipped);
        assert_eq!(item.skip_reason, to.skip_reason);
    }

    #[test]
    fn backfill_item_mapper_preserves_skip_reason() {
        let to = rest_types::rebooking::BackfillPreviewItemTO {
            sales_person_id: Uuid::from_u128(2),
            iso_year: 2026,
            iso_week: 21,
            voluntary_ist: 5.0,
            voluntary_soll: 2.0,
            committed_voluntary: 2.0,
            excess: 0.0,
            would_book: false,
            skipped: true,
            skip_reason: Some("already_processed".to_string()),
        };
        let item = BackfillPreviewItem::from(&to);
        assert!(item.skipped);
        assert_eq!(item.skip_reason.as_deref(), Some("already_processed"));
    }
}
