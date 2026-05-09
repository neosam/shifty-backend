//! `CutoverAdminPage` — Top-Level Route `/admin/cutover` (Phase 8.1).
//!
//! 3-Stage-Stepper-Wizard (Profile → Dry-Run → Commit) + Drift-Resolution-
//! Liste mit Per-Eintrag-Aktionen + Bulk-Convert + Inline-Edit-Modal +
//! Idempotenz-Banner.
//!
//! Single-File-Page-Composition per Plan 08-05 D-Pattern. Soft-cap 1500 LOC.
//!
//! Plan 08.1-09 Task 1a — Page chrome + Stage components. Task 1b will
//! replace the four placeholder components (DriftGroupSection, DriftEntryRow,
//! EditExtraHoursModal, IdempotenzBanner) with full bodies. Task 2 appends
//! 11 dioxus-ssr snapshot tests.

// Some imports are consumed in Task 1b (DriftGroupSection / DriftEntryRow /
// EditExtraHoursModal / IdempotenzBanner full bodies) and Task 2 (snapshot
// tests). Plan 08.1-09 keeps them in place to lock the file's import block
// shape across the three sub-tasks.
#![allow(unused_imports)]

use std::sync::Arc;

use dioxus::prelude::*;
use rest_types::{
    AbsenceCategoryTO, CutoverGateDriftReportTO, CutoverGateDriftRowTO,
    CutoverProfileTO, CutoverQuarantineEntryTO,
};
use uuid::Uuid;

use crate::component::error_view::ErrorView;
use crate::component::TopBar;
use crate::i18n::Key;
use crate::service::auth::AUTH;
use crate::service::cutover::{
    CutoverAction, CUTOVER_DRIFT_REFRESH, CUTOVER_STORE,
};
use crate::service::feature_flag::FEATURE_FLAGS_STORE;
use crate::service::i18n::I18N;
use crate::state::cutover_state::{CutoverWizardState, RunSummary, WizardStage};

// ─── Component-prop wrappers (PartialEq-by-pointer) ────────────────────────
//
// Several Cutover wire DTOs (`CutoverProfileTO`, `CutoverGateDriftReportTO`,
// `CutoverGateDriftRowTO`, `CutoverQuarantineEntryTO`) and the wizard state
// mirror (`CutoverWizardState`) intentionally do NOT implement `PartialEq`
// (their nested payloads carry float / arc / date types). Dioxus' `#[component]`
// macro auto-derives `PartialEq` on the props struct, so we can't pass these
// types directly as component props.
//
// Solution mirrors `absences.rs` `WarningsList` (Plan 08-05): wrap in `Arc`
// and compare by pointer-equality. Pointer-eq is the right semantics for
// re-render skipping — same allocation = same content = no re-render needed.
// The page always clones an `Arc` instead of mutating; the only mutator is
// the service coroutine which writes a fresh state into `CUTOVER_STORE`.

#[derive(Clone, Debug)]
struct StateRef(Arc<CutoverWizardState>);
impl PartialEq for StateRef {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

#[derive(Clone, Debug)]
struct DriftRowRef(Arc<CutoverGateDriftRowTO>);
impl PartialEq for DriftRowRef {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

#[derive(Clone, Debug)]
struct EntryRef(Arc<CutoverQuarantineEntryTO>);
impl PartialEq for EntryRef {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

// ─── Pitfall 9 — cfg-gated date helper (production WASM vs. native test) ──

#[cfg(target_arch = "wasm32")]
#[allow(dead_code)]
fn current_date_for_init() -> time::Date {
    crate::js::current_datetime().date()
}
#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
fn current_date_for_init() -> time::Date {
    time::macros::date!(2026 - 05 - 09)
}

// ─── CutoverAdminPage — top-level route component ─────────────────────────

#[component]
pub fn CutoverAdminPage() -> Element {
    let auth = AUTH.read().clone();

    // Pitfall 4: AUTH-loading-Gate. Without this, the page can render the
    // privilege-gated content twice (auth resolves in two render cycles).
    if !auth.loading_done {
        return rsx! {
            TopBar {}
            div { class: "p-4 text-ink-muted", "Loading..." }
        };
    }

    let is_cutover_admin = auth
        .auth_info
        .as_ref()
        .map(|a| a.has_privilege("cutover_admin"))
        .unwrap_or(false);
    let is_hr = auth
        .auth_info
        .as_ref()
        .map(|a| a.has_privilege("hr"))
        .unwrap_or(false);

    if !is_cutover_admin && !is_hr {
        return rsx! {
            TopBar {}
            div { class: "p-4 text-bad", "Forbidden" }
        };
    }

    let i18n = I18N.read().clone();
    let cutover_handle = use_coroutine_handle::<CutoverAction>();
    let store = CUTOVER_STORE.read().clone();
    let store_arc: Arc<CutoverWizardState> = Arc::new(store);
    let already_done = FEATURE_FLAGS_STORE.read().absence_range_source_active();

    // Auto-load profile on first mount (only if not yet loaded).
    let _profile_load = use_resource(move || async move {
        if CUTOVER_STORE.read().profile.is_none() {
            cutover_handle.send(CutoverAction::LoadProfile);
        }
    });

    // Subscribe to refresh-token bumps for re-renders. The store has been
    // updated synchronously inside the coroutine; this effect is the render
    // trigger. (D-08 Auto-Re-Run pattern.)
    let refresh_token = *CUTOVER_DRIFT_REFRESH.read();
    use_effect(move || {
        let _ = refresh_token;
    });

    let page_title = i18n.t(Key::CutoverPageTitle);
    let page_subtitle = i18n.t(Key::CutoverPageSubtitle);

    rsx! {
        TopBar {}
        ErrorView {}
        div { class: "p-4 md:p-6 flex flex-col gap-3 max-w-5xl mx-auto",
            if already_done {
                IdempotenzBanner {}
            }
            header { class: "flex flex-col gap-1",
                h1 { class: "text-lg font-semibold text-ink", "{page_title}" }
                p  { class: "text-small text-ink-muted", "{page_subtitle}" }
            }
            StageIndicator {
                active: store_arc.stage,
                completed: completed_stages_for(store_arc.as_ref()),
            }
            div { class: "bg-surface border border-border rounded-lg p-6",
                {match store_arc.stage {
                    WizardStage::Profile => rsx! {
                        ProfileStage { store: StateRef(store_arc.clone()) }
                    },
                    WizardStage::DryRun => rsx! {
                        DryRunStage {
                            store: StateRef(store_arc.clone()),
                            is_cutover_admin: is_cutover_admin,
                        }
                    },
                    WizardStage::Commit => rsx! {
                        CommitStage {
                            store: StateRef(store_arc.clone()),
                            is_cutover_admin: is_cutover_admin,
                        }
                    },
                    WizardStage::Success => rsx! {
                        SuccessPage { store: StateRef(store_arc.clone()) }
                    },
                }}
            }
            StageNavFooter { store: StateRef(store_arc.clone()) }
        }
    }
}

/// Helper: derive the list of completed stages for `StageIndicator`.
fn completed_stages_for(store: &CutoverWizardState) -> Vec<WizardStage> {
    let mut done = Vec::new();
    if matches!(
        store.stage,
        WizardStage::DryRun | WizardStage::Commit | WizardStage::Success
    ) && store.profile.is_some()
    {
        done.push(WizardStage::Profile);
    }
    if matches!(store.stage, WizardStage::Commit | WizardStage::Success)
        && store
            .last_dry_run
            .as_ref()
            .map(|r| r.passed || r.total_drift_rows == 0)
            .unwrap_or(false)
    {
        done.push(WizardStage::DryRun);
    }
    if matches!(store.stage, WizardStage::Success) {
        done.push(WizardStage::Commit);
    }
    done
}

// ─── StageIndicator (layout-trivial, final form) ──────────────────────────
//
// Pitfall 5: STATIC Tailwind match-arm strings — never `format!` for class
// names. Inline interpolation in `class` attribute is safe (Dioxus emits the
// literal token at compile-time); `format!`-built strings would break the
// Tailwind purger.

#[component]
fn StageIndicator(active: WizardStage, completed: Vec<WizardStage>) -> Element {
    let i18n = I18N.read().clone();
    let stages = [
        (WizardStage::Profile, Key::CutoverStage1Label),
        (WizardStage::DryRun, Key::CutoverStage2Label),
        (WizardStage::Commit, Key::CutoverStage3Label),
    ];
    let last = stages.len() - 1;
    rsx! {
        div { class: "flex items-center gap-0 w-full max-w-xl mx-auto my-6",
            for (idx, (stage, key)) in stages.iter().enumerate() {
                {
                    let is_active = active == *stage;
                    let is_complete = completed.contains(stage);
                    let circle_cls = match (is_active, is_complete) {
                        (true, _) => "bg-accent text-accent-ink border-accent",
                        (_, true) => "bg-good text-white border-good",
                        _ => "bg-surface text-ink-muted border-border",
                    };
                    let label_cls = match (is_active, is_complete) {
                        (true, _) => "text-accent",
                        (_, true) => "text-good",
                        _ => "text-ink-muted",
                    };
                    let label = i18n.t(*key);
                    rsx! {
                        div { class: "flex flex-col items-center",
                            div { class: "w-8 h-8 rounded-full flex items-center justify-center text-small font-semibold border-2 {circle_cls}",
                                if is_complete {
                                    "✓"
                                } else {
                                    "{idx + 1}"
                                }
                            }
                            span { class: "text-small mt-1 {label_cls}",
                                "{label}"
                            }
                        }
                        if idx < last {
                            div { class: "flex-1 h-0.5 bg-border mx-2" }
                        }
                    }
                }
            }
        }
    }
}

// ─── StatBox (layout-trivial, final form) ─────────────────────────────────
//
// Re-usable stat card. Mirrors `absences.rs` `VacationStatBox` body but with
// the simpler 2-line layout from UI-SPEC § Profile Stage Stat Grid.

#[component]
fn StatBox(label: String, value: String) -> Element {
    rsx! {
        div { class: "bg-surface border border-border rounded-md p-3 flex flex-col gap-1",
            span { class: "text-small text-ink-muted", "{label}" }
            span { class: "text-base font-semibold text-ink", "{value}" }
        }
    }
}

// ─── ProfileStage (layout-critical, final form) ───────────────────────────
//
// 4-StatBox grid + Per-Person tile-list. Aggregates from
// `CutoverProfileTO.buckets` since the wire DTO does not carry pre-rolled
// totals.

#[component]
fn ProfileStage(store: StateRef) -> Element {
    let i18n = I18N.read().clone();
    let profile = store.0.profile.as_ref();
    // Aggregate from buckets — `CutoverProfileTO` does not pre-roll the
    // page-level totals. Each bucket is a (sales_person, category, year)
    // triple with `row_count` + `sum_hours`.
    let total_rows: u32 = profile
        .map(|p| p.buckets.iter().map(|b| b.row_count).sum())
        .unwrap_or(0);
    let affected_persons: u32 = profile
        .map(|p| {
            let mut ids: Vec<Uuid> = p.buckets.iter().map(|b| b.sales_person_id).collect();
            ids.sort();
            ids.dedup();
            ids.len() as u32
        })
        .unwrap_or(0);
    // "Quarantine" stat — number of buckets that flagged any quarantine
    // signals (fractional or weekend-on-workday-only). Derived from the
    // profile rather than the gate-drift report so it shows during Stage 1.
    let quarantine_signal_buckets: u32 = profile
        .map(|p| {
            p.buckets
                .iter()
                .filter(|b| b.fractional_count > 0 || b.weekend_on_workday_only_count > 0)
                .count() as u32
        })
        .unwrap_or(0);
    // Carryover-diff approximation: ISO-53 indicator buckets count as a
    // signal that the year-end carryover may differ. Phase 8.1 only needs
    // the indicator visible, not a precise value.
    let carryover_signal_buckets: u32 = profile
        .map(|p| p.buckets.iter().filter(|b| b.iso_53_indicator).count() as u32)
        .unwrap_or(0);

    let label_total_rows = i18n.t(Key::CutoverStatTotalRows);
    let label_persons = i18n.t(Key::CutoverStatPersons);
    let label_quarantine = i18n.t(Key::CutoverStatQuarantine);
    let label_carryover_diff = i18n.t(Key::CutoverStatCarryoverDiff);

    rsx! {
        div { class: "flex flex-col gap-4",
            div { class: "grid grid-cols-2 md:grid-cols-4 gap-3",
                StatBox {
                    label: label_total_rows.to_string(),
                    value: format!("{total_rows}"),
                }
                StatBox {
                    label: label_persons.to_string(),
                    value: format!("{affected_persons}"),
                }
                StatBox {
                    label: label_quarantine.to_string(),
                    value: format!("{quarantine_signal_buckets}"),
                }
                StatBox {
                    label: label_carryover_diff.to_string(),
                    value: format!("{carryover_signal_buckets}"),
                }
            }
            if let Some(p) = profile {
                div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3",
                    for bucket in p.buckets.iter() {
                        div { class: "bg-surface border border-border rounded-md p-3 flex flex-col gap-1",
                            span { class: "font-medium text-ink", "{bucket.sales_person_name}" }
                            span { class: "text-small text-ink-muted",
                                "{bucket.row_count} rows · {bucket.sum_hours:.2}h"
                            }
                        }
                    }
                }
            }
        }
    }
}

// ─── DryRunStage (shell — populated rows delegate to DriftGroupSection) ───
//
// Empty-state rendered inline. Populated state delegates to
// `DriftGroupSection` (full body in Task 1b).

#[component]
fn DryRunStage(store: StateRef, is_cutover_admin: bool) -> Element {
    let i18n = I18N.read().clone();
    let report = store.0.last_dry_run.as_ref();
    let total_drift = report.map(|r| r.total_drift_rows).unwrap_or(0);
    let empty_heading = i18n.t(Key::CutoverDriftEmptyHeading);
    let empty_body = i18n.t(Key::CutoverDriftEmptyBody);
    if total_drift == 0 {
        return rsx! {
            div { class: "flex flex-col items-center gap-2 py-8",
                h2 { class: "text-lg font-semibold text-good", "{empty_heading}" }
                p  { class: "text-small text-ink-muted text-center max-w-prose",
                    "{empty_body}"
                }
            }
        };
    }
    let report = report.unwrap();
    let skipped = store.0.skipped_extra_hours_ids.clone();
    rsx! {
        div { class: "flex flex-col gap-4",
            for drift_row in report.drift.iter() {
                DriftGroupSection {
                    drift_row: DriftRowRef(Arc::new(drift_row.clone())),
                    skipped: skipped.clone(),
                    is_cutover_admin: is_cutover_admin,
                }
            }
        }
    }
}

// ─── CommitStage (layout-critical, final form) ────────────────────────────
//
// Production CommitStage owns the use_signal for the input value; rendering
// of the actual Btn delegates to `commit_form_inner(matched, is_admin)` so
// Tests 6 + 7 can render the gating logic directly with externally
// controlled bools (no use_signal manipulation in tests).

#[component]
fn CommitStage(store: StateRef, is_cutover_admin: bool) -> Element {
    let i18n = I18N.read().clone();
    let mut input = use_signal(String::new);
    let matched = input.read().as_str() == "CUTOVER";

    let summary_heading = i18n.t(Key::CutoverCommitSummaryHeading);

    let stat_total = i18n.t(Key::CutoverStatTotalRows);
    let stat_persons = i18n.t(Key::CutoverStatPersons);
    let stat_quarantine = i18n.t(Key::CutoverStatQuarantine);

    rsx! {
        div { class: "flex flex-col gap-4",
            h2 { class: "text-lg font-semibold text-ink", "{summary_heading}" }
            div { class: "grid grid-cols-2 md:grid-cols-3 gap-3",
                if let Some(summary) = store.0.last_run_summary.as_ref() {
                    StatBox {
                        label: stat_total.to_string(),
                        value: format!("{}", summary.total_clusters),
                    }
                    StatBox {
                        label: stat_persons.to_string(),
                        value: format!("{}", summary.migrated_clusters),
                    }
                    StatBox {
                        label: stat_quarantine.to_string(),
                        value: format!("{}", summary.quarantined_rows),
                    }
                }
            }
            input {
                r#type: "text",
                class: "border border-border rounded-md p-2 text-base",
                placeholder: "CUTOVER",
                value: "{input}",
                oninput: move |ev| input.set(ev.value()),
            }
            { commit_form_inner(matched, is_cutover_admin) }
        }
    }
}

// ─── commit_form_inner (test-friendly helper, NOT a #[component]) ─────────
//
// Tests 6 + 7 in Task 2 call this directly with externally-controlled bools.

fn commit_form_inner(matched: bool, is_admin: bool) -> Element {
    let i18n = I18N.read().clone();
    let cutover_handle = use_coroutine_handle::<CutoverAction>();
    let disabled = !matched || !is_admin;
    let label = i18n.t(Key::CutoverCommitTypeLabel);
    let btn_label = i18n.t(Key::CutoverCommitBtn);
    let tooltip_admin = i18n.t(Key::CutoverPrivilegeStage3);
    let tooltip_text: Option<std::rc::Rc<str>> = if !is_admin {
        Some(tooltip_admin)
    } else if !matched {
        Some(label.clone())
    } else {
        None
    };
    rsx! {
        div { class: "flex flex-col gap-2",
            p { class: "text-small text-ink-muted", "{label}" }
            button {
                class: if disabled {
                    "px-4 py-2 rounded-md bg-bad-soft text-bad cursor-not-allowed border border-bad"
                } else {
                    "px-4 py-2 rounded-md bg-bad text-white hover:bg-bad-soft border border-bad"
                },
                disabled: disabled,
                onclick: move |_| {
                    if !disabled {
                        cutover_handle.send(CutoverAction::Commit);
                    }
                },
                "{btn_label}"
            }
            if let Some(t) = tooltip_text {
                span { class: "text-small text-ink-muted", "{t}" }
            }
        }
    }
}

// ─── SuccessPage (short final form) ───────────────────────────────────────

#[component]
fn SuccessPage(store: StateRef) -> Element {
    let i18n = I18N.read().clone();
    let backup_path = store
        .0
        .last_run_summary
        .as_ref()
        .and_then(|s| s.diff_report_path.clone())
        .unwrap_or_else(|| "(unavailable)".to_string());
    let heading = i18n.t(Key::CutoverSuccessHeading);
    let body_template = i18n.t(Key::CutoverSuccessBody);
    let body = body_template.as_ref().replace("{path}", &backup_path);
    rsx! {
        div { class: "bg-good-soft border border-good rounded-lg p-8 flex flex-col items-center gap-3",
            h2 { class: "text-xl font-semibold text-good", "{heading}" }
            p { class: "text-small text-ink text-center max-w-prose",
                "{body}"
            }
        }
    }
}

// ─── StageNavFooter (Back / Continue button pair) ─────────────────────────

#[component]
fn StageNavFooter(store: StateRef) -> Element {
    let i18n = I18N.read().clone();
    let cutover_handle = use_coroutine_handle::<CutoverAction>();
    let can_advance = match store.0.stage {
        WizardStage::Profile => store.0.profile.is_some(),
        WizardStage::DryRun => store
            .0
            .last_dry_run
            .as_ref()
            .map(|r| r.passed || r.total_drift_rows == 0)
            .unwrap_or(false),
        // Commit-Btn is the advance gate inside CommitStage — Continue
        // disabled here.
        WizardStage::Commit => false,
        WizardStage::Success => false,
    };
    let back_disabled = matches!(store.0.stage, WizardStage::Profile | WizardStage::Success);
    let back_label = i18n.t(Key::CutoverBtnBack);
    let continue_label = i18n.t(Key::CutoverBtnContinue);
    rsx! {
        div { class: "flex justify-between gap-3",
            button {
                class: if back_disabled {
                    "px-3 py-2 rounded-md bg-surface border border-border text-ink-muted cursor-not-allowed"
                } else {
                    "px-3 py-2 rounded-md bg-surface border border-border text-ink"
                },
                disabled: back_disabled,
                onclick: move |_| {
                    let mut s = CUTOVER_STORE.write();
                    s.stage = match s.stage {
                        WizardStage::DryRun => WizardStage::Profile,
                        WizardStage::Commit => WizardStage::DryRun,
                        other => other,
                    };
                },
                "{back_label}"
            }
            button {
                class: if can_advance {
                    "px-3 py-2 rounded-md bg-accent text-accent-ink"
                } else {
                    "px-3 py-2 rounded-md bg-accent/40 text-accent-ink cursor-not-allowed"
                },
                disabled: !can_advance,
                onclick: move |_| {
                    let mut s = CUTOVER_STORE.write();
                    let next = match s.stage {
                        WizardStage::Profile => {
                            cutover_handle.send(CutoverAction::RunDryRun);
                            WizardStage::DryRun
                        }
                        WizardStage::DryRun => WizardStage::Commit,
                        other => other,
                    };
                    s.stage = next;
                },
                "{continue_label}"
            }
        }
    }
}

// ─── Task-1b stub components (replaced in Task 1b) ────────────────────────
//
// These four signatures are LOCKED for Task 1b — Task 1b replaces only the
// bodies, not the props. The page chrome above already references them.

#[component]
fn DriftGroupSection(
    drift_row: DriftRowRef,
    skipped: Arc<[Uuid]>,
    is_cutover_admin: bool,
) -> Element {
    let _ = (drift_row, skipped, is_cutover_admin);
    rsx! { div { "(Task 1b)" } }
}

#[component]
fn DriftEntryRow(
    entry: EntryRef,
    drift_row_meta: DriftRowRef,
    is_cutover_admin: bool,
) -> Element {
    let _ = (entry, drift_row_meta, is_cutover_admin);
    rsx! { div { "(Task 1b)" } }
}

#[component]
fn EditExtraHoursModal(
    entry: EntryRef,
    on_save: EventHandler<(Uuid, f64, time::Date)>,
    on_cancel: EventHandler<()>,
) -> Element {
    let _ = (entry, on_save, on_cancel);
    rsx! { div { "(Task 1b)" } }
}

#[component]
fn IdempotenzBanner() -> Element {
    rsx! { div { "(Task 1b)" } }
}
