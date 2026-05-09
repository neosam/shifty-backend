//! Frontend state for the Cutover Wizard (Phase 8.1).
//!
//! Mirrors the wire DTOs (`CutoverProfileTO`, `CutoverGateDriftReportTO`,
//! `CutoverRunResultTO`) and adds wizard-specific UI state (current stage,
//! frontend-only skipped set per D-05, last-known summary for the success
//! page, busy-flag for the loading-spinner overlay D-09).
//!
//! Structural template: `state/absence_period.rs` (Plan 08-04).

use std::sync::Arc;

use rest_types::{CutoverGateDriftReportTO, CutoverProfileTO, CutoverRunResultTO};
use uuid::Uuid;

/// Linear 3-step wizard with a terminal Success step shown after Commit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WizardStage {
    Profile,
    DryRun,
    Commit,
    Success,
}

impl Default for WizardStage {
    fn default() -> Self {
        WizardStage::Profile
    }
}

/// Cached migration-summary for the Success-Page hero (post-Commit) and the
/// Stage-3 Type-to-confirm dialog.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunSummary {
    pub total_clusters: u32,
    pub migrated_clusters: u32,
    pub quarantined_rows: u32,
    pub gate_drift_rows: u32,
    pub diff_report_path: Option<String>,
}

impl From<&CutoverRunResultTO> for RunSummary {
    fn from(r: &CutoverRunResultTO) -> Self {
        Self {
            total_clusters: r.total_clusters,
            migrated_clusters: r.migrated_clusters,
            quarantined_rows: r.quarantined_rows,
            gate_drift_rows: r.gate_drift_rows,
            diff_report_path: r.diff_report_path.as_deref().map(|s| s.to_string()),
        }
    }
}

/// Wizard-state mirror — lives in `CUTOVER_STORE` (`service::cutover`).
///
/// `PartialEq` is intentionally NOT derived: `CutoverProfileTO` and
/// `CutoverGateDriftReportTO` (from `rest-types`) do not implement
/// `PartialEq` because their nested DTOs carry float / arc / date payloads
/// without `Eq`-friendly semantics. The wizard-state is read in `use_effect`
/// blocks via `*CUTOVER_STORE.read()` and field-level matchers, never via
/// `==` on the full struct, so the missing `PartialEq` is irrelevant.
#[derive(Clone, Debug, Default)]
pub struct CutoverWizardState {
    pub stage: WizardStage,
    pub profile: Option<CutoverProfileTO>,
    /// Latest `gate_drift_report` — fed inline from Convert/Bulk-Convert
    /// responses (D-08) or from a separate `gate-dry-run` call. `None` until
    /// the user reaches Stage 2.
    pub last_dry_run: Option<CutoverGateDriftReportTO>,
    pub last_run_summary: Option<RunSummary>,
    /// D-05: Skip is frontend-only — these IDs are filtered out of the visible
    /// drift list locally; they reappear after a page reload (no persistence).
    pub skipped_extra_hours_ids: Arc<[Uuid]>,
    /// While a coroutine action is in flight (Convert / Edit / Delete /
    /// Bulk-Convert / RunDryRun): used by Plan 09 for the loading-spinner
    /// overlay (D-09).
    pub busy: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wizard_state_default_starts_at_profile_stage() {
        let s = CutoverWizardState::default();
        assert_eq!(s.stage, WizardStage::Profile);
    }

    #[test]
    fn skipped_set_starts_empty() {
        let s = CutoverWizardState::default();
        assert!(s.skipped_extra_hours_ids.is_empty());
    }

    #[test]
    fn wizard_stage_can_advance() {
        let mut s = CutoverWizardState::default();
        s.stage = WizardStage::DryRun;
        assert_eq!(s.stage, WizardStage::DryRun);
    }
}
