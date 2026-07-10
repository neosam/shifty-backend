//! State-Layer fuer Phase 55 F3+F5 Rebooking-Modals.
//!
//! Thin `From<&…TO>`-Mapper — **keine FE-Arithmetik** (D-55-03,
//! Fat-Backend, MEMORY `feedback_fat_backend_thin_client`). Alle
//! IST/DANN/Delta-Felder kommen 1:1 aus dem Backend.
//!
//! Der `RebookingSubmitError` wird von den Modal-Loadern befuellt, wenn
//! Backend einen strukturierten 409-Body liefert (T-4/T-55-01 Mitigation
//! aus Plan 55-02) und der Modal-Component eine inline-Warn-Section
//! rendert (MEMORY `feedback_warnings_inline_not_dialog`).
//!
//! `#![allow(dead_code)]`: Manche Felder von `RebookingBatch` /
//! `RebookingBatchState` werden vom Wire-Vertrag benoetigt (Response-
//! Deserialisierung + Debug), aber vom FE nicht ausgelesen. Ohne den
//! Allow triggert `-D warnings` ein Dead-Code-False-Positive fuer die
//! reinen Wire-Only-Felder.
#![allow(dead_code)]

use rest_types::{
    ManualRebookingRequestTO, RebookingBatchStateTO, RebookingBatchTO, RebookingDirectionTO,
    RebookingSuggestionTO,
};
use uuid::Uuid;

/// Phase 55 (HR-ALERT-02, D-55-03): Backend-berechnete IST/DANN-Werte
/// fuer eine Pending-Rebooking-Suggestion.
///
/// Alle Delta-Felder sind **Backend-computed** — der FE-Component darf
/// `voluntary_ist - voluntary_soll` NICHT selbst rechnen.
#[derive(Clone, Debug, PartialEq)]
pub struct RebookingSuggestion {
    pub batch_id: Uuid,
    pub sales_person_id: Uuid,
    pub iso_year: u32,
    pub iso_week: u8,
    pub proposed_hours: f32,
    pub balance_before: f32,
    pub voluntary_ist_before: f32,
    pub voluntary_soll_before: f32,
    /// Backend-computed = `voluntary_ist_before - voluntary_soll_before`.
    pub voluntary_delta_before: f32,
    pub balance_after: f32,
    pub voluntary_ist_after: f32,
    pub voluntary_soll_after: f32,
    /// Backend-computed = `voluntary_ist_after - voluntary_soll_after`.
    pub voluntary_delta_after: f32,
}

impl From<&RebookingSuggestionTO> for RebookingSuggestion {
    fn from(to: &RebookingSuggestionTO) -> Self {
        Self {
            batch_id: to.batch_id,
            sales_person_id: to.sales_person_id,
            iso_year: to.iso_year,
            iso_week: to.iso_week,
            proposed_hours: to.proposed_hours,
            balance_before: to.balance_before,
            voluntary_ist_before: to.voluntary_ist_before,
            voluntary_soll_before: to.voluntary_soll_before,
            voluntary_delta_before: to.voluntary_delta_before,
            balance_after: to.balance_after,
            voluntary_ist_after: to.voluntary_ist_after,
            voluntary_soll_after: to.voluntary_soll_after,
            voluntary_delta_after: to.voluntary_delta_after,
        }
    }
}

/// Phase 55 D-55-06: Richtung eines Rebooking-Batches. Bidirektional
/// mit `RebookingDirectionTO`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RebookingDirection {
    VolunteerToExtra,
    ExtraToVolunteer,
}

impl From<&RebookingDirectionTO> for RebookingDirection {
    fn from(to: &RebookingDirectionTO) -> Self {
        match to {
            RebookingDirectionTO::VolunteerToExtra => Self::VolunteerToExtra,
            RebookingDirectionTO::ExtraToVolunteer => Self::ExtraToVolunteer,
        }
    }
}

impl From<RebookingDirection> for RebookingDirectionTO {
    fn from(dir: RebookingDirection) -> Self {
        match dir {
            RebookingDirection::VolunteerToExtra => Self::VolunteerToExtra,
            RebookingDirection::ExtraToVolunteer => Self::ExtraToVolunteer,
        }
    }
}

/// Phase 55 D-55-05 + D-55-06: HR-gewaehltes Wochen/Richtung/Menge-Payload
/// fuer `POST /rebooking/manual`.
#[derive(Clone, Debug, PartialEq)]
pub struct ManualRebookingRequest {
    pub sales_person_id: Uuid,
    pub iso_year: u32,
    pub iso_week: u8,
    pub direction: RebookingDirection,
    pub hours: f32,
}

impl From<&ManualRebookingRequest> for ManualRebookingRequestTO {
    fn from(req: &ManualRebookingRequest) -> Self {
        Self {
            sales_person_id: req.sales_person_id,
            iso_year: req.iso_year,
            iso_week: req.iso_week,
            direction: req.direction.into(),
            hours: req.hours,
        }
    }
}

/// Phase 54 DAO-Enum, gespiegelt auf FE. Bidirektional mit
/// `RebookingBatchStateTO`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RebookingBatchState {
    Pending,
    Approved,
    Rejected,
    SkippedLocked,
}

impl From<&RebookingBatchStateTO> for RebookingBatchState {
    fn from(to: &RebookingBatchStateTO) -> Self {
        match to {
            RebookingBatchStateTO::Pending => Self::Pending,
            RebookingBatchStateTO::Approved => Self::Approved,
            RebookingBatchStateTO::Rejected => Self::Rejected,
            RebookingBatchStateTO::SkippedLocked => Self::SkippedLocked,
        }
    }
}

/// Wire-Repraesentation eines RebookingBatch — Response von
/// `POST /rebooking/manual`, `POST /rebooking-suggestions/{id}/approve|reject`.
#[derive(Clone, Debug, PartialEq)]
pub struct RebookingBatch {
    pub id: Uuid,
    pub sales_person_id: Uuid,
    pub iso_year: u32,
    pub iso_week: u8,
    pub state: RebookingBatchState,
}

impl From<&RebookingBatchTO> for RebookingBatch {
    fn from(to: &RebookingBatchTO) -> Self {
        Self {
            id: to.id,
            sales_person_id: to.sales_person_id,
            iso_year: to.iso_year,
            iso_week: to.iso_week,
            state: (&to.state).into(),
        }
    }
}

/// Modal-spezifischer Fehlertyp fuer `POST /rebooking/manual` +
/// approve/reject: 409-Bodies kommen als strukturierter JSON-Payload
/// mit `error`-Feld (Plan 55-02, T-4/T-55-01 Mitigation).
///
/// Der Modal-Component branched auf die Variante und rendert die
/// zugehoerige i18n-Warn-Section inline (MEMORY
/// `feedback_warnings_inline_not_dialog`).
#[derive(Clone, Debug, PartialEq)]
pub enum RebookingSubmitError {
    /// HTTP 409 mit Body `{"error":"RebookingErrorSlotTaken"}`.
    SlotTaken,
    /// HTTP 409 mit Body `{"error":"RebookingErrorAlreadyResolved"}`.
    AlreadyResolved,
    /// Alle anderen Fehler (Netzwerk, 4xx/5xx ohne bekannten
    /// i18n-Key, 400 mit Validation-Body). Der wrapping-String ist der
    /// Rohtext — der Modal-Component ersetzt ihn typischerweise durch
    /// einen generischen i18n-Warn-Text.
    Other(String),
}
