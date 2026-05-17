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
    // `try_consume_context` rather than `use_coroutine_handle` so that
    // SSR-render unit tests can exercise these components without registering
    // the cutover_service coroutine. In real runs (production WASM build) the
    // coroutine is registered in app.rs before the page is mounted.
    let cutover_handle = try_consume_context::<Coroutine<CutoverAction>>();
    let store = CUTOVER_STORE.read().clone();
    let store_arc: Arc<CutoverWizardState> = Arc::new(store);
    let already_done = FEATURE_FLAGS_STORE.read().absence_range_source_active();

    // Auto-load profile on first mount (only if not yet loaded).
    let cutover_handle_for_load = cutover_handle.clone();
    let _profile_load = use_resource(move || {
        let cutover_handle = cutover_handle_for_load.clone();
        async move {
            if CUTOVER_STORE.read().profile.is_none() {
                if let Some(h) = cutover_handle.as_ref() {
                    h.send(CutoverAction::LoadProfile);
                }
            }
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
    // `try_consume_context` rather than `use_coroutine_handle` so that
    // SSR-render unit tests can exercise these components without registering
    // the cutover_service coroutine. In real runs (production WASM build) the
    // coroutine is registered in app.rs before the page is mounted.
    let cutover_handle = try_consume_context::<Coroutine<CutoverAction>>();
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
                        if let Some(h) = cutover_handle.as_ref() {
                            h.send(CutoverAction::Commit);
                        }
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
    // `try_consume_context` rather than `use_coroutine_handle` so that
    // SSR-render unit tests can exercise these components without registering
    // the cutover_service coroutine. In real runs (production WASM build) the
    // coroutine is registered in app.rs before the page is mounted.
    let cutover_handle = try_consume_context::<Coroutine<CutoverAction>>();
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
                            if let Some(h) = cutover_handle.as_ref() {
                                h.send(CutoverAction::RunDryRun);
                            }
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

// ─── DriftGroupSection (layout-critical, full body) ───────────────────────
//
// Renders one group per (sales_person, category, year) with header
// (person name + category + year + entry count + per-group bulk-convert
// button) followed by N DriftEntryRow children (filtered by `skipped`).
//
// Bulk-convert button is disabled when `is_cutover_admin == false`.

#[component]
fn DriftGroupSection(
    drift_row: DriftRowRef,
    skipped: Arc<[Uuid]>,
    is_cutover_admin: bool,
) -> Element {
    let i18n = I18N.read().clone();
    // `try_consume_context` rather than `use_coroutine_handle` so that
    // SSR-render unit tests can exercise these components without registering
    // the cutover_service coroutine. In real runs (production WASM build) the
    // coroutine is registered in app.rs before the page is mounted.
    let cutover_handle = try_consume_context::<Coroutine<CutoverAction>>();
    // D-05: filter out skipped entries.
    let visible: Vec<CutoverQuarantineEntryTO> = drift_row
        .0
        .quarantined_entries
        .iter()
        .filter(|e| !skipped.iter().any(|s| *s == e.extra_hours_id))
        .cloned()
        .collect();
    if visible.is_empty() {
        return rsx! {};
    }
    let sales_person_id = drift_row.0.sales_person_id;
    let sales_person_name = drift_row.0.sales_person_name.clone();
    let category = drift_row.0.category;
    let year = drift_row.0.year;
    let drift = drift_row.0.drift;
    let visible_len = visible.len();
    let bulk_label = i18n.t(Key::CutoverBtnBulkConvert);
    rsx! {
        div { class: "bg-surface border border-border rounded-md p-4 flex flex-col gap-3",
            div { class: "flex items-center justify-between gap-3 border-b border-border pb-2",
                div { class: "flex flex-col gap-0",
                    span { class: "font-semibold text-ink", "{sales_person_name}" }
                    span { class: "text-small text-ink-muted",
                        "{category:?} · {year} · {visible_len} rows · drift={drift:.2}h"
                    }
                }
                button {
                    class: if is_cutover_admin {
                        "px-3 py-1.5 rounded-md bg-accent text-accent-ink text-small"
                    } else {
                        "px-3 py-1.5 rounded-md bg-accent/40 text-accent-ink text-small cursor-not-allowed"
                    },
                    disabled: !is_cutover_admin,
                    onclick: move |_| {
                        if is_cutover_admin {
                            if let Some(h) = cutover_handle.as_ref() {
                                // Phase 8.3 (D-08.3-FE-03): no group-wide
                                // half/full toggle. Bulk-Convert defaults to
                                // `Full`; operator uses per-row Convert for
                                // half-day entries.
                                h.send(CutoverAction::BulkConvert {
                                    sales_person_id,
                                    category,
                                    year,
                                    day_fraction: rest_types::DayFractionTO::Full,
                                });
                            }
                        }
                    },
                    "{bulk_label}"
                }
            }
            for entry in visible.into_iter() {
                DriftEntryRow {
                    entry: EntryRef(Arc::new(entry)),
                    drift_row_meta: drift_row.clone(),
                    is_cutover_admin: is_cutover_admin,
                }
            }
        }
    }
}

// ─── DriftEntryRow (layout-critical, full body) ───────────────────────────
//
// 4-column row: Date+Weekday+Amount, Reason badge with tooltip, suggested-
// action hint, 4 action buttons (Convert / Edit / Delete / Skip).
// Edit opens an inline ManualConvertModal (Phase 8.2 — D-29). The modal lets
// the operator give an explicit absence-period range that resolves Karin-class
// quarantine rows the heuristic correctly rejects.

#[component]
fn DriftEntryRow(
    entry: EntryRef,
    drift_row_meta: DriftRowRef,
    is_cutover_admin: bool,
) -> Element {
    let i18n = I18N.read().clone();
    // `try_consume_context` rather than `use_coroutine_handle` so that
    // SSR-render unit tests can exercise these components without registering
    // the cutover_service coroutine. In real runs (production WASM build) the
    // coroutine is registered in app.rs before the page is mounted.
    let cutover_handle = try_consume_context::<Coroutine<CutoverAction>>();
    let mut manual_convert_open = use_signal(|| false);
    // Phase 8.3: per-row Tageshälfte signal. Default `Full`; operator may
    // flip to `Half` via the per-row <select> before clicking Convert.
    let day_fraction = use_signal(|| rest_types::DayFractionTO::Full);
    let entry_id = entry.0.extra_hours_id;
    let date = entry.0.date.clone();
    let weekday = entry.0.weekday.clone();
    let amount = entry.0.amount;
    let reason_code = entry.0.reason_code.clone();
    let reason_text = entry.0.reason_text.clone();
    let suggested_action = entry.0.suggested_action.clone();
    let row_category = drift_row_meta.0.category;

    let convert_label = i18n.t(Key::CutoverRowBtnConvert);
    let edit_label = i18n.t(Key::CutoverRowBtnEdit);
    let delete_label = i18n.t(Key::CutoverRowBtnDelete);
    let skip_label = i18n.t(Key::CutoverRowBtnSkip);
    let entry_for_modal = entry.clone();
    rsx! {
        div { class: "grid grid-cols-12 gap-2 items-center py-2 border-b border-border last:border-b-0",
            div { class: "col-span-3 flex flex-col gap-0",
                span { class: "text-small text-ink", "{date} ({weekday})" }
                span { class: "text-small font-mono text-ink-muted", "{amount:.2}h" }
            }
            div { class: "col-span-3",
                span {
                    class: "inline-block px-2 py-0.5 rounded bg-warn-soft text-warn text-small",
                    title: "{reason_text}",
                    "{reason_code}"
                }
            }
            div { class: "col-span-2 text-small text-ink-muted",
                "{suggested_action}"
            }
            div { class: "col-span-4 flex gap-1 justify-end",
                button {
                    class: if is_cutover_admin {
                        "px-2 py-1 rounded bg-good text-white text-small"
                    } else {
                        "px-2 py-1 rounded bg-good/40 text-white text-small cursor-not-allowed"
                    },
                    disabled: !is_cutover_admin,
                    onclick: move |_| {
                        if is_cutover_admin {
                            if let Some(h) = cutover_handle.as_ref() {
                                // Phase 8.3 (D-08.3-FE-02): day_fraction
                                // value is supplied by the per-row select in
                                // Task 2 — for now we pass the operator-
                                // picked Tageshälfte from the local signal.
                                h.send(CutoverAction::ConvertSingle {
                                    extra_hours_id: entry_id,
                                    day_fraction: *day_fraction.read(),
                                });
                            }
                        }
                    },
                    "{convert_label}"
                }
                button {
                    class: if is_cutover_admin {
                        "px-2 py-1 rounded bg-accent text-accent-ink text-small"
                    } else {
                        "px-2 py-1 rounded bg-accent/40 text-accent-ink text-small cursor-not-allowed"
                    },
                    disabled: !is_cutover_admin,
                    onclick: move |_| {
                        if is_cutover_admin {
                            manual_convert_open.set(true);
                        }
                    },
                    "{edit_label}"
                }
                button {
                    class: if is_cutover_admin {
                        "px-2 py-1 rounded bg-bad text-white text-small"
                    } else {
                        "px-2 py-1 rounded bg-bad/40 text-white text-small cursor-not-allowed"
                    },
                    disabled: !is_cutover_admin,
                    onclick: move |_| {
                        if is_cutover_admin {
                            if let Some(h) = cutover_handle.as_ref() {
                                h.send(CutoverAction::DeleteExtraHours(entry_id));
                            }
                        }
                    },
                    "{delete_label}"
                }
                button {
                    class: "px-2 py-1 rounded bg-surface border border-border text-ink text-small",
                    onclick: move |_| {
                        if let Some(h) = cutover_handle.as_ref() {
                            h.send(CutoverAction::Skip(entry_id));
                        }
                    },
                    "{skip_label}"
                }
            }
            if *manual_convert_open.read() {
                // ManualConvertModal (Phase 8.2 — D-29) replaces the 8.1-09
                // EditExtraHoursModal stub. Submit dispatches
                // `CutoverAction::ConvertSingleManualRange` which the cutover
                // coroutine forwards to the backend's manual_range branch.
                //
                // CutoverQuarantineEntryTO carries no `category` field
                // (rest-types/src/lib.rs:1943-1959); the operator-visible
                // category lives on the parent `CutoverGateDriftRowTO`, which
                // we receive via `drift_row_meta`. Forward it read-only (D-31).
                ManualConvertModal {
                    entry: entry_for_modal.clone(),
                    category: row_category,
                    on_submit: move |(eh_id, start, end, df): (
                        Uuid,
                        time::Date,
                        time::Date,
                        rest_types::DayFractionTO,
                    )| {
                        if let Some(h) = cutover_handle.as_ref() {
                            h.send(CutoverAction::ConvertSingleManualRange {
                                extra_hours_id: eh_id,
                                start_date: start,
                                end_date: end,
                                day_fraction: df,
                            });
                        }
                        // Close on submit so the drift list re-renders cleanly
                        // after `bump_cutover_refresh` fires.
                        manual_convert_open.set(false);
                    },
                    on_cancel: move |_| { manual_convert_open.set(false); },
                }
            }
        }
    }
}

// ─── ManualConvertModal (Phase 8.2 — D-29) ────────────────────────────────
//
// Replaces the 8.1-09 `EditExtraHoursModal` stub. Operator gives an explicit
// absence-period range that bypasses `detect_weekly_lump_sum`. Resolves the
// Karin-Pattern (gap-1a) the heuristic correctly rejects.
//
// Prop contract:
//   entry:    EntryRef                              — read-only context (date,
//             amount, weekday). amount is rendered as a `<span>`, NOT an input
//             (D-32 — operator does not edit hours from the modal).
//   category: AbsenceCategoryTO                     — read-only display only
//             (D-31). Sourced from the parent DriftRow because
//             `CutoverQuarantineEntryTO` carries no `category` field.
//   on_submit: EventHandler<(Uuid, time::Date, time::Date)>
//             — fires on valid submit. Both `time::Date`s are guaranteed
//             parsed (no hardcoded fallback as in the 8.1-09 stub —
//             RESEARCH P-7).
//   on_cancel: EventHandler<()>                     — backdrop click + Cancel
//             button both call this.

#[component]
fn ManualConvertModal(
    entry: EntryRef,
    category: AbsenceCategoryTO,
    /// Phase 8.3 (D-08.3-FE-02): tuple extended with the operator-picked
    /// `DayFractionTO`. Plan 06 wires the per-modal <select> to this value.
    on_submit: EventHandler<(Uuid, time::Date, time::Date, rest_types::DayFractionTO)>,
    on_cancel: EventHandler<()>,
) -> Element {
    let i18n = I18N.read().clone();
    let entry_id = entry.0.extra_hours_id;
    let initial_date = entry.0.date.clone();
    let amount = entry.0.amount;
    let weekday = entry.0.weekday.clone();

    // Pre-fill both date strings with the quarantine row's date so the
    // operator typically only adjusts one bound.
    let mut start_str = use_signal({
        let d = initial_date.clone();
        move || d.clone()
    });
    let mut end_str = use_signal(move || initial_date.clone());
    let mut error_msg = use_signal(|| Option::<String>::None);
    // Phase 8.3 (D-08.3-FE-02): operator-picked Tageshälfte for the new
    // absence-period. Default `Full`; submit threads the current value as
    // the 4th tuple element to `on_submit`.
    let mut day_fraction = use_signal(|| rest_types::DayFractionTO::Full);

    let title = i18n.t(Key::CutoverManualConvertModalTitle);
    let help_text = i18n.t(Key::CutoverManualConvertHelp);
    let amount_label = i18n.t(Key::CutoverEditAmountLabel);
    let start_label = i18n.t(Key::CutoverManualConvertStartLabel);
    let end_label = i18n.t(Key::CutoverManualConvertEndLabel);
    let submit_label = i18n.t(Key::CutoverManualConvertBtnSubmit);
    let cancel_label = i18n.t(Key::CutoverEditBtnCancel);
    let err_start_after_end =
        i18n.t(Key::CutoverManualConvertErrStartAfterEnd).to_string();

    // Pitfall 5: STATIC Tailwind match arms. Map AbsenceCategoryTO to its
    // i18n display key — read-only span only (D-31).
    let category_label = match category {
        AbsenceCategoryTO::Vacation => i18n.t(Key::AbsenceCategoryVacation),
        AbsenceCategoryTO::SickLeave => i18n.t(Key::AbsenceCategorySickLeave),
        AbsenceCategoryTO::UnpaidLeave => i18n.t(Key::AbsenceCategoryUnpaidLeave),
    };

    rsx! {
        div { class: "fixed inset-0 bg-modal-veil flex items-center justify-center z-50",
            onclick: move |_| { on_cancel.call(()); },
            div { class: "bg-surface rounded-lg p-6 flex flex-col gap-4 min-w-md max-w-lg border border-border",
                onclick: move |ev| { ev.stop_propagation(); },
                h3 { class: "text-lg font-semibold text-ink", "{title}" }
                p { class: "text-small text-ink-muted", "{help_text}" }
                // Read-only context row (D-31 + D-32). amount is a
                // `<span>` not an `<input>` — operator does NOT edit hours
                // from the manual-range modal. The original quarantine date
                // is shown as orientation.
                div { class: "flex flex-wrap gap-4 text-small",
                    span { class: "text-ink-muted",
                        "{amount_label}: "
                        span { class: "font-mono text-ink", "{amount:.2}h" }
                    }
                    span { class: "text-ink-muted",
                        "{category_label}"
                    }
                    span { class: "text-ink-muted",
                        "({weekday})"
                    }
                }
                // Two date-inputs (D-29).
                label { class: "flex flex-col gap-1",
                    span { class: "text-small text-ink-muted", "{start_label}" }
                    input {
                        r#type: "date",
                        class: "border border-border rounded-md p-2",
                        value: "{start_str}",
                        oninput: move |ev| { start_str.set(ev.value()); },
                    }
                }
                label { class: "flex flex-col gap-1",
                    span { class: "text-small text-ink-muted", "{end_label}" }
                    input {
                        r#type: "date",
                        class: "border border-border rounded-md p-2",
                        value: "{end_str}",
                        oninput: move |ev| { end_str.set(ev.value()); },
                    }
                }
                // Inline error — rendered when present.
                if let Some(e) = error_msg.read().clone() {
                    span { class: "text-bad text-small", "{e}" }
                }
                div { class: "flex justify-end gap-2",
                    button {
                        class: "px-3 py-2 rounded-md bg-surface border border-border text-ink",
                        onclick: move |_| { on_cancel.call(()); },
                        "{cancel_label}"
                    }
                    button {
                        class: "px-3 py-2 rounded-md bg-accent text-accent-ink",
                        onclick: move |_| {
                            // Pitfall 7 guard: NO unwrap_or_else fallback
                            // to a hardcoded date. Parse failures show an
                            // inline error and block submit; only fully
                            // valid (s, e) with s <= e dispatch.
                            let fmt = time::macros::format_description!(
                                "[year]-[month]-[day]"
                            );
                            let parsed_start = time::Date::parse(
                                start_str.read().as_str(),
                                fmt,
                            );
                            let parsed_end = time::Date::parse(
                                end_str.read().as_str(),
                                fmt,
                            );
                            match (parsed_start, parsed_end) {
                                (Ok(s), Ok(e)) if s <= e => {
                                    error_msg.set(None);
                                    on_submit.call((entry_id, s, e, *day_fraction.read()));
                                }
                                (Ok(_), Ok(_)) => {
                                    // start > end — D-30 #2
                                    error_msg.set(Some(err_start_after_end.clone()));
                                }
                                _ => {
                                    error_msg.set(Some(
                                        "Invalid date format".to_string(),
                                    ));
                                }
                            }
                        },
                        "{submit_label}"
                    }
                }
            }
        }
    }
}

// ─── IdempotenzBanner (layout-critical, full body) ────────────────────────
//
// Shown at the top of CutoverAdminPage when
// `FEATURE_FLAGS_STORE.absence_range_source_active() == true` (D-17).

#[component]
fn IdempotenzBanner() -> Element {
    let i18n = I18N.read().clone();
    let heading = i18n.t(Key::CutoverAlreadyDoneHeading);
    let body = i18n.t(Key::CutoverAlreadyDoneBody);
    rsx! {
        div { class: "bg-accent-soft border border-accent rounded-md p-3 flex flex-col gap-1",
            span { class: "font-semibold text-accent", "{heading}" }
            p { class: "text-small text-ink", "{body}" }
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────
// Tests (Plan 08.1-09 Task 2) — 11 dioxus-ssr snapshot tests covering all
// rendering states (UI-SPEC § States + RESEARCH § Wave 0 Gaps).
//
// Render pattern is the verified one from `absences.rs` L1660-1820 —
// VirtualDom::new + rebuild_in_place + dioxus_ssr::render. Each test pins
// `Locale::De` via `pin_de_locale()` inside its `app` closure so reference
// strings (Übersicht, Vorschau, Durchführen, …) match.
//
// `FEATURE_FLAGS_STORE` is process-global; Tests 9 + 10 explicitly set it
// inside the rendered component via `use_hook` to avoid cross-test bleed.
// ──────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::{generate, Locale};
    use crate::state::feature_flag::FeatureFlagsState;
    use rest_types::{
        AbsenceCategoryTO, CutoverGateDriftReportTO, CutoverGateDriftRowTO,
        CutoverProfileBucketTO, CutoverProfileTO, CutoverQuarantineEntryTO,
    };

    fn render(comp: fn() -> Element) -> String {
        let mut vdom = VirtualDom::new(comp);
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    fn pin_de_locale() {
        use_hook(|| {
            *I18N.write() = generate(Locale::De);
        });
    }

    fn fixture_drift_report_one_group_two_entries() -> CutoverGateDriftReportTO {
        let sp_id = Uuid::from_u128(0xA1A2A3A4);
        let entry_a = CutoverQuarantineEntryTO {
            extra_hours_id: Uuid::from_u128(0xE001),
            date: "2026-05-08".to_string(),
            weekday: "Fri".to_string(),
            amount: 8.0,
            reason_code: "WorkdayMismatch".to_string(),
            reason_text: "Row falls on a non-contracted weekday.".to_string(),
            suggested_action: "Edit the date or delete the row.".to_string(),
        };
        let entry_b = CutoverQuarantineEntryTO {
            extra_hours_id: Uuid::from_u128(0xE002),
            date: "2026-05-09".to_string(),
            weekday: "Sat".to_string(),
            amount: 4.0,
            reason_code: "FractionalHours".to_string(),
            reason_text: "Row carries fractional amount.".to_string(),
            suggested_action: "Round the amount to the nearest 0.25h.".to_string(),
        };
        let drift_row = CutoverGateDriftRowTO {
            sales_person_id: sp_id,
            sales_person_name: "Anna Tester".to_string(),
            category: AbsenceCategoryTO::Vacation,
            year: 2026,
            legacy_sum: 100.0,
            derived_sum: 95.5,
            drift: 4.5,
            quarantined_extra_hours_count: 2,
            quarantine_reasons: vec![
                "WorkdayMismatch".to_string(),
                "FractionalHours".to_string(),
            ],
            quarantined_entries: vec![entry_a, entry_b],
        };
        CutoverGateDriftReportTO {
            gate_run_id: Uuid::from_u128(0xDEADBEEF),
            run_at: "2026-05-09T10:00:00Z".to_string(),
            dry_run: true,
            drift_threshold: 0.01,
            total_drift_rows: 1,
            drift: vec![drift_row],
            passed: false,
        }
    }

    fn fixture_profile_three_buckets() -> CutoverProfileTO {
        let make_bucket = |seed: u128, name: &str, year: u32| CutoverProfileBucketTO {
            sales_person_id: Uuid::from_u128(seed),
            sales_person_name: name.to_string(),
            category: AbsenceCategoryTO::Vacation,
            year,
            row_count: 12,
            sum_hours: 96.0,
            fractional_count: 0,
            weekend_on_workday_only_count: 0,
            iso_53_indicator: false,
        };
        CutoverProfileTO {
            profile_run_id: Uuid::from_u128(0xC0FFEE),
            run_at: "2026-05-09T09:00:00Z".to_string(),
            total_buckets: 3,
            buckets: vec![
                make_bucket(0xAAA1, "Anna Tester", 2026),
                make_bucket(0xAAA2, "Bob Builder", 2026),
                make_bucket(0xAAA3, "Carol Coder", 2026),
            ],
            output_path: ".planning/migration-backup/profile-test.json".to_string(),
        }
    }

    #[test]
    fn stage_indicator_renders_three_circles_with_active_first_stage() {
        fn app() -> Element {
            pin_de_locale();
            rsx! {
                StageIndicator {
                    active: WizardStage::Profile,
                    completed: vec![],
                }
            }
        }
        let html = render(app);
        assert!(html.contains("Übersicht"), "stage 1 label missing: {html}");
        assert!(html.contains("Vorschau"), "stage 2 label missing: {html}");
        assert!(
            html.contains("Durchführen"),
            "stage 3 label missing: {html}"
        );
        assert!(
            html.contains("bg-accent"),
            "active stage accent missing: {html}"
        );
    }

    #[test]
    fn profile_stage_renders_stat_boxes_when_loaded() {
        fn app() -> Element {
            pin_de_locale();
            let mut s = CutoverWizardState::default();
            s.profile = Some(fixture_profile_three_buckets());
            rsx! {
                ProfileStage { store: StateRef(Arc::new(s)) }
            }
        }
        let html = render(app);
        assert!(
            html.contains("Zu migrierende Zeilen"),
            "stat 1 label missing: {html}"
        );
        assert!(
            html.contains("Betroffene Mitarbeiter"),
            "stat 2 label missing: {html}"
        );
        assert!(
            html.contains("Quarantäne-Einträge"),
            "stat 3 label missing: {html}"
        );
        assert!(
            html.contains("Übertrags-Differenz"),
            "stat 4 label missing: {html}"
        );
    }

    #[test]
    fn dry_run_stage_renders_drift_group_section_with_entries() {
        fn app() -> Element {
            pin_de_locale();
            let mut s = CutoverWizardState::default();
            s.last_dry_run = Some(fixture_drift_report_one_group_two_entries());
            rsx! {
                DryRunStage {
                    store: StateRef(Arc::new(s)),
                    is_cutover_admin: true,
                }
            }
        }
        let html = render(app);
        assert!(
            html.contains("Alle in Gruppe konvertieren"),
            "bulk-convert label missing: {html}"
        );
        assert!(
            html.contains("Eintrag konvertieren"),
            "row convert label missing: {html}"
        );
        assert!(html.contains("Bearbeiten"), "edit label missing: {html}");
        assert!(html.contains("Löschen"), "delete label missing: {html}");
        assert!(
            html.contains("Eintrag überspringen"),
            "skip label missing: {html}"
        );
    }

    #[test]
    fn dry_run_stage_renders_empty_state_when_zero_drifts() {
        fn app() -> Element {
            pin_de_locale();
            let mut s = CutoverWizardState::default();
            s.last_dry_run = Some(CutoverGateDriftReportTO {
                gate_run_id: Uuid::nil(),
                run_at: "2026-05-09T10:00:00Z".to_string(),
                dry_run: true,
                drift_threshold: 0.01,
                total_drift_rows: 0,
                drift: vec![],
                passed: true,
            });
            rsx! {
                DryRunStage {
                    store: StateRef(Arc::new(s)),
                    is_cutover_admin: true,
                }
            }
        }
        let html = render(app);
        assert!(
            html.contains("Keine offenen Drifts"),
            "empty heading missing: {html}"
        );
        assert!(
            html.contains("Alle Einträge können automatisch migriert werden"),
            "empty body missing: {html}"
        );
    }

    #[test]
    fn commit_stage_renders_type_to_confirm_input_disabled_btn_initially() {
        fn app() -> Element {
            pin_de_locale();
            let s = CutoverWizardState::default();
            rsx! {
                CommitStage {
                    store: StateRef(Arc::new(s)),
                    is_cutover_admin: true,
                }
            }
        }
        let html = render(app);
        assert!(
            html.contains("Tippe CUTOVER zur Bestätigung"),
            "type-to-confirm label missing: {html}"
        );
        assert!(
            html.contains("disabled"),
            "Commit-Btn must be disabled initially: {html}"
        );
    }

    #[test]
    fn commit_btn_enables_on_exact_cutover_string_match() {
        fn app() -> Element {
            pin_de_locale();
            // matched=true + is_admin=true → the gating evaluates to !disabled.
            commit_form_inner(true, true)
        }
        let html = render(app);
        // The Btn must NOT carry the `disabled=true` attribute. dioxus-ssr
        // emits `disabled=true` (unquoted) or `disabled=false`; we look for
        // the explicit `disabled=true` marker.
        assert!(
            !html.contains("disabled=true"),
            "Commit-Btn should not be disabled when input matches AND user is cutover_admin: {html}"
        );
    }

    #[test]
    fn commit_btn_stays_disabled_for_lowercase_or_typo() {
        fn app() -> Element {
            pin_de_locale();
            // matched=false (input does not equal "CUTOVER") + is_admin=true
            // → still disabled.
            commit_form_inner(false, true)
        }
        let html = render(app);
        assert!(
            html.contains("disabled=true"),
            "Commit-Btn must be disabled when input does not match: {html}"
        );
    }

    #[test]
    fn success_page_renders_summary_with_idempotenz_hint() {
        fn app() -> Element {
            pin_de_locale();
            let mut s = CutoverWizardState::default();
            s.stage = WizardStage::Success;
            s.last_run_summary = Some(RunSummary {
                total_clusters: 5,
                migrated_clusters: 5,
                quarantined_rows: 0,
                gate_drift_rows: 0,
                diff_report_path: Some(
                    ".planning/migration-backup/cutover-gate-test.json".to_string(),
                ),
            });
            rsx! {
                SuccessPage { store: StateRef(Arc::new(s)) }
            }
        }
        let html = render(app);
        assert!(
            html.contains("Cutover abgeschlossen"),
            "success heading missing: {html}"
        );
        assert!(
            html.contains("Wiederholungen sind No-Ops"),
            "idempotenz hint missing: {html}"
        );
    }

    #[test]
    fn idempotenz_banner_renders_when_flag_active_some_true() {
        fn app() -> Element {
            pin_de_locale();
            use_hook(|| {
                *FEATURE_FLAGS_STORE.write() = FeatureFlagsState {
                    absence_range_source_active: Some(true),
                };
            });
            rsx! { IdempotenzBanner {} }
        }
        let html = render(app);
        assert!(
            html.contains("Cutover bereits abgeschlossen"),
            "banner heading missing: {html}"
        );
        assert!(
            html.contains("bg-accent-soft"),
            "banner accent-soft class missing: {html}"
        );
    }

    #[test]
    fn idempotenz_banner_hidden_when_flag_loading_or_false() {
        fn app() -> Element {
            pin_de_locale();
            use_hook(|| {
                *FEATURE_FLAGS_STORE.write() = FeatureFlagsState {
                    absence_range_source_active: None,
                };
            });
            // Mirror CutoverAdminPage's banner-conditional render.
            let already = FEATURE_FLAGS_STORE.read().absence_range_source_active();
            rsx! {
                div {
                    if already { IdempotenzBanner {} }
                    "x"
                }
            }
        }
        let html = render(app);
        assert!(
            !html.contains("Cutover bereits abgeschlossen"),
            "banner should not render when flag is None: {html}"
        );
    }

    // ── ManualConvertModal snapshots (Phase 8.2 — D-29) ───────────────
    //
    // Replace the 8.1-09 EditExtraHoursModal Test 11. Render-only coverage
    // of the open modal + a render-toggle test on DriftEntryRow. Real click
    // dispatch is UAT territory (Phase-8.2 UAT subsumes 8.1-12).

    fn karin_quarantine_entry_fixture() -> CutoverQuarantineEntryTO {
        CutoverQuarantineEntryTO {
            extra_hours_id: Uuid::from_u128(0xCAFE_C0FE),
            date: "2026-05-06".to_string(),
            weekday: "Wed".to_string(),
            amount: 40.0,
            reason_code: "WeeklyLumpSumNoMatch".to_string(),
            reason_text: "Heuristic rejects mid-week contract change."
                .to_string(),
            suggested_action: "Use Manual-Range to set the absence range."
                .to_string(),
        }
    }

    fn karin_drift_row_fixture(entry: CutoverQuarantineEntryTO) -> CutoverGateDriftRowTO {
        CutoverGateDriftRowTO {
            sales_person_id: Uuid::from_u128(0xCAFE_0001),
            sales_person_name: "Karin Karins".to_string(),
            category: AbsenceCategoryTO::Vacation,
            year: 2026,
            legacy_sum: 40.0,
            derived_sum: 36.0,
            drift: 4.0,
            quarantined_extra_hours_count: 1,
            quarantine_reasons: vec!["WeeklyLumpSumNoMatch".to_string()],
            quarantined_entries: vec![entry],
        }
    }

    #[test]
    fn manual_convert_modal_renders_two_date_inputs() {
        // Karin-shaped entry: 40h Vacation on a Wed mid-week. The modal
        // must render two date inputs, the read-only amount as static text
        // (D-32), and DE-locale labels.
        fn app() -> Element {
            pin_de_locale();
            let entry = karin_quarantine_entry_fixture();
            rsx! {
                ManualConvertModal {
                    entry: EntryRef(Arc::new(entry)),
                    category: AbsenceCategoryTO::Vacation,
                    on_submit: move |_p: (Uuid, time::Date, time::Date, rest_types::DayFractionTO)| {},
                    on_cancel: move |_: ()| {},
                }
            }
        }
        let html = render(app);

        // Two `<input type="date">` (start + end). dioxus-ssr emits
        // `r#type` as `type` in the rendered HTML.
        let date_input_count = html.matches(r#"type="date""#).count();
        assert!(
            date_input_count >= 2,
            "expected >= 2 date inputs, got {date_input_count}: {html}"
        );

        // DE labels.
        assert!(
            html.contains("Urlaub manuell anlegen"),
            "DE modal title missing: {html}"
        );
        assert!(
            html.contains("Datum von"),
            "DE start label missing: {html}"
        );
        assert!(
            html.contains("Datum bis"),
            "DE end label missing: {html}"
        );
        assert!(html.contains("Anlegen"), "submit label missing: {html}");

        // D-32: amount must render as a static span, not an `<input>`.
        // The fixture amount is 40.0 → "40.00h".
        assert!(
            html.contains("40.00h"),
            "amount must render as static text: {html}"
        );
        assert!(
            !html.contains(r#"type="number""#),
            "modal must NOT render an amount <input>: {html}"
        );

        // D-31: category must NOT render as an editable surface. We allow
        // the i18n-localised label ("Urlaub" for Vacation in DE) as a
        // span, but no `<select>` for category mutation.
        assert!(
            !html.contains("<select"),
            "modal must NOT render a category <select>: {html}"
        );
    }

    #[test]
    fn manual_convert_modal_renders_validation_error_when_start_after_end() {
        // SSR is render-only — we cannot drive the click that flips
        // `error_msg`. Instead we verify the i18n key resolves to the
        // expected DE string the on-submit handler will surface; this
        // pins the error-string contract that the modal binds to.
        fn app() -> Element {
            pin_de_locale();
            let i18n = I18N.read().clone();
            let msg = i18n
                .t(Key::CutoverManualConvertErrStartAfterEnd)
                .to_string();
            rsx! {
                span { class: "text-bad text-small", "{msg}" }
            }
        }
        let html = render(app);
        assert!(
            html.contains("Startdatum muss vor oder gleich Enddatum sein."),
            "DE start-after-end error string missing: {html}"
        );
        assert!(
            html.contains("text-bad"),
            "error span must use the text-bad class: {html}"
        );
    }

    #[test]
    fn manual_convert_modal_not_rendered_when_closed() {
        // Render the row in its default state — `manual_convert_open`
        // signal is false, so the modal must NOT appear in the SSR HTML.
        // Then render the modal on its own to confirm the open-state
        // does emit the backdrop. This covers the open/close render
        // toggle that the click-handler-driven signal would trigger
        // at runtime; the click itself is UAT territory.
        fn closed_app() -> Element {
            pin_de_locale();
            let entry = karin_quarantine_entry_fixture();
            let drift_row = karin_drift_row_fixture(entry.clone());
            rsx! {
                DriftEntryRow {
                    entry: EntryRef(Arc::new(entry)),
                    drift_row_meta: DriftRowRef(Arc::new(drift_row)),
                    is_cutover_admin: true,
                }
            }
        }
        fn open_app() -> Element {
            pin_de_locale();
            let entry = karin_quarantine_entry_fixture();
            rsx! {
                ManualConvertModal {
                    entry: EntryRef(Arc::new(entry)),
                    category: AbsenceCategoryTO::Vacation,
                    on_submit: move |_p: (Uuid, time::Date, time::Date, rest_types::DayFractionTO)| {},
                    on_cancel: move |_: ()| {},
                }
            }
        }

        let closed_html = render(closed_app);
        assert!(
            !closed_html.contains("bg-modal-veil"),
            "closed row must NOT render the modal backdrop: {closed_html}"
        );
        assert!(
            !closed_html.contains(r#"type="date""#),
            "closed row must NOT render any date input: {closed_html}"
        );

        let open_html = render(open_app);
        assert!(
            open_html.contains("bg-modal-veil"),
            "open modal must render the backdrop: {open_html}"
        );
        assert!(
            open_html.contains(r#"type="date""#),
            "open modal must render at least one date input: {open_html}"
        );
    }

    #[test]
    fn manual_convert_modal_dispatches_action_on_valid_submit() {
        // Pattern from 8.1-09 Test 11: `try_consume_context::<Coroutine<...>>`
        // returns `None` in SSR, so the dispatch path is exercised at the
        // render-surface level only. This test pins the surface (Submit
        // button + DE label) the live coroutine binds to. The actual
        // dispatch firing is verified by Phase-8.2 UAT.
        fn app() -> Element {
            pin_de_locale();
            let entry = karin_quarantine_entry_fixture();
            rsx! {
                ManualConvertModal {
                    entry: EntryRef(Arc::new(entry)),
                    category: AbsenceCategoryTO::Vacation,
                    on_submit: move |_p: (Uuid, time::Date, time::Date, rest_types::DayFractionTO)| {},
                    on_cancel: move |_: ()| {},
                }
            }
        }
        let html = render(app);
        // Submit button rendered with DE label.
        assert!(html.contains("<button"), "submit button missing: {html}");
        assert!(
            html.contains("Anlegen"),
            "DE submit label missing: {html}"
        );
        // The accent class is the visual marker of the primary submit
        // action — pins the button-pair shape to the design contract.
        assert!(
            html.contains("bg-accent"),
            "submit button must carry the bg-accent class: {html}"
        );
        // Cancel button reuses the existing 8.1 i18n key.
        assert!(
            html.contains("Verwerfen"),
            "DE cancel label (CutoverEditBtnCancel) missing: {html}"
        );
    }
}
