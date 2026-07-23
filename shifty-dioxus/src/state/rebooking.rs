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
#[allow(dead_code)] // reason: constructed by loader::open_rebooking_suggestion, consumed by rebooking_suggestion_modal (Task 3) + page (55-05)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RebookingSuggestion {
    /// Die pending-Batch-id (fuer Approve/Reject-Routen).
    pub batch_id: Uuid,
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
}
