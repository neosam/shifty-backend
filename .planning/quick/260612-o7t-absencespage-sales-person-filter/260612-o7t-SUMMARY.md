---
phase: quick-260612-o7t
plan: 01
subsystem: frontend
tags: [absences, stats, vacation, person-filter, tdd, dioxus]
dependency_graph:
  requires: [quick-260612-nlv]
  provides: [D1-StatsGrid-person-scope, D2-VacationCard-self-view]
  affects: [shifty-dioxus/src/page/absences.rs]
tech_stack:
  added: []
  patterns: [pure-helper-TDD, props-default-none, show_self-branch]
key_files:
  modified:
    - shifty-dioxus/src/page/absences.rs
decisions:
  - "D1: StatsGrid scoped by person dimension ONLY — category/status filters not applied to stats boxes"
  - "D2: HR selecting a person renders VacationEntitlementSelfBody (hero + 5 StatBoxes) using that person's VacationBalance from vacation_team"
  - "D3: Employee (non-HR) path byte-identical — show_self=true when !is_hr, same as before"
  - "D4: No backend / rest-types / state / api changes — pure frontend filter over already-loaded stores"
metrics:
  duration: "~15 minutes"
  completed: "2026-06-12"
  tasks_completed: 2
  files_changed: 1
---

# Phase quick-260612-o7t Plan 01: AbsencesPage Sales Person Filter (StatsGrid + VacationCard) Summary

**One-liner:** Person-scope StatsGrid via `stats_for_person` pure helper and self-view branch in `VacationEntitlementCard` when HR selects a sales person, TDD RED-first for both tasks.

## Tasks Completed

| Task | Name | jj Change | Commit | Files |
|------|------|-----------|--------|-------|
| 1 | Extract stats_for_person + person-scope StatsGrid | pmqnltkw | 1899d8e5 | absences.rs |
| 2 | VacationEntitlementCard self-view for selected person | nskymxlq | 5402438f | absences.rs |

## What Was Built

### Task 1: `stats_for_person` pure helper + StatsGrid person-scoping (D1)

- Added `pub fn stats_for_person(absences: &[AbsencePeriod], person_filter: Option<Uuid>, year: u32, today: time::Date) -> (i64, i64, usize)` near `compute_status`. Lifts the existing StatsGrid loop verbatim with an added `person_filter` guard. Person dimension ONLY — no category/status filtering (D1).
- Refactored `StatsGrid` to call `stats_for_person` instead of inlining the loop.
- Added `person_filter: Option<Uuid>` prop to `StatsGridProps` with `#[props(!optional, default = None)]`.
- Wired `person_filter: person_filter_val` at the AbsencesPage render call site (~line 1880). Passes unfiltered `absences.clone()` — the helper does the scoping.
- 5 RED-first unit tests: `none_counts_all_persons`, `some_scopes_to_that_person`, `sick_and_unpaid_inclusive_day_count`, `active_count_uses_compute_status`, `excludes_out_of_year`.

### Task 2: VacationEntitlementCard self-view branch (D2, D3)

- Added `selected_person: Option<Uuid>` prop to `VacationEntitlementCardProps` with `#[props(!optional, default = None)]`.
- Compute `forced_self: Option<VacationBalance>` by finding the matching balance in `vacation_team` for the selected person.
- `show_self = !props.is_hr || forced_self.is_some()` — renders `VacationEntitlementSelfBody` when true, `VacationEntitlementHrBody` when false.
- Self-view path: `vacation_self: forced_self.clone().or(props.vacation_self.clone())` — for HR-with-selection uses the selected person's balance; for employee falls back to their own.
- Wired `selected_person: person_filter_val` at AbsencesPage render call site.
- No i18n changes — reuses existing `Key::VacationCardSelfTitle`, `Key::VacationCardSelfSubtitle`, `Key::VacationEntitlementHero`.
- 3 snapshot tests: HR-with-selection shows self-body + excludes team aggregate / HR-without-selection shows team aggregate / missing-from-team falls back without panic.

## Test Results

| Suite | Before | After | Delta |
|-------|--------|-------|-------|
| `page::absences` | 24 | 32 | +8 |
| Total shifty-dioxus | 565 | 573 | +8 |

All tests pass. WASM build exits 0.

## Verify Gates Passed

- `cargo test stats_for_person` — 5/5 ok
- `cargo test vacation_card` — 3/3 ok
- `cargo test page::absences` — 32/32 ok
- `nix develop -c cargo build --target wasm32-unknown-unknown` — exit 0
- `git diff --stat` (read-only) — only `shifty-dioxus/src/page/absences.rs` (1 file, +147/-15)

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None.

## Threat Flags

None — no new network endpoints, no auth path changes, no backend changes. Matches T-o7t-01/T-o7t-02 `accept` dispositions (HR already has vacation_team loaded and authorized; filter is read-only UI signal).

## Self-Check: PASSED

- `/home/neosam/programming/rust/projects/shifty/shifty-backend/.planning/quick/260612-o7t-absencespage-sales-person-filter/260612-o7t-SUMMARY.md` — this file
- Task 1 commit 1899d8e5 — confirmed in jj log
- Task 2 commit 5402438f — confirmed in jj log
- Only `shifty-dioxus/src/page/absences.rs` modified — confirmed by `git diff --stat`
