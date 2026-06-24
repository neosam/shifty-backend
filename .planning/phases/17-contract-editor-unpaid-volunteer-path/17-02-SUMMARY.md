---
phase: 17-contract-editor-unpaid-volunteer-path
plan: "02"
subsystem: backend/booking_information
tags: [D-05, D-06, CVC-10, achse-b, committed-voluntary, gate-extension]
requirements: [CVC-10]

dependency_graph:
  requires:
    - "Phase 15: two-band model (Band 1/Band 2) + cap-gate in get_weekly_summary"
    - "Phase 16: committed_voluntary_hours display pipeline (Band 1 rendered)"
    - "Phase 17-01: is_paid gate (Achse A, billing/paid_hours)"
  provides:
    - "D-05: extended committed_voluntary read-gate for expected_hours=0 persons in year-view (Band 1 + Band 2)"
    - "D-06: rein-unbezahlte Freiwillige (is_paid=false, expected_hours=0) flow into Band 1 committed_voluntary_hours"
  affects:
    - "service_impl/src/booking_information.rs (get_weekly_summary, first variant)"
    - "service_impl/src/test/booking_information.rs (3 new D-05 fixture tests)"

tech_stack:
  added: []
  patterns:
    - "Extended filter predicate: cap_planned_hours_to_expected || expected_hours == 0.0 (symmetry with D-01 editor visibility)"
    - "Pure-logic fixture tests for filter gate extension (no mock required)"

key_files:
  modified:
    - "service_impl/src/booking_information.rs — two .filter() extensions in first get_weekly_summary variant"
    - "service_impl/src/test/booking_information.rs — 3 new D-05 tests + updated snapshot_schema_version comment"

decisions:
  - "Test variant chosen: pure-logic filter predicate tests over synthetic (cap, expected_hours, committed) tuples — consistent with the existing helper-level test style; no service mock needed (D-05 gate is a simple boolean predicate)"
  - "band1_committed_with_d05_gate helper function: replicates exactly the production .filter + .map + .sum pipeline for testability"
  - "Comment on snapshot_schema_version_unchanged_at_7 updated to mention Phase 17 D-05 + Plan 01 as additional non-bump cases"

metrics:
  duration: "~5 min"
  completed: "2026-06-24"
  tasks_completed: 1
  tasks_total: 1
  files_modified: 2
---

# Phase 17 Plan 02: D-05 committed_voluntary Read-Gate Extension Summary

**One-liner:** Extended both get_weekly_summary Band-1 and Band-2 `.filter()` gates from `cap_planned_hours_to_expected` to `cap_planned_hours_to_expected || expected_hours == 0.0` (D-05), allowing rein-unbezahlte Freiwillige to flow their pledge into committed_voluntary_hours (Achse B, Jahresansicht).

## What Changed

### Task 1: D-05 Gate-Erweiterung an beiden .filter-Stellen + Fixture-Tests

**File:** `service_impl/src/booking_information.rs`

Two `.filter()` calls in the **first** `get_weekly_summary` variant (year-view, lines 136–295) were extended:

**Stelle 1 — Band-2-Surplus-Loop** (line 210–213):
```rust
// Before:
&& wh.cap_planned_hours_to_expected // CVC-06 per row

// After:
&& (wh.cap_planned_hours_to_expected || wh.expected_hours == 0.0) // D-05: cap || rein-freiwillig (expected_hours=0)
```

**Stelle 2 — Band-1-committed-Summe** (line 224):
```rust
// Before:
.filter(|wh| wh.cap_planned_hours_to_expected) // CVC-06 gate, per row

// After:
.filter(|wh| wh.cap_planned_hours_to_expected || wh.expected_hours == 0.0) // D-05: cap || rein-freiwillig (expected_hours=0), symmetrisch zu D-01 Editor-Sichtbarkeit
```

**Second variant (`get_summery_for_week`, lines 297–562) left UNTOUCHED** — `committed_voluntary_hours: 0.0` placeholder at line 547 confirmed preserved.

**File:** `service_impl/src/test/booking_information.rs`

Three new D-05 fixture tests added (lines 383–440), using a `band1_committed_with_d05_gate` helper that replicates the production filter predicate over synthetic `(cap, expected_hours, committed)` tuples:

- `d05_expected_hours_zero_flows_into_band1` — cap=false, expected_hours=0.0, committed=5.0 → Band 1 = 5.0 (new gate branch fires)
- `d05_capped_person_still_counted` — cap=true, expected_hours=40.0, committed=3.0 → Band 1 = 3.0 (backward-compat)
- `d05_uncapped_nonzero_excluded` — cap=false, expected_hours=40.0, committed=7.0 → Band 1 = 0.0 (neither branch fires)

**`snapshot_schema_version_unchanged_at_7` comment updated** with Phase 17 addendum noting D-05 gate extension and Plan 01 is_paid gate are both Achse-B-only, no bump.

## Test Choice Rationale

Existing tests in this file test pure helper functions (`volunteer_surplus_above_committed`, `volunteer_surplus_band2`) without service mocks. The D-05 gate is a boolean predicate (`cap || expected_hours == 0.0`) — a pure-logic test over synthetic tuples is the most direct way to pin it, without introducing mock overhead. The `band1_committed_with_d05_gate` helper replicates exactly the production pipeline (`.filter` + `.map` + `.sum`).

## Acceptance Criteria Verification

- `grep -c "expected_hours == 0.0" service_impl/src/booking_information.rs` → **2** (both gates)
- Both D-05 gate lines confirmed at lines 212 and 224
- `grep -c "committed_voluntary_hours: 0.0" service_impl/src/booking_information.rs` → **3** (placeholder in second variant preserved, plus two in first variant struct literal)
- All three D-05 test names present in test file (lines 402, 415, 427)
- `CURRENT_SNAPSHOT_SCHEMA_VERSION` = **7** (unchanged, billing_period_report.rs line 75)

## Build and Test Results

- `cargo build` (workspace): **PASSED** (20.83s, no errors)
- `cargo test -p service_impl booking_information`: **22/22 PASSED** (19 existing + 3 new D-05)
- `cargo test -p service_impl` (full suite): **445/445 PASSED**, 0 failed

## Deviations from Plan

None — plan executed exactly as written.

## VCS Note

All changes are uncommitted in the working tree. The user commits manually via `jj`. Files to commit:
- `service_impl/src/booking_information.rs`
- `service_impl/src/test/booking_information.rs`

## Self-Check: PASSED

- `service_impl/src/booking_information.rs`: FOUND (two gate extensions at lines 212, 224)
- `service_impl/src/test/booking_information.rs`: FOUND (three D-05 tests at lines 402, 415, 427)
- `CURRENT_SNAPSHOT_SCHEMA_VERSION = 7`: CONFIRMED (billing_period_report.rs line 75)
- Build: PASSED
- Tests: 445 passed, 0 failed
