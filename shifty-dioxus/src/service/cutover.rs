//! Cutover wizard service-coroutine + global stores (Phase 8.1).
//!
//! Owns:
//! - `CUTOVER_STORE`: the wizard state mirror (stage / profile / last drift /
//!   last run summary / skipped set / busy-flag).
//! - `CUTOVER_DRIFT_REFRESH`: refresh-token bumped on each successful resolve
//!   action so the page re-renders the drift list (Plan 09 `use_effect`,
//!   D-08 Auto-Re-Run pattern).
//! - `cutover_service`: the coroutine that processes `CutoverAction` events.
//!
//! Pattern mirror: `service/absence.rs` (Plan 08-04 baseline) — same
//! GlobalSignal store + bump-token + Action enum + while-let-coroutine
//! shape. Differences vs. absence_service:
//! - No modal-event side-channel (Plan 09 will read the busy-flag + drift
//!   refresh-token directly; no per-action outcome events needed yet).
//! - Convert / Bulk-Convert responses carry an inline
//!   `refreshed_drift_report` (D-08 inline pattern, Plan 05) — the coroutine
//!   writes that directly to `CUTOVER_STORE.last_dry_run`, no extra
//!   `gate-dry-run` roundtrip.
//! - Skip is frontend-only (D-05) — no API call, just appends to the skipped
//!   set on the store.

use std::sync::Arc;

use dioxus::prelude::*;
use futures_util::StreamExt;
use rest_types::{AbsenceCategoryTO, ExtraHoursTO, ManualRangeTO};
use tracing::info;
use uuid::Uuid;

use crate::api;
use crate::error::ShiftyError;
use crate::state::Config;
use crate::state::cutover_state::{CutoverWizardState, RunSummary, WizardStage};

use super::config::CONFIG;
use super::error::{ErrorStore, ERROR_STORE};

pub static CUTOVER_STORE: GlobalSignal<CutoverWizardState> =
    Signal::global(CutoverWizardState::default);

/// Refresh token bumped on every successful resolve action (ConvertSingle /
/// BulkConvert / UpdateExtraHours / DeleteExtraHours). Plan 09's drift-list
/// `use_effect` subscribes to this to scroll-into-view / animate the list
/// after a re-run. RunDryRun does NOT bump (the page reads `last_dry_run`
/// directly).
pub static CUTOVER_DRIFT_REFRESH: GlobalSignal<u64> = Signal::global(|| 0);

pub(crate) fn bump_cutover_refresh() {
    *CUTOVER_DRIFT_REFRESH.write() += 1;
}

/// Cutover-wizard action enum dispatched by Plan 09 via
/// `use_coroutine_handle::<CutoverAction>()`.
///
/// `Skip` is the only frontend-only variant (D-05 — no API call, just
/// appends to the skipped set).
#[derive(Debug)]
pub enum CutoverAction {
    LoadProfile,
    RunDryRun,
    Commit,
    ConvertSingle(Uuid),
    /// Phase 8.2 (D-29): operator-supplied date range. Backend skips the
    /// heuristic and uses `start_date..=end_date` directly. Resolves the
    /// Karin-Pattern (gap-1a) that `detect_weekly_lump_sum` rejects by design.
    ConvertSingleManualRange {
        extra_hours_id: Uuid,
        start_date: time::Date,
        end_date: time::Date,
    },
    BulkConvert {
        sales_person_id: Uuid,
        category: AbsenceCategoryTO,
        year: u32,
    },
    UpdateExtraHours(ExtraHoursTO),
    DeleteExtraHours(Uuid),
    /// D-05: frontend-only — no API call.
    Skip(Uuid),
}

pub async fn cutover_service(mut rx: UnboundedReceiver<CutoverAction>) {
    while let Some(action) = rx.next().await {
        info!("CutoverAction: {:?}", &action);
        let config = CONFIG.read().clone();

        // Mark busy for spinner overlay (D-09). Skip is instant — no busy.
        if !matches!(action, CutoverAction::Skip(_)) {
            CUTOVER_STORE.write().busy = true;
        }

        let result = process_action(action, config).await;

        if let Err(err) = result {
            *ERROR_STORE.write() = ErrorStore { error: Some(err) };
        }
        CUTOVER_STORE.write().busy = false;
    }
}

async fn process_action(
    action: CutoverAction,
    config: Config,
) -> Result<(), ShiftyError> {
    match action {
        CutoverAction::LoadProfile => {
            let profile = api::cutover_profile(config).await?;
            CUTOVER_STORE.write().profile = Some(profile);
        }
        CutoverAction::RunDryRun => {
            let result = api::cutover_gate_dry_run(config).await?;
            let mut store = CUTOVER_STORE.write();
            store.last_dry_run = result.gate_drift_report.clone();
            store.last_run_summary = Some(RunSummary::from(&result));
        }
        CutoverAction::Commit => {
            let result = api::cutover_commit(config).await?;
            let mut store = CUTOVER_STORE.write();
            store.last_run_summary = Some(RunSummary::from(&result));
            store.stage = WizardStage::Success;
        }
        CutoverAction::ConvertSingle(extra_hours_id) => {
            // Heuristic path (8.1): manual_range = None preserves original behaviour.
            let resp = api::cutover_convert_quarantine_entry(config, extra_hours_id, None).await?;
            // D-08 inline pattern: write refreshed gate-drift-report directly.
            CUTOVER_STORE.write().last_dry_run = resp.refreshed_drift_report;
            bump_cutover_refresh();
        }
        CutoverAction::ConvertSingleManualRange {
            extra_hours_id,
            start_date,
            end_date,
        } => {
            // Phase 8.2 (D-29): operator-supplied range bypasses the heuristic.
            // Submit-time validation in the modal (Task 3) ensures both dates
            // are valid `time::Date`s before dispatch — `unwrap_or_default()`
            // here is a defence-in-depth belt; backend will reject empty
            // strings with ValidationError on parse-fail (Plan 01 P-5).
            let fmt = time::macros::format_description!("[year]-[month]-[day]");
            let manual_range = ManualRangeTO {
                start_date: start_date.format(&fmt).unwrap_or_default(),
                end_date: end_date.format(&fmt).unwrap_or_default(),
            };
            let resp = api::cutover_convert_quarantine_entry(
                config.clone(),
                extra_hours_id,
                Some(manual_range),
            )
            .await?;
            // P-6 fallback: if the backend's inline replay failed, do a
            // separate gate-dry-run so the drift list still refreshes
            // (mirrors the UpdateExtraHours / DeleteExtraHours branches).
            let drift = match resp.refreshed_drift_report {
                Some(r) => Some(r),
                None => api::cutover_gate_dry_run(config)
                    .await
                    .ok()
                    .and_then(|r| r.gate_drift_report),
            };
            CUTOVER_STORE.write().last_dry_run = drift;
            bump_cutover_refresh();
        }
        CutoverAction::BulkConvert {
            sales_person_id,
            category,
            year,
        } => {
            let resp = api::cutover_bulk_convert_quarantine_rows(
                config,
                sales_person_id,
                category,
                year,
                /* explicit ids */ None,
            )
            .await?;
            CUTOVER_STORE.write().last_dry_run = resp.refreshed_drift_report;
            bump_cutover_refresh();
        }
        CutoverAction::UpdateExtraHours(body) => {
            // Re-uses existing PUT /extra-hours/{id} (D-04).
            let _ = api::update_extra_hour(config.clone(), body).await?;
            // After edit, re-run gate-dry-run so the drift list reflects the
            // updated row (D-08 — Edit triggers Auto-Re-Run).
            let result = api::cutover_gate_dry_run(config).await?;
            CUTOVER_STORE.write().last_dry_run = result.gate_drift_report.clone();
            bump_cutover_refresh();
        }
        CutoverAction::DeleteExtraHours(id) => {
            // delete_extra_hour returns Result<(), reqwest::Error>; convert
            // via the From<reqwest::Error> for ShiftyError impl.
            api::delete_extra_hour(config.clone(), id)
                .await
                .map_err(ShiftyError::from)?;
            // Same D-08 Auto-Re-Run.
            let result = api::cutover_gate_dry_run(config).await?;
            CUTOVER_STORE.write().last_dry_run = result.gate_drift_report.clone();
            bump_cutover_refresh();
        }
        CutoverAction::Skip(id) => {
            // D-05: frontend-only retain — no API call, no Auto-Re-Run.
            apply_skip(id);
        }
    }
    Ok(())
}

/// Pure helper for the `Skip` branch — extracted so unit tests can exercise
/// it without spinning up a tokio runtime / mock HTTP server. The branch is
/// also called from `process_action` above.
fn apply_skip(id: Uuid) {
    let mut store = CUTOVER_STORE.write();
    let mut next = store.skipped_extra_hours_ids.to_vec();
    next.push(id);
    store.skipped_extra_hours_ids = Arc::from(next.into_boxed_slice());
}

#[cfg(test)]
mod tests {
    //! Service-level smoke tests. The coroutine itself drives `reqwest` calls
    //! against the configured backend, so we cannot exercise it end-to-end in
    //! a unit test without standing up an HTTP server. We instead lock the
    //! observable contracts the page relies on:
    //! - `bump_cutover_refresh` is observable.
    //! - `Skip` appends to `skipped_extra_hours_ids` (frontend-only D-05).
    //!
    //! Pattern mirror: `service/absence.rs` tests — wrap GlobalSignal access
    //! in a `VirtualDom::new(...)` so the Dioxus runtime is in scope.
    use super::*;

    #[test]
    fn bump_cutover_refresh_increments_observable_signal() {
        fn assertion_app() -> Element {
            let before = *CUTOVER_DRIFT_REFRESH.read();
            bump_cutover_refresh();
            let after = *CUTOVER_DRIFT_REFRESH.read();
            assert_eq!(after, before.wrapping_add(1));
            rsx! {}
        }
        let mut vdom = VirtualDom::new(assertion_app);
        vdom.rebuild_in_place();
    }

    #[test]
    fn skip_action_appends_to_skipped_set() {
        fn assertion_app() -> Element {
            // Reset to a known empty set (other tests share global state).
            CUTOVER_STORE.write().skipped_extra_hours_ids = Arc::from([] as [Uuid; 0]);
            let id = Uuid::from_u128(0xCAFEBABE);
            apply_skip(id);
            let store = CUTOVER_STORE.read();
            assert!(
                store.skipped_extra_hours_ids.iter().any(|x| *x == id),
                "skipped set must contain the id we just skipped"
            );
            rsx! {}
        }
        let mut vdom = VirtualDom::new(assertion_app);
        vdom.rebuild_in_place();
    }
}
