//! Phase 55 (REB-MANUAL / HR-ALERT, 55-03): Transport-Objekte fuer die
//! Rebooking-Reconciliation-REST-Routes (F3 manuelle Umbuchung + F5-Modell-B
//! Suggest/Claim/Approve/Reject).
//!
//! D-55-Pin (aus 55-02): eigenes Modul statt Aufblaehung von `lib.rs`.
//!
//! Alle DANN-/proposed-Felder sind vom Backend berechnet (Fat-Backend,
//! HR-ALERT-02) — das Frontend rechnet nichts nach. Die From-Mappings von den
//! Domain-Strukturen leben unter `#[cfg(feature = "service-impl")]`, damit das
//! reine Wire-Crate ohne die `service`-Dependency baubar bleibt.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Richtung einer manuellen Umbuchung (D-55-06). Das Modal waehlt die Richtung
/// explizit — kein Delta-Vorzeichen-Ableiten. Spiegelt
/// `service::rebooking_reconciliation::RebookingDirection`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub enum RebookingDirectionTO {
    /// -N VolunteerWork, +N ExtraWork (Standard-Ausgleich fuer gedeckelte
    /// Mitarbeiter im Minus).
    VolunteerToExtra,
    /// -N ExtraWork, +N VolunteerWork (Korrektur ueberzahlter Zusagen).
    ExtraToVolunteer,
}

#[cfg(feature = "service-impl")]
impl From<RebookingDirectionTO> for service::rebooking_reconciliation::RebookingDirection {
    fn from(direction: RebookingDirectionTO) -> Self {
        match direction {
            RebookingDirectionTO::VolunteerToExtra => {
                service::rebooking_reconciliation::RebookingDirection::VolunteerToExtra
            }
            RebookingDirectionTO::ExtraToVolunteer => {
                service::rebooking_reconciliation::RebookingDirection::ExtraToVolunteer
            }
        }
    }
}

#[cfg(feature = "service-impl")]
impl From<service::rebooking_reconciliation::RebookingDirection> for RebookingDirectionTO {
    fn from(direction: service::rebooking_reconciliation::RebookingDirection) -> Self {
        match direction {
            service::rebooking_reconciliation::RebookingDirection::VolunteerToExtra => {
                RebookingDirectionTO::VolunteerToExtra
            }
            service::rebooking_reconciliation::RebookingDirection::ExtraToVolunteer => {
                RebookingDirectionTO::ExtraToVolunteer
            }
        }
    }
}

/// F3-Request-Body (REB-MANUAL-01/02): one-shot manuelle Umbuchung fuer den
/// UNIQUE-Slot `(sales_person_id, iso_year, iso_week)`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ManualRebookingRequestTO {
    pub sales_person_id: Uuid,
    pub direction: RebookingDirectionTO,
    pub hours: f32,
    pub iso_year: u32,
    pub iso_week: u8,
}

/// F5-Suggest/Claim-Request-Body (HR-ALERT-02/04, Modell B): claimt den
/// pending-Batch fuer den UNIQUE-Slot. Bewusste Mutation (Claim), kein
/// GET-Read-Preview.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct RebookingSuggestRequestTO {
    pub sales_person_id: Uuid,
    pub iso_year: u32,
    pub iso_week: u8,
}

/// F5-Approve-Request-Body (HR-ALERT-03): die zu buchenden Stunden. Der Client
/// uebergibt die im Suggest-Response gelieferten `proposed_hours` unveraendert
/// zurueck; der Service transitioniert den pending-Batch und schreibt das
/// -N/+N-Pair mit genau diesem Betrag.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct RebookingApproveRequestTO {
    pub hours: f32,
}

/// Backend-computed IST/DANN-Vorschau fuer den F5-HR-Alert (HR-ALERT-02).
///
/// Alle DANN-Felder sind vom Backend berechnet (kein FE-Arithmetik-Vorgriff):
/// `balance_after = balance + proposed_hours`,
/// `voluntary_ist_after = voluntary_ist - proposed_hours`. Spiegelt
/// `service::rebooking_reconciliation::RebookingSuggestion`; `sales_person_id`
/// stammt aus dem Request-Kontext.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct RebookingSuggestionTO {
    /// Die neu erzeugte (oder bereits offene, idempotent) pending-Batch-id.
    pub batch_id: Uuid,
    pub sales_person_id: Uuid,
    /// Vorgeschlagene Umbuchungs-Stunden = `min(|balance|, voluntary_ist)`.
    pub proposed_hours: f32,
    /// IST: Stundenkonto-Balance vor Umbuchung.
    pub balance: f32,
    /// DANN: Balance nach Umbuchung (`balance + proposed_hours`).
    pub balance_after: f32,
    /// IST: freiwillige Ist-Stunden vor Umbuchung.
    pub voluntary_ist: f32,
    /// DANN: freiwillige Ist-Stunden nach Umbuchung
    /// (`voluntary_ist - proposed_hours`).
    pub voluntary_ist_after: f32,
}

#[cfg(feature = "service-impl")]
impl RebookingSuggestionTO {
    /// Baut die TO aus der Domain-Vorschau + der `sales_person_id` aus dem
    /// Request-Kontext (die Domain-Struktur traegt sie nicht).
    pub fn from_domain(
        suggestion: &service::rebooking_reconciliation::RebookingSuggestion,
        sales_person_id: Uuid,
    ) -> Self {
        Self {
            batch_id: suggestion.pending_rebooking_id,
            sales_person_id,
            proposed_hours: suggestion.proposed_hours,
            balance: suggestion.balance,
            balance_after: suggestion.balance_after,
            voluntary_ist: suggestion.voluntary_ist,
            voluntary_ist_after: suggestion.voluntary_ist_after,
        }
    }
}

/// State eines Rebooking-Batches fuer die Listen-Ansicht (optional
/// `GET /rebooking-suggestions`). Spiegelt
/// `dao::rebooking_batch::RebookingBatchState`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub enum RebookingBatchStateTO {
    Pending,
    Approved,
    Rejected,
    SkippedLocked,
}

#[cfg(feature = "service-impl")]
impl From<dao::rebooking_batch::RebookingBatchState> for RebookingBatchStateTO {
    fn from(state: dao::rebooking_batch::RebookingBatchState) -> Self {
        match state {
            dao::rebooking_batch::RebookingBatchState::Pending => RebookingBatchStateTO::Pending,
            dao::rebooking_batch::RebookingBatchState::Approved => RebookingBatchStateTO::Approved,
            dao::rebooking_batch::RebookingBatchState::Rejected => RebookingBatchStateTO::Rejected,
            dao::rebooking_batch::RebookingBatchState::SkippedLocked => {
                RebookingBatchStateTO::SkippedLocked
            }
        }
    }
}

/// Batch-TO fuer die Listen-Ansicht offener Vorschlaege.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct RebookingBatchTO {
    pub id: Uuid,
    pub sales_person_id: Uuid,
    pub iso_year: u32,
    pub iso_week: u8,
    pub state: RebookingBatchStateTO,
}

#[cfg(feature = "service-impl")]
impl From<&dao::rebooking_batch::RebookingBatchEntity> for RebookingBatchTO {
    fn from(entity: &dao::rebooking_batch::RebookingBatchEntity) -> Self {
        Self {
            id: entity.id,
            sales_person_id: entity.sales_person_id,
            iso_year: entity.iso_year,
            iso_week: entity.iso_week,
            state: entity.state.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rebooking_to_serde_roundtrip() {
        // ManualRebookingRequestTO roundtrip.
        let manual = ManualRebookingRequestTO {
            sales_person_id: Uuid::from_u128(1),
            direction: RebookingDirectionTO::VolunteerToExtra,
            hours: 3.5,
            iso_year: 2026,
            iso_week: 30,
        };
        let json = serde_json::to_string(&manual).unwrap();
        let back: ManualRebookingRequestTO = serde_json::from_str(&json).unwrap();
        assert_eq!(manual, back);

        // RebookingSuggestionTO roundtrip (alle IST/DANN-Felder).
        let suggestion = RebookingSuggestionTO {
            batch_id: Uuid::from_u128(2),
            sales_person_id: Uuid::from_u128(1),
            proposed_hours: 2.0,
            balance: -2.0,
            balance_after: 0.0,
            voluntary_ist: 5.0,
            voluntary_ist_after: 3.0,
        };
        let json = serde_json::to_string(&suggestion).unwrap();
        let back: RebookingSuggestionTO = serde_json::from_str(&json).unwrap();
        assert_eq!(suggestion, back);
    }
}
