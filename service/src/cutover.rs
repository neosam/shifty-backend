//! Phase 4 — Cutover orchestration trait + DTOs.
//!
//! Business-Logic-Tier service per shifty-backend/CLAUDE.md § "Service-Tier-
//! Konventionen". Consumes AbsenceService + ExtraHoursService +
//! CarryoverRebuildService + FeatureFlagService + EmployeeWorkDetailsService +
//! CutoverDao + PermissionService + TransactionDao. NO Sub-Service depends on
//! CutoverService — no cycle.

use std::sync::Arc;

use async_trait::async_trait;
use dao::absence::AbsencePeriodEntity;
use dao::MockTransaction;
use mockall::automock;
use uuid::Uuid;

use crate::absence::AbsenceCategory;
use crate::permission::Authentication;
use crate::ServiceError;

pub const CUTOVER_ADMIN_PRIVILEGE: &str = "cutover_admin";

/// Reasons an extra_hours row landed in the quarantine table.
/// Persisted as snake_case strings; see D-Phase4-03 / specifics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuarantineReason {
    AmountBelowContractHours,
    AmountAboveContractHours,
    ContractHoursZeroForDay,
    ContractNotActiveAtDate,
    Iso53WeekGap,
}

impl QuarantineReason {
    pub fn as_persisted_str(&self) -> &'static str {
        match self {
            Self::AmountBelowContractHours => "amount_below_contract_hours",
            Self::AmountAboveContractHours => "amount_above_contract_hours",
            Self::ContractHoursZeroForDay => "contract_hours_zero_for_day",
            Self::ContractNotActiveAtDate => "contract_not_active_at_date",
            Self::Iso53WeekGap => "iso_53_week_gap",
        }
    }

    /// Human-readable explanation of the reason in **English**. The backend
    /// default; the frontend (Plan 08-08 follow-up) may translate via i18n.
    /// Used by `CutoverQuarantineEntryTO.reason_text` so an HR user can read
    /// the failed-gate response without having to consult external docs.
    pub fn human_text(&self) -> &'static str {
        match self {
            Self::AmountBelowContractHours => {
                "Booked hours are below the employee's contract hours for this day"
            }
            Self::AmountAboveContractHours => {
                "Booked hours exceed the employee's contract hours for this day"
            }
            Self::ContractHoursZeroForDay => {
                "Employee has zero contract hours on this weekday \
                 (e.g. a 4-day-week contract with the booking falling on a non-workday)"
            }
            Self::ContractNotActiveAtDate => {
                "Employee had no active working-hours contract on this date"
            }
            Self::Iso53WeekGap => {
                "Booking falls into ISO calendar week 53, which the new range model does not represent"
            }
        }
    }

    /// Suggested remediation in **English**. Pairs with `human_text` and is
    /// surfaced verbatim in `CutoverQuarantineEntryTO.suggested_action`.
    pub fn suggested_action(&self) -> &'static str {
        match self {
            Self::AmountBelowContractHours | Self::AmountAboveContractHours => {
                "Adjust the entry's hours to match the employee's contract for that day, \
                 or delete the entry if it was a mistake"
            }
            Self::ContractHoursZeroForDay => {
                "Delete the entry, or move it to a workday on which the employee's \
                 contract specifies > 0 hours"
            }
            Self::ContractNotActiveAtDate => {
                "Verify the employee's working-hours contract covers this date, \
                 or delete the entry"
            }
            Self::Iso53WeekGap => {
                "Move the entry to a date inside an ISO calendar week 1-52 \
                 (typically the first or last day of the surrounding week)"
            }
        }
    }
}

/// Single quarantined `extra_hours` row attached to a `DriftRow` in Plan 08-08.
///
/// Holds enough info for the failed-gate REST response to render a
/// human-readable explanation per entry without a separate file lookup:
/// the legacy `extra_hours_id` (so an HR user can `DELETE /extra-hours/{id}`),
/// the booking date (for the front-end weekday code), the booked amount,
/// and the (typed) reason. The `From<&CutoverQuarantineEntry>` impl on the
/// REST DTO does the reason → reason_code/reason_text/suggested_action
/// mapping via `QuarantineReason::{as_persisted_str, human_text, suggested_action}`.
#[derive(Clone, Debug, PartialEq)]
pub struct CutoverQuarantineEntry {
    pub extra_hours_id: Uuid,
    pub date: time::Date,
    pub amount: f32,
    pub reason: QuarantineReason,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DriftRow {
    pub sales_person_id: Uuid,
    pub sales_person_name: Arc<str>,
    pub category: AbsenceCategory,
    pub year: u32,
    pub legacy_sum: f32,
    pub derived_sum: f32,
    pub drift: f32,
    pub quarantined_extra_hours_count: u32,
    pub quarantine_reasons: Arc<[Arc<str>]>,
    /// Plan 08-08: per-entry list of the quarantined `extra_hours` rows that
    /// fall under this `(sales_person, category, year)` drift bucket. Empty
    /// is a valid value (drift may exist without any quarantined entries —
    /// e.g. a derive-vs-legacy formula change in absence of quarantine).
    pub quarantined_entries: Arc<[CutoverQuarantineEntry]>,
}

/// Inline-friendly drift report carried in `CutoverRunResult.gate_drift_report`
/// when the gate fails. Mirrors the on-disk JSON file (under `.planning/
/// migration-backup/cutover-gate-{ts}.json`) but is also returned over HTTP
/// so the file-path detour is optional.
#[derive(Clone, Debug, PartialEq)]
pub struct CutoverGateDriftReport {
    pub gate_run_id: Uuid,
    pub run_at: time::PrimitiveDateTime,
    pub dry_run: bool,
    pub drift_threshold: f32,
    pub total_drift_rows: u32,
    pub drift: Arc<[DriftRow]>,
    pub passed: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GateResult {
    pub passed: bool,
    pub drift_rows: Arc<[DriftRow]>,
    pub diff_report_path: Arc<str>,
    /// Set of (sales_person_id, year) tuples that the gate evaluated, used
    /// downstream by the carryover refresh scope (D-Phase4-12).
    pub scope_set: Arc<[(Uuid, u32)]>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CutoverRunResult {
    pub run_id: Uuid,
    pub ran_at: time::PrimitiveDateTime,
    pub dry_run: bool,
    pub gate_passed: bool,
    pub total_clusters: u32,
    pub migrated_clusters: u32,
    pub quarantined_rows: u32,
    pub gate_drift_rows: u32,
    pub diff_report_path: Option<Arc<str>>,
    /// Plan 08-08: when `gate_passed = false`, this carries the full inline
    /// drift report (also persisted on disk at `diff_report_path`). `None`
    /// when the gate passed — there is nothing to interpret in that case.
    pub gate_drift_report: Option<CutoverGateDriftReport>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CutoverProfileBucket {
    pub sales_person_id: Uuid,
    pub sales_person_name: Arc<str>,
    pub category: AbsenceCategory,
    pub year: u32,
    pub row_count: u32,
    pub sum_amount: f32,
    pub fractional_count: u32,
    pub weekend_on_workday_only_contract_count: u32,
    pub iso_53_indicator: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CutoverProfile {
    pub run_id: Uuid,
    pub generated_at: time::PrimitiveDateTime,
    pub buckets: Arc<[CutoverProfileBucket]>,
    pub profile_path: Arc<str>,
}

/// Outcome of `CutoverService::convert_quarantine_entry` (Phase 8.1, D-01).
/// Carries the freshly inserted `absence_period`, the soft-deleted
/// `extra_hours_id`, and an inline refreshed gate-drift report so the caller
/// avoids a follow-up `gate-dry-run` roundtrip (D-08, RESEARCH P-03 option a).
#[derive(Clone, Debug, PartialEq)]
pub struct ConvertQuarantineEntryOutcome {
    pub absence_period: AbsencePeriodEntity,
    pub deleted_extra_hours_id: Uuid,
    pub refreshed_drift_report: Option<CutoverGateDriftReport>,
}

/// Phase 8.2 (D-29) — domain-tier Manual-Range. REST-Layer parsed
/// `ManualRangeTO` (String-ISO-8601) und mapped auf diese Domain-Struct.
/// Backend-Service akzeptiert pre-parsed `time::Date`s — kein doppeltes
/// Parsing in REST + Service.
#[derive(Clone, Debug, PartialEq)]
pub struct ManualRange {
    pub start_date: time::Date,
    pub end_date: time::Date,
}

/// Outcome of `CutoverService::bulk_convert_quarantine_rows` (Phase 8.1, D-02).
/// All `converted_absence_periods` and `deleted_extra_hours_ids` are produced
/// in a single Tx (strict-atomic — see RESEARCH Q2 / P-10). The `errors` Vec
/// is reserved for future relaxed semantics; on a successful 200 response it
/// is empty. `refreshed_drift_report` mirrors Plan 02 (D-08).
#[derive(Clone, Debug, PartialEq)]
pub struct BulkConvertQuarantineRowsOutcome {
    pub converted_absence_periods: Vec<AbsencePeriodEntity>,
    pub deleted_extra_hours_ids: Vec<Uuid>,
    pub refreshed_drift_report: Option<CutoverGateDriftReport>,
    pub errors: Vec<BulkConvertRowError>,
}

/// Per-row failure detail. Only populated on partial-tolerance modes (none in
/// 8.1 — strict-atomic returns `Err(ValidationError)` for the whole batch).
#[derive(Clone, Debug, PartialEq)]
pub struct BulkConvertRowError {
    pub extra_hours_id: Uuid,
    pub reason: String,
}

#[automock(type Context=(); type Transaction=MockTransaction;)]
#[async_trait]
pub trait CutoverService {
    type Context: Clone + std::fmt::Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    /// Single entry point — both `/admin/cutover/gate-dry-run` and
    /// `/admin/cutover/commit` call this with different `dry_run` values.
    /// Permission check inside (HR for dry_run; cutover_admin for commit) per
    /// Pattern 3 in RESEARCH.md (D-Phase4-08).
    async fn run(
        &self,
        dry_run: bool,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<CutoverRunResult, ServiceError>;

    /// Production-data profile (SC-1 / C-Phase4-05). Read-only — runs full
    /// extra_hours scan and writes `.planning/migration-backup/profile-{ts}.json`.
    /// Permission: HR. Separate from `run` because it must remain runnable on
    /// arbitrarily large data without affecting cutover state.
    async fn profile(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<CutoverProfile, ServiceError>;

    /// Convert a single quarantined extra_hours row into an `absence_period`
    /// in one atomic Tx (D-01). Range derived via `detect_weekly_lump_sum`
    /// heuristic (Plan 08-09); frontend never supplies dates. Privilege:
    /// `cutover_admin` (commit-class). On heuristic mismatch the Tx rolls
    /// back and `ServiceError::ValidationError` is returned (→ HTTP 422).
    ///
    /// Phase 8.2 (D-29): optional `manual_range` skipt die Heuristik und
    /// schreibt direkt eine `absence_period` mit dem gegebenen Range.
    /// Validation: `start <= end`, beide im Quarantäne-Eintrag-Jahr, kein
    /// Overlap mit existing `absence_period`-Rows derselben Person/Kategorie.
    /// `None` (Default) → 8.1-Heuristik-Verhalten unverändert.
    ///
    /// Phase 8.3 (D-08.3-06): optional `day_fraction` setzt die Tageshälfte
    /// auf der resultierenden `absence_period`. `None` / Default → `Full`
    /// (no-drift, CONTEXT.md). Gilt orthogonal zu `manual_range`: sowohl
    /// Heuristic- als auch Manual-Pfad akzeptieren das Feld. Heuristik
    /// (`detect_weekly_lump_sum`) bleibt unverändert — Backend ist passiv,
    /// Operator entscheidet im Frontend.
    async fn convert_quarantine_entry(
        &self,
        extra_hours_id: Uuid,
        manual_range: Option<ManualRange>,
        day_fraction: Option<crate::absence::DayFraction>,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<ConvertQuarantineEntryOutcome, ServiceError>;

    /// Convert all quarantined `extra_hours` rows matching the
    /// `(sales_person_id, category, year)` triple (optionally narrowed by
    /// `extra_hours_ids`) in a single atomic Tx (D-02). All rows share one
    /// `synthetic_run_id` (RESEARCH Q3). Strict-atomic on heuristic
    /// mismatch (RESEARCH P-10): any row that fails `detect_weekly_lump_sum`
    /// rolls back the whole Tx with `ValidationError` (HTTP 422). Privilege:
    /// `cutover_admin`.
    ///
    /// Phase 8.3 (D-08.3-07): optional `day_fraction` gilt einheitlich für
    /// ALLE Rows der Bulk-Operation. `None` / Default → `Full` für jede
    /// konvertierte Zeile (Backwards-Compat).
    async fn bulk_convert_quarantine_rows(
        &self,
        sales_person_id: Uuid,
        category: crate::absence::AbsenceCategory,
        year: u32,
        explicit_ids: Option<Arc<[Uuid]>>,
        day_fraction: Option<crate::absence::DayFraction>,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<BulkConvertQuarantineRowsOutcome, ServiceError>;
}
