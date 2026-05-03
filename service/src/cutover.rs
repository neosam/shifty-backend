//! Phase 4 — Cutover orchestration trait + DTOs.
//!
//! Business-Logic-Tier service per shifty-backend/CLAUDE.md § "Service-Tier-
//! Konventionen". Consumes AbsenceService + ExtraHoursService +
//! CarryoverRebuildService + FeatureFlagService + EmployeeWorkDetailsService +
//! CutoverDao + PermissionService + TransactionDao. NO Sub-Service depends on
//! CutoverService — no cycle.

use std::sync::Arc;

use async_trait::async_trait;
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
}
