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

/// F4 (REB-AUTO-04): Backfill-Request. `from` ist eine ISO-Woche im Format
/// `YYYY-Www` (z.B. `2026-W20`); `dry_run` steuert Vorschau (true, kein Write)
/// vs. Apply (false, persistiert `AutoCronBackfill`-Batches). Wird aus den
/// Query-Parametern deserialisiert; der Handler validiert `from` zusaetzlich
/// gegen ein gueltiges ISO-Wochen-Muster (malformed → 400) BEVOR der Service
/// laeuft.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct BackfillRequestTO {
    /// ISO-Woche im Format `YYYY-Www` (z.B. `2026-W20`).
    pub from: String,
    /// `true` = Vorschau (kein Write); `false` = Apply (persistiert Batches).
    pub dry_run: bool,
}

/// F4 (REB-AUTO-04): Ein Kandidaten-Item der Backfill-Vorschau. Spiegelt
/// `service::rebooking_reconciliation::BackfillPreviewItem`. Jedes Feld ist
/// backend-berechnet (Fat-Backend); `skip_reason` ist ein stabiler
/// lowercase-String (`no_excess` | `already_processed` | `week_locked` |
/// `toggle_off`) statt der Domain-Enum-Variante.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct BackfillPreviewItemTO {
    pub sales_person_id: Uuid,
    pub iso_year: u32,
    pub iso_week: u8,
    /// F1-Ist der Woche (`voluntary_ist`).
    pub voluntary_ist: f32,
    /// F2-Soll der Woche (`voluntary_soll`).
    pub voluntary_soll: f32,
    /// Committed-voluntary-Beitrag der Woche (= `voluntary_soll` fuer die
    /// Single-Week-Range).
    pub committed_voluntary: f32,
    /// Der Ueberschuss, der gebucht wuerde (`max(ist - (soll + committed), 0)`).
    pub excess: f32,
    /// `true`, wenn `excess > 0` und die Woche nicht geskippt wuerde.
    pub would_book: bool,
    /// `true`, wenn diese Woche NICHT gebucht wurde/wuerde.
    pub skipped: bool,
    /// Grund fuer `skipped` als stabiler String; `None`, wenn gebucht.
    pub skip_reason: Option<String>,
}

/// F4 (REB-AUTO-04): Backfill-Vorschau-Antwort. `items` ist die (ggf.
/// paginierte) Liste der Kandidaten; `total_candidates` = Gesamtzahl der
/// Kandidaten vor Pagination; `already_processed` = Anzahl der Kandidaten, die
/// wegen eines bereits belegten Slots geskippt wurden (Idempotenz-Diagnose).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct BackfillPreviewTO {
    /// Spiegelt den Request: `true` = kein Write erfolgt.
    pub dry_run: bool,
    /// Die (ggf. paginierte) Kandidatenliste.
    pub items: Vec<BackfillPreviewItemTO>,
    /// Gesamtzahl der Kandidaten VOR Pagination.
    pub total_candidates: usize,
    /// Anzahl der Kandidaten mit `skip_reason == already_processed`.
    pub already_processed: usize,
}

#[cfg(feature = "service-impl")]
impl BackfillPreviewItemTO {
    /// Mappt ein Domain-`BackfillPreviewItem` auf die TO; die `SkipReason`-Enum
    /// wird auf einen stabilen lowercase-String projiziert.
    pub fn from_domain(item: &service::rebooking_reconciliation::BackfillPreviewItem) -> Self {
        Self {
            sales_person_id: item.sales_person_id,
            iso_year: item.iso_year,
            iso_week: item.iso_week,
            voluntary_ist: item.voluntary_ist,
            voluntary_soll: item.voluntary_soll,
            committed_voluntary: item.committed_voluntary,
            excess: item.excess,
            would_book: item.would_book,
            skipped: item.skipped,
            skip_reason: item.skip_reason.map(skip_reason_to_string),
        }
    }
}

/// Stabile lowercase-String-Projektion der Domain-`SkipReason`. Bewusst
/// ausserhalb von `From`, damit sowohl `BackfillPreviewItemTO::from_domain` als
/// auch die `already_processed`-Zaehlung denselben Namen teilen.
#[cfg(feature = "service-impl")]
fn skip_reason_to_string(reason: service::rebooking_reconciliation::SkipReason) -> String {
    use service::rebooking_reconciliation::SkipReason;
    match reason {
        SkipReason::NoExcess => "no_excess",
        SkipReason::AlreadyProcessed => "already_processed",
        SkipReason::WeekLocked => "week_locked",
        SkipReason::ToggleOff => "toggle_off",
    }
    .to_string()
}

#[cfg(feature = "service-impl")]
impl BackfillPreviewTO {
    /// Baut die Vorschau-TO aus dem Domain-`Vec<BackfillPreviewItem>`.
    /// `total_candidates` = `items.len()` (vor Pagination), `already_processed`
    /// = Anzahl der `SkipReason::AlreadyProcessed`. Die Pagination-Slicing der
    /// `items` erfolgt im Handler NACH diesem Mapping; `total_candidates` bleibt
    /// die ungeschnittene Gesamtzahl.
    pub fn from_domain(
        items: &[service::rebooking_reconciliation::BackfillPreviewItem],
        dry_run: bool,
    ) -> Self {
        use service::rebooking_reconciliation::SkipReason;
        let already_processed = items
            .iter()
            .filter(|i| i.skip_reason == Some(SkipReason::AlreadyProcessed))
            .count();
        Self {
            dry_run,
            items: items.iter().map(BackfillPreviewItemTO::from_domain).collect(),
            total_candidates: items.len(),
            already_processed,
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

    #[test]
    fn backfill_request_serde_roundtrip() {
        let req = BackfillRequestTO {
            from: "2026-W20".to_string(),
            dry_run: true,
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: BackfillRequestTO = serde_json::from_str(&json).unwrap();
        assert_eq!(req, back);

        // Preview-TO serialisiert mit stabiler Feld-Reihenfolge und
        // skip_reason als lowercase-String.
        let preview = BackfillPreviewTO {
            dry_run: false,
            items: vec![BackfillPreviewItemTO {
                sales_person_id: Uuid::from_u128(7),
                iso_year: 2026,
                iso_week: 20,
                voluntary_ist: 10.0,
                voluntary_soll: 4.0,
                committed_voluntary: 4.0,
                excess: 6.0,
                would_book: true,
                skipped: false,
                skip_reason: None,
            }],
            total_candidates: 1,
            already_processed: 0,
        };
        let json = serde_json::to_string(&preview).unwrap();
        let back: BackfillPreviewTO = serde_json::from_str(&json).unwrap();
        assert_eq!(preview, back);
    }

    #[cfg(feature = "service-impl")]
    #[test]
    fn backfill_preview_from_domain_maps_and_counts() {
        use service::rebooking_reconciliation::{BackfillPreviewItem, SkipReason};

        let items = vec![
            // Booked candidate (excess, no skip).
            BackfillPreviewItem {
                sales_person_id: Uuid::from_u128(1),
                iso_year: 2026,
                iso_week: 20,
                voluntary_ist: 12.0,
                voluntary_soll: 4.0,
                committed_voluntary: 4.0,
                excess: 8.0,
                would_book: true,
                skipped: false,
                skip_reason: None,
            },
            // Already-processed candidate (idempotency skip).
            BackfillPreviewItem {
                sales_person_id: Uuid::from_u128(2),
                iso_year: 2026,
                iso_week: 21,
                voluntary_ist: 5.0,
                voluntary_soll: 2.0,
                committed_voluntary: 2.0,
                excess: 3.0,
                would_book: false,
                skipped: true,
                skip_reason: Some(SkipReason::AlreadyProcessed),
            },
        ];

        let to = BackfillPreviewTO::from_domain(&items, true);

        assert!(to.dry_run);
        assert_eq!(to.total_candidates, 2);
        assert_eq!(to.already_processed, 1);
        assert_eq!(to.items.len(), 2);

        // Booked item: no skip_reason.
        assert_eq!(to.items[0].sales_person_id, Uuid::from_u128(1));
        assert!(to.items[0].would_book);
        assert_eq!(to.items[0].skip_reason, None);

        // Skipped item: skip_reason projected to the stable lowercase string.
        assert!(to.items[1].skipped);
        assert_eq!(
            to.items[1].skip_reason.as_deref(),
            Some("already_processed")
        );
    }
}
