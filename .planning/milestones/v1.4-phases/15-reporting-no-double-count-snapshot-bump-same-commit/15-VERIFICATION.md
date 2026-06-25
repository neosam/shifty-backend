---
phase: 15-reporting-no-double-count-snapshot-bump-same-commit
verified: 2026-06-24T00:00:00Z
status: passed
score: 8/8 must-haves verified
overrides_applied: 0
---

# Phase 15: Reporting No-Double-Count Verification Report

**Phase Goal:** Decompose freiwillige-Kapazität into two stacked bands on WeeklySummary (Achse B only) — Band 1 `committed_voluntary_hours` (cap-gated Σ committed pledge), Band 2 `volunteer_hours` reduced to per-person surplus max(actual−committed,0) (FORMULA B, no-double-count CVC-04), cap-gated (CVC-06); NO snapshot schema version bump (CVC-05, version stays 7); maximal test coverage (D-02).
**Verified:** 2026-06-24
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `WeeklySummary` carries a NEW separate field `committed_voluntary_hours: f32` (Band 1, not folded into paid_hours or volunteer_hours) | VERIFIED | `service/src/booking_information.rs:45` — field present with correct Band 1/2 sibling comments |
| 2 | Two stacked bands per person/ISO-week — Band 1 = cap-gated Σ committed, Band 2 = Σ max(actual_p − committed_p, 0) via FORMULA B | VERIFIED | `service_impl/src/booking_information.rs:207-226` — `volunteer_surplus_band2` used for Band 2; explicit per-row cap filter for Band 1 |
| 3 | CVC-04 no-double-count: per-person subtraction using `volunteer_surplus_band2` that aggregates per-day rows into per-person weekly totals BEFORE applying max (CR-01 fix present) | VERIFIED | `service_impl/src/booking_information.rs:52-67` — `volunteer_surplus_band2` aggregates per-day rows into HashMap before max; regression tests `cvc04_multi_day_single_person` and `cvc04_multi_day_multi_person` pass |
| 4 | Multi-person FORMULA B: A(cap,c=5,a=0)+B(c=0,a=3) ⇒ committed_voluntary_hours=5, volunteer_hours=3, total=8 | VERIFIED | `service_impl/src/test/booking_information.rs:163-197` — `cvc04_multi_person` test asserts all three values, passes |
| 5 | Pure surplus helper `volunteer_surplus_above_committed` is unit-testable in isolation without mocking service deps | VERIFIED | `service_impl/src/booking_information.rs:35-37` — `pub(crate) fn volunteer_surplus_above_committed(actual: f32, committed: f32) -> f32`; inline tests pass (surplus_over_fulfilled, surplus_pledge_covers, surplus_committed_zero_backward_compat) |
| 6 | CVC-06: committed term AND per-person committed subtracted in Band 2 are gated on `cap_planned_hours_to_expected == true`; cap=false rows contribute 0.0 | VERIFIED | `service_impl/src/booking_information.rs:210-212` (Band 2 filter) and `:224` (Band 1 filter); `cvc06_cap_false_zero` and `cvc06_committed_zero_backward_compat` tests pass |
| 7 | CVC-05 NO snapshot version bump — `CURRENT_SNAPSHOT_SCHEMA_VERSION` stays 7 | VERIFIED | `service_impl/src/billing_period_report.rs:75` — `pub const CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 7`; `snapshot_schema_version_unchanged_at_7` regression test passes |
| 8 | D-02 maximal test coverage — full fixture suite with epsilon comparisons, no f32 `==` | VERIFIED | `service_impl/src/test/booking_information.rs` — 11 tests total (9 D-05 fixtures + 2 CR-01 regression tests + 1 version regression); all use `approx()` epsilon helper; all pass |

**Score:** 8/8 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `service/src/booking_information.rs` | `committed_voluntary_hours: f32` on WeeklySummary | VERIFIED | Line 45; adjacent to `volunteer_hours` as siblings |
| `service_impl/src/booking_information.rs` | Band 1 + Band 2 wired into `get_weekly_summary`; `volunteer_surplus_above_committed` helper; `volunteer_surplus_band2` helper (CR-01 fix) | VERIFIED | Lines 35-67 (helpers), 187-226 (wiring in get_weekly_summary), 547 (0.0 placeholder in get_summery_for_week) |
| `service_impl/src/test/booking_information.rs` | D-05 two-band fixture suite + version regression test | VERIFIED | 11 `#[test]` functions; all pass; min_lines > 90 (320 lines) |
| `service_impl/src/test/mod.rs` | `pub mod booking_information` registered | VERIFIED | Line 4 |
| `service_impl/src/billing_period_report.rs` | `CURRENT_SNAPSHOT_SCHEMA_VERSION = 7` + Phase 15 no-bump comment | VERIFIED | Line 75 (const = 7); line 74 (Phase 15 comment added) |
| `.planning/REQUIREMENTS.md` | CVC-05 reworded to no-bump justification | VERIFIED | Line 19 — "(revidiert per D-01 — KEIN Bump)" wording present |
| `.planning/ROADMAP.md` | Phase 15 SC#3 and Goal reconciled from "bump 7→8" to no-bump | MOSTLY VERIFIED | SC#3 (line 148) and Goal block (line 138) correctly reconciled; HOWEVER: section heading at line 136 still reads "snapshot bump (SAME commit)" and "Depends on" at line 140 still says "Snapshot-Bump und Formel-Switch sind per Snapshot-Versioning-Contract atomar (selber Commit)" — these are orphaned sentences that contradict the no-bump decision; the acceptance criterion `grep -ni "7→8\|7->8" .planning/ROADMAP.md` → no results (PASSES), but the heading text and one Depends-on sentence are incomplete reconciliation artifacts |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `service_impl/src/booking_information.rs::get_weekly_summary` | `service_impl/src/reporting.rs::find_working_hours_for_calendar_week` | Band 1 per-row cap-gated committed sum + Band 2 per-person committed lookup | WIRED | `use crate::reporting::find_working_hours_for_calendar_week` at line 2; used at lines 209 and 219 |
| `service_impl/src/booking_information.rs::get_weekly_summary` | `volunteer_surplus_band2` | Band 2 per-person surplus with CR-01 fix (per-day aggregation before max) | WIRED | `volunteer_surplus_band2(per_day_actuals, ...)` at line 207 |
| `service_impl/src/test/booking_information.rs` | `service_impl/src/booking_information.rs::volunteer_surplus_above_committed` | Direct pure-function unit tests (FORMULA B) | WIRED | `use crate::booking_information::{volunteer_surplus_above_committed, volunteer_surplus_band2}` at line 12 |
| `service_impl/src/test/booking_information.rs` | `service_impl::billing_period_report::CURRENT_SNAPSHOT_SCHEMA_VERSION` | Version regression assertion `== 7` | WIRED | `crate::billing_period_report::CURRENT_SNAPSHOT_SCHEMA_VERSION` at line 316; asserts with `assert_eq!` |

### Data-Flow Trace (Level 4)

Not applicable — `WeeklySummary.committed_voluntary_hours` is deliberately inert on the wire (no TO mapping until Phase 16). The field is set by `get_weekly_summary` internal computation from `all_work_details` loaded from `EmployeeWorkDetailsService`. The computation itself is verified by pure-function unit tests.

### Behavioral Spot-Checks

| Behavior | Method | Result | Status |
|----------|--------|--------|--------|
| `volunteer_surplus_above_committed(7.0, 5.0) == 2.0` | `cargo test -p service_impl surplus_over_fulfilled` | ok | PASS |
| `volunteer_surplus_band2` aggregates per-day rows before max (CR-01 fix) | `cargo test -p service_impl cvc04_multi_day_single_person` | ok | PASS |
| Multi-person FORMULA B: committed=5, volunteer=3, total=8 | `cargo test -p service_impl cvc04_multi_person` | ok | PASS |
| Version stays 7 | `cargo test -p service_impl snapshot_schema_version_unchanged_at_7` | ok | PASS |
| Full workspace green | `cargo test --workspace` | 17 result lines, all `ok. X passed; 0 failed` | PASS |

### Requirements Coverage

| Requirement | Source Plans | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| CVC-04 | 15-01, 15-02 | No-double-count per-person decomposition into two bands, FORMULA B | SATISFIED | `volunteer_surplus_band2` helper + Band 1/2 wiring in `get_weekly_summary`; cvc04_* fixture suite all pass |
| CVC-05 | 15-02 | `CURRENT_SNAPSHOT_SCHEMA_VERSION` stays 7, no bump; audit-trail justified | SATISFIED | Const = 7 at line 75; `snapshot_schema_version_unchanged_at_7` passes; REQUIREMENTS.md reconciled |
| CVC-06 | 15-01, 15-02 | cap=false rows contribute 0.0 to both bands | SATISFIED | Per-row `.filter(cap_planned_hours_to_expected)` for Band 1 and Band 2; `cvc06_cap_false_zero` and `cvc06_committed_zero_backward_compat` pass |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `service_impl/src/booking_information.rs` | 547 | `committed_voluntary_hours: 0.0` in `get_summery_for_week` | INFO | Intentional placeholder per decision option (a) — documented; Phase 16 wires if needed |
| `.planning/ROADMAP.md` | 136 | Section heading still reads "snapshot bump (SAME commit)" | WARNING | Old heading text contradicts no-bump decision; SC#3 and Goal text are correctly reconciled; no "7→8" wording present; no code impact |
| `.planning/ROADMAP.md` | 140 | "Depends on" sentence: "Snapshot-Bump und Formel-Switch sind per Snapshot-Versioning-Contract atomar (selber Commit)" | WARNING | Orphaned contradiction of the no-bump decision; doc inconsistency only, no code impact |

### Human Verification Required

None — all truths verifiable programmatically. The `committed_voluntary_hours` field is inert on the wire (not yet displayed in frontend until Phase 16), so no UI verification is needed for Phase 15.

### Gaps Summary

No blockers. The phase goal is achieved:

1. `WeeklySummary` carries the new `committed_voluntary_hours` field (Band 1).
2. `get_weekly_summary` computes Band 1 (cap-gated Σ committed) and Band 2 (per-person surplus via `volunteer_surplus_band2`) correctly — the CR-01 per-day bug found in the review is confirmed fixed by the new `volunteer_surplus_band2` helper and the two regression tests.
3. All 17 phase-specific tests pass; full workspace (437 service_impl + 61 integration tests + others) is green with 0 failures.
4. `CURRENT_SNAPSHOT_SCHEMA_VERSION` stays 7 with a passing regression test.
5. REQUIREMENTS.md CVC-05 is reconciled to no-bump justification.
6. ROADMAP SC#3 and Goal block are correctly reconciled; two residual sentences in the detail block heading and "Depends on" clause still reference the old "snapshot bump" framing, but neither contains "7→8" and they have no code impact.

Two WARNING-level documentation artifacts in ROADMAP.md (lines 136 and 140) are incomplete reconciliation. These are doc-only inconsistencies and do not affect code correctness or test behavior. They can be cleaned up in the next doc-maintenance pass.

---

_Verified: 2026-06-24_
_Verifier: Claude (gsd-verifier)_
